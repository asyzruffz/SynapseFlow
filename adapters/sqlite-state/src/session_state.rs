use std::sync::Arc;
use std::{path::Path, sync::Mutex};

use rusqlite::{params, Connection, OptionalExtension, Transaction};
use synapseflow_domain::execution::{
    CheckpointRef, FrameSequence, SafeTraceId, SessionId, StreamId,
};
use synapseflow_domain::{
    AdmissionDecision, AdmissionRejection, AuthenticatedPrincipal, CancellationResult, DomainError,
    DomainResult, GrantedScope, GrantedScopes, IdempotencyKey, ModelReference, PrincipalPseudonym,
    PublicSessionId, PublicSessionState,
};
use synapseflow_ports::{
    ActiveSessionControl, AdmissionAccounting, AdmissionRequest, CreateSessionResult,
    DurableSession, ExecutionCancellation, RequestFingerprint, SessionIdentifierIssuer,
    SessionStore,
};
use uuid::Uuid;

use crate::{active_sessions::ActiveSessions, database};

/// Capacity limits atomically rechecked when a durable session is created.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SqliteNodeStateSettings {
    pub max_concurrent_sessions: usize,
    pub max_sessions_per_principal: usize,
}

/// Durable single-node session state and associated adapter implementations.
pub struct SqliteNodeState {
    connection: Mutex<Connection>,
    settings: SqliteNodeStateSettings,
    active: ActiveSessions,
}

impl SqliteNodeState {
    /// Opens or creates the local database and applies the current schema.
    pub fn open(path: impl AsRef<Path>, settings: SqliteNodeStateSettings) -> DomainResult<Self> {
        if settings.max_concurrent_sessions == 0 || settings.max_sessions_per_principal == 0 {
            return Err(DomainError::AdmissionUnavailable);
        }
        Ok(Self {
            connection: Mutex::new(database::open(path.as_ref())?),
            settings,
            active: ActiveSessions::default(),
        })
    }

    #[cfg(test)]
    fn in_memory(settings: SqliteNodeStateSettings) -> DomainResult<Self> {
        Ok(Self {
            connection: Mutex::new(database::open_in_memory()?),
            settings,
            active: ActiveSessions::default(),
        })
    }

    fn lock(&self) -> DomainResult<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|_| DomainError::PersistenceFailure)
    }
}

impl SessionIdentifierIssuer for SqliteNodeState {
    fn issue(&self) -> DomainResult<PublicSessionId> {
        PublicSessionId::new(format!("node-session-{}", Uuid::new_v4().simple()))
    }
}

impl AdmissionAccounting for SqliteNodeState {
    fn admit(&self, request: &AdmissionRequest) -> DomainResult<AdmissionDecision> {
        let connection = self.lock().map_err(|_| DomainError::AdmissionUnavailable)?;
        admission_decision(&connection, &request.principal, self.settings)
            .map_err(|_| DomainError::AdmissionUnavailable)
    }

    fn release(&self, _: &PublicSessionId) -> DomainResult<()> {
        // Capacity is derived from durable non-terminal rows, so a committed
        // terminal state is its own release and cannot be lost on restart.
        Ok(())
    }
}

impl SessionStore for SqliteNodeState {
    fn create(&self, session: DurableSession) -> DomainResult<CreateSessionResult> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction()
            .map_err(|_| DomainError::PersistenceFailure)?;

        if let Some(existing) = find_by_idempotency(
            &transaction,
            &session.owner,
            session.idempotency_key.as_ref(),
        )? {
            return if existing.request_fingerprint == session.request_fingerprint {
                Ok(CreateSessionResult::Replayed(existing))
            } else {
                Ok(CreateSessionResult::Conflict)
            };
        }
        if admission_decision(&transaction, &session.owner, self.settings)?
            != AdmissionDecision::Admitted
        {
            return Err(DomainError::AdmissionUnavailable);
        }
        insert_session(&transaction, &session)?;
        transaction
            .commit()
            .map_err(|_| DomainError::PersistenceFailure)?;
        Ok(CreateSessionResult::Created(session))
    }

    fn load(&self, session_id: &PublicSessionId) -> DomainResult<Option<DurableSession>> {
        let connection = self.lock()?;
        load(&connection, session_id)
    }

    fn find_by_idempotency(
        &self,
        owner: &AuthenticatedPrincipal,
        key: &IdempotencyKey,
    ) -> DomainResult<Option<DurableSession>> {
        let connection = self.lock()?;
        find_by_idempotency(&connection, owner, Some(key))
    }

    fn replace(&self, session: DurableSession) -> DomainResult<()> {
        let mut connection = self.lock()?;
        let transaction = connection
            .transaction()
            .map_err(|_| DomainError::PersistenceFailure)?;
        let changed = transaction
            .execute(
                "UPDATE node_sessions SET owner_scopes = ?2, model_reference = ?3,
                 idempotency_key = ?4, request_fingerprint = ?5, state = ?6, trace_id = ?7
                 WHERE session_id = ?1",
                params![
                    session.id.as_str(),
                    encode_scopes(session.owner.scopes()),
                    session.model.as_str(),
                    session.idempotency_key.as_ref().map(IdempotencyKey::as_str),
                    session.request_fingerprint.as_ref().map(fingerprint_bytes),
                    state_text(session.state),
                    session.trace_id.as_ref().map(SafeTraceId::as_str),
                ],
            )
            .map_err(|_| DomainError::PersistenceFailure)?;
        if changed == 0 {
            return Err(DomainError::SessionUnavailable);
        }
        transaction
            .execute(
                "DELETE FROM node_session_checkpoints WHERE session_id = ?1",
                params![session.id.as_str()],
            )
            .map_err(|_| DomainError::PersistenceFailure)?;
        insert_checkpoints(&transaction, &session)?;
        transaction
            .commit()
            .map_err(|_| DomainError::PersistenceFailure)
    }
}

