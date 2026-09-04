use std::{
    collections::BTreeSet,
    str::FromStr,
    sync::Mutex,
    time::{Duration, Instant},
};

use jsonwebtoken::{
    decode, decode_header,
    jwk::{Jwk, JwkSet},
    Algorithm, DecodingKey, Validation,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use synapseflow_domain::{
    AuthenticatedPrincipal, DomainError, DomainResult, GrantedScope, GrantedScopes,
    PrincipalPseudonym,
};
use synapseflow_ports::{BearerCredential, IdentityVerifier};

use crate::KeycloakSettings;

/// Fetches verified issuer metadata and its current signing-key set.
pub trait KeycloakMetadataSource: Send + Sync {
    fn jwks(&self, issuer: &str) -> Result<JwkSet, KeycloakMetadataError>;
}

/// Deliberately detail-free discovery/JWKS failure at the node trust boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeycloakMetadataError {
    Unavailable,
}

/// HTTPS Keycloak discovery/JWKS client used by the production composition root.
pub struct HttpKeycloakMetadataSource {
    client: reqwest::blocking::Client,
}

impl HttpKeycloakMetadataSource {
    pub fn new() -> Result<Self, KeycloakMetadataError> {
        reqwest::blocking::Client::builder()
            .https_only(true)
            .timeout(Duration::from_secs(5))
            .build()
            .map(|client| Self { client })
            .map_err(|_| KeycloakMetadataError::Unavailable)
    }
}

impl KeycloakMetadataSource for HttpKeycloakMetadataSource {
    fn jwks(&self, issuer: &str) -> Result<JwkSet, KeycloakMetadataError> {
        let issuer_url =
            reqwest::Url::parse(issuer).map_err(|_| KeycloakMetadataError::Unavailable)?;
        let discovery_url = issuer_url
            .join(".well-known/openid-configuration")
            .map_err(|_| KeycloakMetadataError::Unavailable)?;
        let discovery = self
            .client
            .get(discovery_url)
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .map_err(|_| KeycloakMetadataError::Unavailable)?
            .json::<OpenIdConfiguration>()
            .map_err(|_| KeycloakMetadataError::Unavailable)?;
        if discovery.issuer != issuer {
            return Err(KeycloakMetadataError::Unavailable);
        }
        let jwks_url = reqwest::Url::parse(&discovery.jwks_uri)
            .map_err(|_| KeycloakMetadataError::Unavailable)?;
        if jwks_url.origin() != issuer_url.origin() || jwks_url.scheme() != "https" {
            return Err(KeycloakMetadataError::Unavailable);
        }
        self.client
            .get(jwks_url)
            .send()
            .and_then(reqwest::blocking::Response::error_for_status)
            .map_err(|_| KeycloakMetadataError::Unavailable)?
            .json::<JwkSet>()
            .map_err(|_| KeycloakMetadataError::Unavailable)
    }
}

#[derive(Deserialize)]
struct OpenIdConfiguration {
    issuer: String,
    jwks_uri: String,
}

struct CachedJwks {
    fetched_at: Instant,
    keys: JwkSet,
}

/// Locally validates Keycloak access tokens against a bounded JWKS cache.
pub struct KeycloakIdentityVerifier<S> {
    settings: KeycloakSettings,
    source: S,
    cache: Mutex<Option<CachedJwks>>,
}

impl<S> KeycloakIdentityVerifier<S>
where
    S: KeycloakMetadataSource,
{
    pub fn new(settings: KeycloakSettings, source: S) -> Self {
        Self {
            settings,
            source,
            cache: Mutex::new(None),
        }
    }

    fn validation(&self) -> DomainResult<Validation> {
        let algorithms = self
            .settings
            .allowed_algorithms
            .iter()
            .map(|algorithm| {
                Algorithm::from_str(algorithm).map_err(|_| DomainError::AuthenticationInvalid)
            })
            .collect::<DomainResult<Vec<_>>>()?;
        let first = *algorithms
            .first()
            .ok_or(DomainError::AuthenticationInvalid)?;
        let mut validation = Validation::new(first);
        validation.algorithms = algorithms;
        validation.set_issuer(&[self.settings.issuer.as_str()]);
        validation.set_audience(&[self.settings.audience.as_str()]);
        validation.set_required_spec_claims(&["exp", "nbf", "iss", "aud", "sub"]);
        validation.validate_nbf = true;
        validation.leeway = self.settings.clock_skew_seconds;
        Ok(validation)
    }

    fn key_for(&self, kid: &str) -> DomainResult<Jwk> {
        let maximum_age = Duration::from_secs(self.settings.jwks_max_staleness_seconds);
        let mut cache = self
            .cache
            .lock()
            .map_err(|_| DomainError::AuthenticationInvalid)?;
        let requires_refresh = cache.as_ref().is_none_or(|cached| {
            cached.fetched_at.elapsed() > maximum_age || cached.keys.find(kid).is_none()
        });
        if requires_refresh {
            let keys = self
                .source
                .jwks(&self.settings.issuer)
                .map_err(|_| DomainError::AuthenticationInvalid)?;
            *cache = Some(CachedJwks {
                fetched_at: Instant::now(),
                keys,
            });
        }
        cache
            .as_ref()
            .and_then(|cached| cached.keys.find(kid))
            .cloned()
            .ok_or(DomainError::AuthenticationInvalid)
    }
}

