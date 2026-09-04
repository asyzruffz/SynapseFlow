use std::{future::Future, sync::Arc, time::Duration};

use axum::{http::StatusCode, response::IntoResponse, Router};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder as HyperConnectionBuilder,
    service::TowerToHyperService,
};
use rustls::ServerConfig;
use rustls_pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};
use synapseflow_application::{GenerationSessionManager, SessionExecutionService};
use synapseflow_domain::{DomainResult, PublicSessionId};
use synapseflow_kernel::{Core, SynapseFlow};
use synapseflow_ports::{AuditSink, IdentityVerifier, ModelAccessPolicy, TelemetrySink};
use tokio::{net::TcpListener, sync::watch, task::JoinSet};
use tokio_rustls::TlsAcceptor;

use crate::{api, NodeError, NodeSettings, NodeWorkflowRegistry, TlsSettings};

/// Reusable HTTP listener construction surface. Public API routes are installed
/// in Step 5; the empty shell is intentionally deny-by-default until then.
pub struct NodeServer {
    settings: NodeSettings,
    dependencies: Option<NodeDependencies>,
    workflows: Arc<NodeWorkflowRegistry>,
}

/// Application services and port implementations selected by the CLI composition root.
pub struct NodeDependencies {
    pub identity: Arc<dyn IdentityVerifier>,
    pub audit: Arc<dyn AuditSink>,
    pub telemetry: Arc<dyn TelemetrySink>,
    pub model_policy: Arc<dyn ModelAccessPolicy>,
    pub sessions: Arc<GenerationSessionManager>,
    pub execution: Arc<SessionExecutionService>,
}

impl NodeServer {
    pub fn new(settings: NodeSettings) -> Result<Self, NodeError> {
        settings.validate()?;
        Ok(Self {
            settings,
            dependencies: None,
            workflows: Arc::new(NodeWorkflowRegistry::default()),
        })
    }

    pub fn with_dependencies(
        settings: NodeSettings,
        dependencies: NodeDependencies,
    ) -> Result<Self, NodeError> {
        settings.validate()?;
        Ok(Self {
            settings,
            dependencies: Some(dependencies),
            workflows: Arc::new(NodeWorkflowRegistry::default()),
        })
    }

    /// Returns the port implementations used by the future API routes.
    pub fn dependencies(&self) -> Option<&NodeDependencies> {
        self.dependencies.as_ref()
    }

    /// Registers one presentation workflow for an application-issued session.
    ///
    /// The workflow registry only owns the kernel view and subscriber bridge;
    /// application storage remains the authority for session lifecycle and
    /// authorization. HTTP handlers added in Step 5 call this after durable
    /// session creation succeeds.
    pub fn open_workflow(&self, session_id: PublicSessionId) -> DomainResult<()> {
        self.workflows
            .insert(session_id, Core::<SynapseFlow>::new())
    }

    /// Returns the node-owned workflow bridge used by HTTP/SSE handlers.
    pub fn workflows(&self) -> &NodeWorkflowRegistry {
        &self.workflows
    }

    /// Starts the public and management listener pair until the supplied future resolves.
    pub async fn serve_until<F>(self, shutdown: F) -> Result<(), NodeError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let (shutdown_sender, shutdown_receiver) = watch::channel(false);
        let drain = Duration::from_secs(self.settings.shutdown.drain_seconds);
        tokio::spawn(async move {
            shutdown.await;
            let _ = shutdown_sender.send(true);
        });
        let public = self
            .dependencies
            .map(|dependencies| {
                api::router(
                    Arc::new(dependencies),
                    self.settings.admission.max_request_bytes,
                    self.workflows.clone(),
                )
            })
            .unwrap_or_else(public_router);
        let management = serve_plain(
            self.settings.management_listener.bind,
            management_router(),
            shutdown_receiver.clone(),
        );

        match self.settings.public_tls {
            Some(tls) => {
                let public = serve_tls(
                    self.settings.public_listener.bind,
                    tls,
                    public,
                    shutdown_receiver,
                    drain,
                );
                tokio::try_join!(management, public)?;
                Ok(())
            }
            None => {
                let public = serve_plain(
                    self.settings.public_listener.bind,
                    public,
                    shutdown_receiver,
                );
                tokio::try_join!(management, public)?;
                Ok(())
            }
        }
    }
}

async fn serve_plain(
    address: std::net::SocketAddr,
    router: Router,
    shutdown: watch::Receiver<bool>,
) -> Result<(), NodeError> {
    let listener = TcpListener::bind(address)
        .await
        .map_err(|_| NodeError::ServerUnavailable)?;

    axum::serve(listener, router)
        .with_graceful_shutdown(wait_for_shutdown(shutdown))
        .await
        .map_err(|_| NodeError::ServerUnavailable)
}

async fn serve_tls(
    address: std::net::SocketAddr,
    tls: TlsSettings,
    router: Router,
    mut shutdown: watch::Receiver<bool>,
    drain: Duration,
) -> Result<(), NodeError> {
    let tls_acceptor = TlsAcceptor::from(Arc::new(load_tls_configuration(&tls)?));
    let listener = TcpListener::bind(address)
        .await
        .map_err(|_| NodeError::ServerUnavailable)?;
    let connection_builder = HyperConnectionBuilder::new(TokioExecutor::new());
    let mut connections = JoinSet::new();

    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (stream, _) = accepted.map_err(|_| NodeError::ServerUnavailable)?;
                let tls_acceptor = tls_acceptor.clone();
                let service = TowerToHyperService::new(router.clone());
                let connection_builder = connection_builder.clone();
                connections.spawn(async move {
                    let stream = tls_acceptor.accept(stream).await.map_err(|_| ())?;
                    connection_builder
                        .serve_connection(TokioIo::new(stream), service)
                        .await
                        .map_err(|_| ())
                });
            }
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    break;
                }
            }
        }
    }

    let _ = tokio::time::timeout(drain, wait_for_connections(&mut connections)).await;
    connections.abort_all();
    Ok(())
}

fn load_tls_configuration(tls: &TlsSettings) -> Result<ServerConfig, NodeError> {
    let certificates = CertificateDer::pem_file_iter(&tls.certificate_path)
        .map_err(|_| NodeError::ServerUnavailable)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| NodeError::ServerUnavailable)?;
    let private_key = PrivateKeyDer::from_pem_file(&tls.private_key_path)
        .map_err(|_| NodeError::ServerUnavailable)?;

    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certificates, private_key)
        .map_err(|_| NodeError::ServerUnavailable)
}

async fn wait_for_shutdown(mut shutdown: watch::Receiver<bool>) {
    if !*shutdown.borrow() {
        let _ = shutdown.changed().await;
    }
}

async fn wait_for_connections(connections: &mut JoinSet<Result<(), ()>>) {
    while connections.join_next().await.is_some() {}
}

fn public_router() -> Router {
    Router::new().fallback(not_found)
}

fn management_router() -> Router {
    Router::new().fallback(not_found)
}

async fn not_found() -> impl IntoResponse {
    StatusCode::NOT_FOUND
}
