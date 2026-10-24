use std::path::Path;
use std::str::FromStr;
use std::time::Duration;
use serde_json::Value;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{Row, SqlitePool};

use crate::error::CrucibleError;
use crate::record::{OpusRunRecord, SuspendedTokenRecord, VesselEventRecord};

/// The SQLite-backed persistence and event sourcing engine for Rubedo workflows.
#[derive(Clone, Debug)]
pub struct CrucibleEngine {
    pool: SqlitePool,
}

impl CrucibleEngine {
    /// Establish a high-performance in-memory SQLite connection pool with embedded migrations applied.
    pub async fn connect_in_memory() -> Result<Self, CrucibleError> {
        let opts = SqliteConnectOptions::from_str("sqlite::memory:")?
            .foreign_keys(true)
            .busy_timeout(Duration::from_millis(5000));

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(opts)
            .await?;

        let engine = Self { pool };
        engine.migrate().await?;
        Ok(engine)
    }

    /// Connect to a persistent SQLite database file in WAL mode with migrations applied.
    pub async fn connect_file(path: impl AsRef<Path>) -> Result<Self, CrucibleError> {
        let opts = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .busy_timeout(Duration::from_millis(5000));

        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect_with(opts)
            .await?;

        let engine = Self { pool };
        engine.migrate().await?;
        Ok(engine)
    }

    /// Construct a `CrucibleEngine` directly from an existing connection pool.
    pub async fn from_pool(pool: SqlitePool) -> Result<Self, CrucibleError> {
        let engine = Self { pool };
        engine.migrate().await?;
        Ok(engine)
    }