impl<S> IdentityVerifier for KeycloakIdentityVerifier<S>
where
    S: KeycloakMetadataSource,
{
    fn verify(&self, credential: BearerCredential<'_>) -> DomainResult<AuthenticatedPrincipal> {
        let token = credential.expose_to_verifier();
        let header = decode_header(token).map_err(|_| DomainError::AuthenticationInvalid)?;
        let algorithm_name = format!("{:?}", header.alg);
        if !self.settings.allowed_algorithms.contains(&algorithm_name) {
            return Err(DomainError::AuthenticationInvalid);
        }
        let key_id = header.kid.ok_or(DomainError::AuthenticationInvalid)?;
        if key_id.is_empty() {
            return Err(DomainError::AuthenticationInvalid);
        }
        let key = DecodingKey::from_jwk(&self.key_for(&key_id)?)
            .map_err(|_| DomainError::AuthenticationInvalid)?;
        let decoded = decode::<AccessTokenClaims>(token, &key, &self.validation()?)
            .map_err(|_| DomainError::AuthenticationInvalid)?;
        principal_from_claims(decoded.claims)
    }
}

#[derive(Deserialize)]
struct AccessTokenClaims {
    sub: String,
    scope: String,
}

fn principal_from_claims(claims: AccessTokenClaims) -> DomainResult<AuthenticatedPrincipal> {
    if claims.sub.is_empty() || claims.scope.is_empty() {
        return Err(DomainError::AuthenticationInvalid);
    }
    let scope_values = claims.scope.split_ascii_whitespace().collect::<Vec<_>>();
    let unique_scope_values = scope_values.iter().copied().collect::<BTreeSet<_>>();
    if scope_values.len() != unique_scope_values.len() {
        return Err(DomainError::ScopeInvalid);
    }
    let scopes = scope_values
        .into_iter()
        .map(GrantedScope::parse)
        .collect::<DomainResult<Vec<_>>>()?;
    let principal_digest = Sha256::digest(claims.sub.as_bytes());
    let pseudonym = PrincipalPseudonym::new(format!("kc_{principal_digest:x}"))?;
    Ok(AuthenticatedPrincipal::new(
        pseudonym,
        GrantedScopes::new(scopes),
    ))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeSet, VecDeque},
        sync::{atomic::AtomicUsize, Arc, Mutex},
        time::{SystemTime, UNIX_EPOCH},
    };

    use jsonwebtoken::{encode, EncodingKey, Header};
    use serde::Serialize;
    use synapseflow_ports::{BearerCredential, IdentityVerifier};

    use super::{KeycloakIdentityVerifier, KeycloakMetadataSource};
    use crate::KeycloakSettings;

    const ISSUER: &str = "https://identity.example/realms/synapseflow";
    const AUDIENCE: &str = "synapseflow-node";
    const PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQCtOwJr4gpj+ca9\nsLiL3MEgdoLvd6Jr0PWlfsFdZee+81SSaEvj38ZzJQsftnnb0dibkdw4o//irkSi\n35NLIdmcszCkIsaPiPys7B8AyumnEzpT3G95Kk9Ke2RdgIqglx88/fum1vItwRhN\n1HT5ZBIxy0o5OeWt6kCScNrSmnRoupFJVgQjhkEZNYtH2KOaeWTKOy5ije6D8zL2\nCj5LpvdZA//5RP8V2TX5VZtTrNA6ZVA/2IXPrvRIT8SbblDED5/2oUrRYF1rkeL9\nFva/zpZ/B4nOSbBHTgGilCScVkLjhrWaWkQWCVXYaNbzE/Iu2UzVvZA30uSGVlUv\nOJoeysIhAgMBAAECggEAKfYoUP+xKqR/asWa/m4b7gQnFWCyXFGCn3MD3d7ocw24\nR7qx32H+TTgE6Mqn3AKJ6K09Xg8D1eIGyDlGEaYCc33IY4n09SHmqvCLVgLQ9GKo\n91VnPz9rc4xONIQFkH7q1zhis/hPM5wZigjTyPFfouYudYw7wZQDzjU+HFPDrPZW\nMey2i44vxkAgl/AYBqYx2hj6n0yq2fOmphvTW7I0QdFKx7Lsq/pXE03A1Q6X4frx\n+lMw3Wtb1Ky784cMI24fLoiAf4dm5x/HLDydeWAPPF+1hXHBJDypMv3IJ4bUxhEi\n1NsgPjBPFe7PPvBnt+Jdcx5bVBsXy+zDNJU8g+TQPQKBgQDGBavUwKeruDZMIQLt\nojzOJo71w7EvsRblxlx1qRpxrTb84OHtYMEmP+GMlVUOlu0YyGRbf4TgOEz2UuTB\n9N9fkiLeuGdFdrSc1BZYUPSQGhLEL4DV22natJfj13JDHP2jQfSHVXjiEQwDMj37\ne6Xu75Q/nh3akLuIbPo2R9ADiwKBgQDf8yITAAmiczRy5iBTinKJs5uhzkzd79Sy\nTM+Thg+1Mat+Htu3whuC80itLzOimlhWFeOLXZf9MgL0+0L61GB5d80+3RlOa+15\n09xS7mpsXjuEsc8NYTPvuElAkhPv1B1urs0WT5OkCjj+CiZwF3yfK1Qvxs4FBkk7\nhce/4oEWgwKBgFyrnsSL/GvCY7aw5DvtZuNa7CBbmnolOAAEGpT9tGBqnYcufsym\nMP4DezxdTlbrjr3AWibvwHFmJ65HEMMsI7UIIMV2Ku45JUEXh/WAvVMKwKmLSZHL\ngvhU95gq5VA/KvvSC+uhtlalf6enRZaBQSBWglxbVMFKZljsFxR7+v8NAoGBALrk\nCDea/G9ZfRe1/Jw7GcLY5LRvma5NC0+Q0lnmsw0fWmJyFiKQFq19odUFYy37aGTO\n94nCnahrKBSR6x+wRKKZ2+ruUMQlRZU5vNBort+o9DqUuJoN2G3heSAtx/2JItbP\ngc9wsWFgNpeqmNFKiHG8kxEb86o1yL+nsT7tI5VJAoGAEGvTW1b46hdd3AYDECrW\ncWOTEUVIjQiLPjyCyOsr4OwO8oVcCcSkKe2ct4uVkQWbNCrvyuTFfFxhZmekg99H\n0iDGjdPkWNezHJ9Ee81PbKhMU1AEAmu2tnQ4iKH7l9QAxxsA8qO+D/fkRZQJM5nL\n8McQa3XQeJcfdEO7RpI7Yx0=\n-----END PRIVATE KEY-----";
    const MODULUS: &str = "rTsCa-IKY_nGvbC4i9zBIHaC73eia9D1pX7BXWXnvvNUkmhL49_GcyULH7Z529HYm5HcOKP_4q5Eot-TSyHZnLMwpCLGj4j8rOwfAMrppxM6U9xveSpPSntkXYCKoJcfPP37ptbyLcEYTdR0-WQSMctKOTnlrepAknDa0pp0aLqRSVYEI4ZBGTWLR9ijmnlkyjsuYo3ug_My9go-S6b3WQP_-UT_Fdk1-VWbU6zQOmVQP9iFz670SE_Em25QxA-f9qFK0WBda5Hi_Rb2v86WfweJzkmwR04BopQknFZC44a1mlpEFglV2GjW8xPyLtlM1b2QN9LkhlZVLziaHsrCIQ";

    #[derive(Clone)]
    struct FixtureSource {
        responses: Arc<Mutex<VecDeque<jsonwebtoken::jwk::JwkSet>>>,
        calls: Arc<AtomicUsize>,
    }

    impl FixtureSource {
        fn new(responses: impl IntoIterator<Item = jsonwebtoken::jwk::JwkSet>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(responses.into_iter().collect())),
                calls: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    impl KeycloakMetadataSource for FixtureSource {
        fn jwks(&self, _: &str) -> Result<jsonwebtoken::jwk::JwkSet, super::KeycloakMetadataError> {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let mut responses = self
                .responses
                .lock()
                .map_err(|_| super::KeycloakMetadataError::Unavailable)?;
            responses
                .pop_front()
                .ok_or(super::KeycloakMetadataError::Unavailable)
        }
    }

    #[derive(Serialize)]
    struct Claims<'a> {
        iss: &'a str,
        aud: &'a str,
        sub: &'a str,
        scope: &'a str,
        exp: u64,
        nbf: u64,
    }

    fn now_seconds() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after the epoch")
            .as_secs()
    }

    fn settings() -> KeycloakSettings {
        KeycloakSettings {
            issuer: ISSUER.to_owned(),
            audience: AUDIENCE.to_owned(),
            allowed_algorithms: BTreeSet::from(["RS256".to_owned()]),
            jwks_max_staleness_seconds: 3_600,
            clock_skew_seconds: 0,
        }
    }

    fn jwks(kids: &[&str]) -> jsonwebtoken::jwk::JwkSet {
        serde_json::from_value(serde_json::json!({
            "keys": kids.iter().map(|kid| serde_json::json!({
                "kty": "RSA", "kid": kid, "alg": "RS256", "use": "sig", "n": MODULUS, "e": "AQAB"
            })).collect::<Vec<_>>()
        }))
        .expect("fixture JWKS should parse")
    }

    fn token(kid: &str, issuer: &str, audience: &str, exp: u64, nbf: u64, scope: &str) -> String {
        let mut header = Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some(kid.to_owned());
        encode(
            &header,
            &Claims {
                iss: issuer,
                aud: audience,
                sub: "service-account-node",
                scope,
                exp,
                nbf,
            },
            &EncodingKey::from_rsa_pem(PRIVATE_KEY.as_bytes()).expect("fixture key should parse"),
        )
        .expect("fixture token should encode")
    }

    #[test]
    fn verifies_a_valid_token_and_exposes_only_a_pseudonymous_principal() {
        let source = FixtureSource::new([jwks(&["key-1"])]);
        let verifier = KeycloakIdentityVerifier::new(settings(), source);
        let now = now_seconds();
        let credential = token(
            "key-1",
            ISSUER,
            AUDIENCE,
            now + 60,
            now - 1,
            "synapseflow:generate",
        );

        let principal = verifier
            .verify(BearerCredential::new(&credential).expect("fixture credential"))
            .expect("valid token should verify");

        assert!(principal.pseudonym().as_str().starts_with("kc_"));
        assert!(principal
            .scopes()
            .contains(synapseflow_domain::GrantedScope::Generate));
    }

    #[test]
    fn rejects_wrong_issuer_audience_expiry_and_unexpected_scope() {
        let now = now_seconds();
        for (issuer, audience, exp, scope) in [
            (
                "https://identity.example/realms/other",
                AUDIENCE,
                now + 60,
                "synapseflow:generate",
            ),
            (ISSUER, "other-audience", now + 60, "synapseflow:generate"),
            (
                ISSUER,
                AUDIENCE,
                now.saturating_sub(1),
                "synapseflow:generate",
            ),
            (ISSUER, AUDIENCE, now + 60, "profile"),
        ] {
            let source = FixtureSource::new([jwks(&["key-1"])]);
            let verifier = KeycloakIdentityVerifier::new(settings(), source);
            let credential = token("key-1", issuer, audience, exp, now.saturating_sub(1), scope);
            assert!(verifier
                .verify(BearerCredential::new(&credential).expect("fixture credential"))
                .is_err());
        }
    }

    #[test]
    fn refreshes_once_for_an_unknown_key_id_and_accepts_rotated_keys() {
        let source = FixtureSource::new([jwks(&["key-1"]), jwks(&["key-1", "key-2"])]);
        let calls = source.calls.clone();
        let verifier = KeycloakIdentityVerifier::new(settings(), source);
        let now = now_seconds();
        for kid in ["key-1", "key-2"] {
            let credential = token(
                kid,
                ISSUER,
                AUDIENCE,
                now + 60,
                now - 1,
                "synapseflow:generate",
            );
            verifier
                .verify(BearerCredential::new(&credential).expect("fixture credential"))
                .expect("rotated key should verify after one refresh");
        }
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    }
}