impl ActiveSessionControl for SqliteNodeState {
    fn activate(
        &self,
        session_id: &PublicSessionId,
    ) -> DomainResult<Arc<dyn ExecutionCancellation>> {
        ActiveSessionControl::activate(&self.active, session_id)
    }

    fn request_cancellation(
        &self,
        session_id: &PublicSessionId,
    ) -> DomainResult<CancellationResult> {
        ActiveSessionControl::request_cancellation(&self.active, session_id)
    }

    fn deactivate(&self, session_id: &PublicSessionId) -> DomainResult<()> {
        ActiveSessionControl::deactivate(&self.active, session_id)
    }
}

fn admission_decision(
    connection: &Connection,
    principal: &AuthenticatedPrincipal,
    settings: SqliteNodeStateSettings,
) -> DomainResult<AdmissionDecision> {
    let active: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM node_sessions WHERE state NOT IN ('completed', 'cancelled', 'failed')",
            [],
            |row| row.get(0),
        )
        .map_err(|_| DomainError::PersistenceFailure)?;
    if u64::try_from(active).map_err(|_| DomainError::PersistenceFailure)?
        >= u64::try_from(settings.max_concurrent_sessions)
            .map_err(|_| DomainError::PersistenceFailure)?
    {
        return Ok(AdmissionDecision::Rejected(
            AdmissionRejection::NodeCapacity,
        ));
    }
    let principal_active: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM node_sessions
             WHERE owner_pseudonym = ?1 AND state NOT IN ('completed', 'cancelled', 'failed')",
            params![principal.pseudonym().as_str()],
            |row| row.get(0),
        )
        .map_err(|_| DomainError::PersistenceFailure)?;
    if u64::try_from(principal_active).map_err(|_| DomainError::PersistenceFailure)?
        >= u64::try_from(settings.max_sessions_per_principal)
            .map_err(|_| DomainError::PersistenceFailure)?
    {
        Ok(AdmissionDecision::Rejected(
            AdmissionRejection::PrincipalCapacity,
        ))
    } else {
        Ok(AdmissionDecision::Admitted)
    }
}

fn insert_session(transaction: &Transaction<'_>, session: &DurableSession) -> DomainResult<()> {
    transaction
        .execute(
            "INSERT INTO node_sessions (
                session_id, owner_pseudonym, owner_scopes, model_reference,
                idempotency_key, request_fingerprint, state, trace_id
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                session.id.as_str(),
                session.owner.pseudonym().as_str(),
                encode_scopes(session.owner.scopes()),
                session.model.as_str(),
                session.idempotency_key.as_ref().map(IdempotencyKey::as_str),
                session.request_fingerprint.as_ref().map(fingerprint_bytes),
                state_text(session.state),
                session.trace_id.as_ref().map(SafeTraceId::as_str),
            ],
        )
        .map_err(|_| DomainError::PersistenceFailure)?;
    insert_checkpoints(transaction, session)
}

fn insert_checkpoints(transaction: &Transaction<'_>, session: &DurableSession) -> DomainResult<()> {
    for (position, checkpoint) in session.checkpoints.iter().enumerate() {
        transaction
            .execute(
                "INSERT INTO node_session_checkpoints (
                    session_id, position, execution_session_id, stream_id, frame_sequence
                 ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    session.id.as_str(),
                    i64::try_from(position).map_err(|_| DomainError::PersistenceFailure)?,
                    checkpoint.session_id.as_str(),
                    sqlite_integer(checkpoint.stream_id.value())?,
                    sqlite_integer(checkpoint.sequence.value())?,
                ],
            )
            .map_err(|_| DomainError::PersistenceFailure)?;
    }
    Ok(())
}