    /// Reference the underlying `SqlitePool`.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Apply embedded database migrations.
    pub async fn migrate(&self) -> Result<(), CrucibleError> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| CrucibleError::Migration(e.to_string()))?;
        Ok(())
    }

    /// Insert a new Opus workflow run into the state store with initial 'Pending' status.
    pub async fn insert_run(
        &self,
        run_id: &str,
        opus_name: &str,
        initial_state: &Value,
    ) -> Result<(), CrucibleError> {
        let state_str = serde_json::to_string(initial_state)?;

        sqlx::query(
            "INSERT INTO opus_runs (run_id, opus_name, status, current_state) VALUES (?, ?, 'Pending', ?)"
        )
        .bind(run_id)
        .bind(opus_name)
        .bind(state_str)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update the lifecycle status and current state JSON snapshot of an existing run.
    pub async fn update_run_state(
        &self,
        run_id: &str,
        status: &str,
        current_state: &Value,
    ) -> Result<(), CrucibleError> {
        let state_str = serde_json::to_string(current_state)?;

        let result = sqlx::query(
            "UPDATE opus_runs SET status = ?, current_state = ?, updated_at = CURRENT_TIMESTAMP WHERE run_id = ?"
        )
        .bind(status)
        .bind(state_str)
        .bind(run_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(CrucibleError::RunNotFound(run_id.to_string()));
        }

        Ok(())
    }

    /// Append an immutable step execution transition event into the journal.
    pub async fn record_vessel_event(
        &self,
        run_id: &str,
        vessel_id: &str,
        status: &str,
        payload: Option<&Value>,
    ) -> Result<i64, CrucibleError> {
        let payload_str = match payload {
            Some(p) => Some(serde_json::to_string(p)?),
            None => None,
        };

        let row = sqlx::query(
            "INSERT INTO vessel_events (run_id, vessel_id, status, payload) VALUES (?, ?, ?, ?) RETURNING event_id"
        )
        .bind(run_id)
        .bind(vessel_id)
        .bind(status)
        .bind(payload_str)
        .fetch_one(&self.pool)
        .await?;

        let event_id: i64 = row.try_get("event_id")?;
        Ok(event_id)
    }

    /// Register a newly minted resumption token for a suspended Vessel step.
    pub async fn register_suspended_token(
        &self,
        token: &str,
        run_id: &str,
        vessel_id: &str,
    ) -> Result<(), CrucibleError> {
        sqlx::query(
            "INSERT INTO suspended_tokens (token, run_id, vessel_id) VALUES (?, ?, ?)"
        )
        .bind(token)
        .bind(run_id)
        .bind(vessel_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Atomically consume and invalidate an active resume token.
    ///
    /// Returns `(run_id, vessel_id)` if valid, or `CrucibleError::TokenNotFound` if invalid/already consumed.
    pub async fn consume_suspended_token(
        &self,
        token: &str,
    ) -> Result<(String, String), CrucibleError> {
        let row_opt = sqlx::query(
            "DELETE FROM suspended_tokens WHERE token = ? RETURNING run_id, vessel_id"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        match row_opt {
            Some(row) => {
                let run_id: String = row.try_get("run_id")?;
                let vessel_id: String = row.try_get("vessel_id")?;
                Ok((run_id, vessel_id))
            }
            None => Err(CrucibleError::TokenNotFound(token.to_string())),
        }
    }

    /// Retrieve the current run record snapshot for a given `run_id`.
    pub async fn get_run(&self, run_id: &str) -> Result<OpusRunRecord, CrucibleError> {
        let row_opt = sqlx::query(
            "SELECT run_id, opus_name, status, current_state, created_at, updated_at FROM opus_runs WHERE run_id = ?"
        )
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?;

        let row = row_opt.ok_or_else(|| CrucibleError::RunNotFound(run_id.to_string()))?;

        let state_str: String = row.try_get("current_state")?;
        let current_state: Value = serde_json::from_str(&state_str)?;

        Ok(OpusRunRecord {
            run_id: row.try_get("run_id")?,
            opus_name: row.try_get("opus_name")?,
            status: row.try_get("status")?,
            current_state,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    /// List the most recent workflow runs up to `limit`.
    pub async fn list_runs(&self, limit: usize) -> Result<Vec<OpusRunRecord>, CrucibleError> {
        let rows = sqlx::query(
            "SELECT run_id, opus_name, status, current_state, created_at, updated_at FROM opus_runs ORDER BY updated_at DESC LIMIT ?"
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            let state_str: String = row.try_get("current_state")?;
            let current_state: Value = serde_json::from_str(&state_str)?;

            records.push(OpusRunRecord {
                run_id: row.try_get("run_id")?,
                opus_name: row.try_get("opus_name")?,
                status: row.try_get("status")?,
                current_state,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            });
        }

        Ok(records)
    }

    /// Retrieve the complete chronological sequence of transition events for a given `run_id`.
    pub async fn get_vessel_events(
        &self,
        run_id: &str,
    ) -> Result<Vec<VesselEventRecord>, CrucibleError> {
        let rows = sqlx::query(
            "SELECT event_id, run_id, vessel_id, status, payload, recorded_at FROM vessel_events WHERE run_id = ? ORDER BY event_id ASC"
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            let payload_opt: Option<String> = row.try_get("payload")?;
            let payload = match payload_opt {
                Some(s) => Some(serde_json::from_str(&s)?),
                None => None,
            };

            events.push(VesselEventRecord {
                event_id: row.try_get("event_id")?,
                run_id: row.try_get("run_id")?,
                vessel_id: row.try_get("vessel_id")?,
                status: row.try_get("status")?,
                payload,
                recorded_at: row.try_get("recorded_at")?,
            });
        }

        Ok(events)
    }

    /// Inspect an active suspended token without consuming it.
    pub async fn get_suspended_token(
        &self,
        token: &str,
    ) -> Result<Option<SuspendedTokenRecord>, CrucibleError> {
        let row_opt = sqlx::query(
            "SELECT token, run_id, vessel_id, created_at FROM suspended_tokens WHERE token = ?"
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?;

        match row_opt {
            Some(row) => Ok(Some(SuspendedTokenRecord {
                token: row.try_get("token")?,
                run_id: row.try_get("run_id")?,
                vessel_id: row.try_get("vessel_id")?,
                created_at: row.try_get("created_at")?,
            })),
            None => Ok(None),
        }
    }
}
