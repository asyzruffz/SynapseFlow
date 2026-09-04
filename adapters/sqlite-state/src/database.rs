use std::{path::Path, time::Duration};

use rusqlite::Connection;
use synapseflow_domain::{DomainError, DomainResult};

pub(super) fn open(path: &Path) -> DomainResult<Connection> {
    let connection = Connection::open(path).map_err(|_| DomainError::PersistenceFailure)?;
    configure(&connection)?;
    migrate(&connection)?;
    Ok(connection)
}

#[cfg(test)]
pub(super) fn open_in_memory() -> DomainResult<Connection> {
    let connection = Connection::open_in_memory().map_err(|_| DomainError::PersistenceFailure)?;
    configure(&connection)?;
    migrate(&connection)?;
    Ok(connection)
}

fn configure(connection: &Connection) -> DomainResult<()> {
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(|_| DomainError::PersistenceFailure)?;
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = FULL;",
        )
        .map_err(|_| DomainError::PersistenceFailure)
}

fn migrate(connection: &Connection) -> DomainResult<()> {
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS node_sessions (
                session_id TEXT PRIMARY KEY NOT NULL,
                owner_pseudonym TEXT NOT NULL,
                owner_scopes TEXT NOT NULL,
                model_reference TEXT NOT NULL,
                idempotency_key TEXT,
                request_fingerprint BLOB,
                state TEXT NOT NULL,
                trace_id TEXT
            );
            CREATE UNIQUE INDEX IF NOT EXISTS node_sessions_owner_idempotency
                ON node_sessions(owner_pseudonym, idempotency_key)
                WHERE idempotency_key IS NOT NULL;
            CREATE TABLE IF NOT EXISTS node_session_checkpoints (
                session_id TEXT NOT NULL REFERENCES node_sessions(session_id) ON DELETE CASCADE,
                position INTEGER NOT NULL,
                execution_session_id TEXT NOT NULL,
                stream_id INTEGER NOT NULL,
                frame_sequence INTEGER NOT NULL,
                PRIMARY KEY(session_id, position)
            );
            CREATE INDEX IF NOT EXISTS node_sessions_active_owner
                ON node_sessions(owner_pseudonym, state);",
        )
        .map_err(|_| DomainError::PersistenceFailure)
}