fn load(connection: &Connection, id: &PublicSessionId) -> DomainResult<Option<DurableSession>> {
    let session = connection
        .query_row(
            "SELECT session_id, owner_pseudonym, owner_scopes, model_reference,
                    idempotency_key, request_fingerprint, state, trace_id
             FROM node_sessions WHERE session_id = ?1",
            params![id.as_str()],
            read_session,
        )
        .optional()
        .map_err(|_| DomainError::PersistenceFailure)?;
    session
        .map(|mut session| {
            session.checkpoints = load_checkpoints(connection, &session.id)?;
            Ok(session)
        })
        .transpose()
}

fn find_by_idempotency(
    connection: &Connection,
    owner: &AuthenticatedPrincipal,
    key: Option<&IdempotencyKey>,
) -> DomainResult<Option<DurableSession>> {
    let Some(key) = key else {
        return Ok(None);
    };
    let session = connection
        .query_row(
            "SELECT session_id, owner_pseudonym, owner_scopes, model_reference,
                    idempotency_key, request_fingerprint, state, trace_id
             FROM node_sessions WHERE owner_pseudonym = ?1 AND idempotency_key = ?2",
            params![owner.pseudonym().as_str(), key.as_str()],
            read_session,
        )
        .optional()
        .map_err(|_| DomainError::PersistenceFailure)?;
    session
        .map(|mut session| {
            session.checkpoints = load_checkpoints(connection, &session.id)?;
            Ok(session)
        })
        .transpose()
}

fn read_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<DurableSession> {
    let id = PublicSessionId::new(row.get::<_, String>(0)?).map_err(domain_error)?;
    let pseudonym = PrincipalPseudonym::new(row.get::<_, String>(1)?).map_err(domain_error)?;
    let scopes = decode_scopes(&row.get::<_, String>(2)?).map_err(domain_error)?;
    let model = ModelReference::parse(row.get::<_, String>(3)?).map_err(domain_error)?;
    let idempotency_key = row
        .get::<_, Option<String>>(4)?
        .map(IdempotencyKey::new)
        .transpose()
        .map_err(domain_error)?;
    let request_fingerprint = row
        .get::<_, Option<Vec<u8>>>(5)?
        .map(fingerprint_from_bytes)
        .transpose()
        .map_err(domain_error)?;
    let state = parse_state(&row.get::<_, String>(6)?).map_err(domain_error)?;
    let trace_id = row
        .get::<_, Option<String>>(7)?
        .map(SafeTraceId::new)
        .transpose()
        .map_err(domain_error)?;
    Ok(DurableSession {
        id,
        owner: AuthenticatedPrincipal::new(pseudonym, scopes),
        model,
        idempotency_key,
        request_fingerprint,
        state,
        checkpoints: Vec::new(),
        trace_id,
    })
}

fn load_checkpoints(
    connection: &Connection,
    session_id: &PublicSessionId,
) -> DomainResult<Vec<CheckpointRef>> {
    let mut statement = connection
        .prepare(
            "SELECT execution_session_id, stream_id, frame_sequence
             FROM node_session_checkpoints WHERE session_id = ?1 ORDER BY position",
        )
        .map_err(|_| DomainError::PersistenceFailure)?;
    let checkpoints = statement
        .query_map(params![session_id.as_str()], |row| {
            let execution_session_id =
                SessionId::new(row.get::<_, String>(0)?).map_err(domain_error)?;
            let stream_id =
                StreamId::new(sqlite_unsigned(row.get::<_, i64>(1)?).map_err(domain_error)?)
                    .map_err(domain_error)?;
            Ok(CheckpointRef {
                session_id: execution_session_id,
                stream_id,
                sequence: FrameSequence::new(
                    sqlite_unsigned(row.get::<_, i64>(2)?).map_err(domain_error)?,
                ),
            })
        })
        .map_err(|_| DomainError::PersistenceFailure)?;
    checkpoints
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| DomainError::PersistenceFailure)
}

fn encode_scopes(scopes: &GrantedScopes) -> String {
    scopes
        .iter()
        .map(GrantedScope::as_str)
        .collect::<Vec<_>>()
        .join(" ")
}

fn decode_scopes(value: &str) -> DomainResult<GrantedScopes> {
    value
        .split_whitespace()
        .map(GrantedScope::parse)
        .collect::<DomainResult<Vec<_>>>()
        .map(GrantedScopes::new)
}

fn fingerprint_bytes(fingerprint: &RequestFingerprint) -> Vec<u8> {
    fingerprint.as_bytes().to_vec()
}

fn fingerprint_from_bytes(bytes: Vec<u8>) -> DomainResult<RequestFingerprint> {
    let bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| DomainError::PersistenceFailure)?;
    Ok(RequestFingerprint::new(bytes))
}

fn state_text(state: PublicSessionState) -> &'static str {
    match state {
        PublicSessionState::Accepted => "accepted",
        PublicSessionState::Running => "running",
        PublicSessionState::Cancelling => "cancelling",
        PublicSessionState::Completed => "completed",
        PublicSessionState::Cancelled => "cancelled",
        PublicSessionState::Failed => "failed",
    }
}

fn parse_state(value: &str) -> DomainResult<PublicSessionState> {
    match value {
        "accepted" => Ok(PublicSessionState::Accepted),
        "running" => Ok(PublicSessionState::Running),
        "cancelling" => Ok(PublicSessionState::Cancelling),
        "completed" => Ok(PublicSessionState::Completed),
        "cancelled" => Ok(PublicSessionState::Cancelled),
        "failed" => Ok(PublicSessionState::Failed),
        _ => Err(DomainError::PersistenceFailure),
    }
}

fn domain_error(_: DomainError) -> rusqlite::Error {
    rusqlite::Error::InvalidQuery
}

fn sqlite_integer(value: u64) -> DomainResult<i64> {
    i64::try_from(value).map_err(|_| DomainError::PersistenceFailure)
}

fn sqlite_unsigned(value: i64) -> DomainResult<u64> {
    u64::try_from(value).map_err(|_| DomainError::PersistenceFailure)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use synapseflow_domain::execution::{CheckpointRef, FrameSequence, SessionId, StreamId};
    use synapseflow_domain::{
        AdmissionDecision, AuthenticatedPrincipal, GrantedScope, GrantedScopes, ModelReference,
        PrincipalPseudonym, PublicSessionState,
    };
    use synapseflow_ports::{
        AdmissionAccounting, AdmissionRequest, RequestFingerprint, SessionStore,
    };

    use super::{SqliteNodeState, SqliteNodeStateSettings};

    fn settings() -> SqliteNodeStateSettings {
        SqliteNodeStateSettings {
            max_concurrent_sessions: 2,
            max_sessions_per_principal: 1,
        }
    }

    fn principal() -> AuthenticatedPrincipal {
        AuthenticatedPrincipal::new(
            PrincipalPseudonym::new("owner_0001".to_owned()).expect("fixture principal"),
            GrantedScopes::new([GrantedScope::Generate]),
        )
    }

    fn session(state: &SqliteNodeState) -> synapseflow_ports::DurableSession {
        synapseflow_ports::DurableSession {
            id: synapseflow_ports::SessionIdentifierIssuer::issue(state).expect("identifier"),
            owner: principal(),
            model: ModelReference::parse(format!(
                "registry://fixtures/model@sha256:{}",
                "a".repeat(64)
            ))
            .expect("fixture model"),
            idempotency_key: Some(
                synapseflow_domain::IdempotencyKey::new("idempotency-00001".to_owned())
                    .expect("fixture key"),
            ),
            request_fingerprint: Some(RequestFingerprint::new([7; 32])),
            state: PublicSessionState::Accepted,
            checkpoints: vec![CheckpointRef {
                session_id: SessionId::new("execution-session-0001".to_owned())
                    .expect("fixture session"),
                stream_id: StreamId::new(1).expect("fixture stream"),
                sequence: FrameSequence::new(12),
            }],
            trace_id: None,
        }
    }

    #[test]
    fn persists_idempotency_and_checkpoints_without_storing_request_payloads() {
        let state = SqliteNodeState::in_memory(settings()).expect("state opens");
        let candidate = session(&state);
        assert!(matches!(
            state.create(candidate.clone()),
            Ok(synapseflow_ports::CreateSessionResult::Created(_))
        ));
        let loaded = state
            .load(&candidate.id)
            .expect("load")
            .expect("session exists");
        assert_eq!(loaded, candidate);
        assert!(matches!(
            state.create(candidate),
            Ok(synapseflow_ports::CreateSessionResult::Replayed(_))
        ));
    }

    #[test]
    fn creation_rechecks_durable_principal_capacity_transactionally() {
        let state = Arc::new(SqliteNodeState::in_memory(settings()).expect("state opens"));
        let first = session(&state);
        state.create(first).expect("first session persists");
        let admission = state
            .admit(&AdmissionRequest {
                principal: principal(),
                session_id: synapseflow_ports::SessionIdentifierIssuer::issue(&*state)
                    .expect("identifier"),
                reserved_output_tokens: 4,
            })
            .expect("admission query");
        assert!(matches!(admission, AdmissionDecision::Rejected(_)));
    }
}
