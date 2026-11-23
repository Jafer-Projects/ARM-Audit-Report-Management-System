use rusqlite::{Connection, Result};
use std::path::Path;

pub struct Database {
    pub connection: Connection,
}

impl Database {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let connection = Connection::open(path)?;
        Self::configure(connection)
    }

    pub fn memory() -> Result<Self> {
        Self::configure(Connection::open_in_memory()?)
    }

    fn configure(connection: Connection) -> Result<Self> {
        connection.pragma_update(None, "foreign_keys", "ON")?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "synchronous", "FULL")?;
        let db = Self { connection };
        db.migrate()?;
        db.seed_demo()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migration (
                version INTEGER PRIMARY KEY,
                checksum TEXT NOT NULL,
                applied_at TEXT NOT NULL
            );"
        )?;
        let applied: i64 = self.connection.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migration",
            [],
            |row| row.get(0),
        )?;
        if applied < 1 {
            let tx = self.connection.unchecked_transaction()?;
            tx.execute_batch(
                "CREATE TABLE demo_firm (
                    id TEXT PRIMARY KEY NOT NULL,
                    name TEXT NOT NULL,
                    created_at TEXT NOT NULL
                );
                CREATE TABLE demo_actor (
                    id TEXT PRIMARY KEY NOT NULL,
                    display_name TEXT NOT NULL,
                    persona TEXT NOT NULL,
                    created_at TEXT NOT NULL
                );
                CREATE TABLE app_setting (
                    key TEXT PRIMARY KEY NOT NULL,
                    value TEXT NOT NULL
                );
                CREATE TABLE client (
                    id TEXT PRIMARY KEY NOT NULL,
                    firm_id TEXT NOT NULL REFERENCES demo_firm(id),
                    record_kind TEXT NOT NULL,
                    legal_name TEXT NOT NULL,
                    display_name TEXT NOT NULL DEFAULT '',
                    tin_raw TEXT NOT NULL DEFAULT '',
                    sector TEXT NOT NULL DEFAULT '',
                    contact_name TEXT NOT NULL DEFAULT '',
                    contact_email TEXT NOT NULL DEFAULT '',
                    contact_phone TEXT NOT NULL DEFAULT '',
                    address_text TEXT NOT NULL DEFAULT '',
                    internal_reference TEXT NOT NULL DEFAULT '',
                    notes TEXT NOT NULL DEFAULT '',
                    status TEXT NOT NULL CHECK(status IN ('Draft','Active','Archived')),
                    row_version INTEGER NOT NULL DEFAULT 1,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    created_by TEXT NOT NULL REFERENCES demo_actor(id),
                    updated_by TEXT NOT NULL REFERENCES demo_actor(id),
                    archived_at TEXT
                );
                CREATE INDEX client_firm_status ON client(firm_id, status);
                CREATE TABLE workspace_record (
                    id TEXT PRIMARY KEY NOT NULL,
                    firm_id TEXT NOT NULL REFERENCES demo_firm(id),
                    client_id TEXT REFERENCES client(id),
                    record_type TEXT NOT NULL CHECK(record_type IN
                      ('category','product','invoice','historical_record','period','staff','task','import_case','import_cost_input')),
                    title TEXT NOT NULL,
                    status TEXT NOT NULL,
                    reference TEXT NOT NULL DEFAULT '',
                    amount_text TEXT NOT NULL DEFAULT '',
                    notes TEXT NOT NULL DEFAULT '',
                    payload_json TEXT NOT NULL DEFAULT '{}',
                    row_version INTEGER NOT NULL DEFAULT 1,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    created_by TEXT NOT NULL REFERENCES demo_actor(id),
                    updated_by TEXT NOT NULL REFERENCES demo_actor(id)
                );
                CREATE INDEX workspace_record_type ON workspace_record(firm_id, record_type, updated_at);
                CREATE TABLE local_event (
                    id TEXT PRIMARY KEY NOT NULL,
                    firm_id TEXT NOT NULL REFERENCES demo_firm(id),
                    actor_id TEXT NOT NULL REFERENCES demo_actor(id),
                    event_type TEXT NOT NULL,
                    target_type TEXT NOT NULL,
                    target_id TEXT NOT NULL,
                    occurred_at_utc TEXT NOT NULL,
                    correlation_id TEXT NOT NULL,
                    payload_json TEXT NOT NULL,
                    schema_version INTEGER NOT NULL DEFAULT 1
                );
                CREATE INDEX local_event_time ON local_event(firm_id, occurred_at_utc DESC);"
            )?;
            tx.execute(
                "INSERT INTO schema_migration(version, checksum, applied_at) VALUES(1, ?1, ?2)",
                rusqlite::params!["arm-initial-schema-v1", now_utc()],
            )?;
            tx.commit()?;
        }
        if applied < 2 {
            let tx = self.connection.unchecked_transaction()?;
            tx.execute_batch(
                "CREATE TABLE local_note (
                    id TEXT PRIMARY KEY NOT NULL,
                    firm_id TEXT NOT NULL REFERENCES demo_firm(id),
                    client_id TEXT REFERENCES client(id),
                    actor_id TEXT NOT NULL REFERENCES demo_actor(id),
                    title TEXT NOT NULL CHECK(length(title) BETWEEN 1 AND 120),
                    body TEXT NOT NULL CHECK(length(body) <= 2000),
                    status TEXT NOT NULL CHECK(status='Local draft'),
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );
                CREATE INDEX local_note_firm_updated ON local_note(firm_id, updated_at);
"
            )?;
            tx.execute(
                "INSERT INTO schema_migration(version, checksum, applied_at) VALUES(2, ?1, ?2)",
                rusqlite::params!["arm-local-notes-v2", now_utc()],
            )?;
            tx.commit()?;
        }
        if applied > 3 {
            return Err(rusqlite::Error::InvalidQuery);
        }
        if applied < 3 {
            let tx = self.connection.unchecked_transaction()?;
            tx.execute_batch(
                "CREATE TRIGGER local_event_no_update BEFORE UPDATE ON local_event
                 BEGIN SELECT RAISE(ABORT, 'local_event is append-only'); END;
                 CREATE TRIGGER local_event_no_delete BEFORE DELETE ON local_event
                 BEGIN SELECT RAISE(ABORT, 'local_event is append-only'); END;"
            )?;
            tx.execute(
                "INSERT INTO schema_migration(version, checksum, applied_at) VALUES(3, ?1, ?2)",
                rusqlite::params!["arm-append-only-events-v3", now_utc()],
            )?;
            tx.commit()?;
        }
        for (version, expected_checksum) in [
            (1_i64, "arm-initial-schema-v1"),
            (2_i64, "arm-local-notes-v2"),
            (3_i64, "arm-append-only-events-v3"),
        ] {
            let actual: String = self.connection.query_row(
                "SELECT checksum FROM schema_migration WHERE version=?1",
                [version],
                |row| row.get(0),
            )?;
            if actual != expected_checksum {
                return Err(rusqlite::Error::InvalidQuery);
            }
        }
        Ok(())
    }

    fn seed_demo(&self) -> Result<()> {
        let firm_count: i64 = self.connection.query_row("SELECT COUNT(*) FROM demo_firm", [], |r| r.get(0))?;
        if firm_count == 0 {
            let firm = "00000000-0000-4000-8000-000000000001";
            self.connection.execute(
                "INSERT INTO demo_firm(id,name,created_at) VALUES(?1,?2,?3)",
                rusqlite::params![firm, "ARM Demonstration Firm", now_utc()],
            )?;
            for (id, name, persona) in [
                ("00000000-0000-4000-8000-000000000011", "Demo Firm Administrator", "administrator"),
                ("00000000-0000-4000-8000-000000000012", "Demo Engagement Manager", "manager"),
                ("00000000-0000-4000-8000-000000000013", "Demo Auditor", "auditor"),
                ("00000000-0000-4000-8000-000000000014", "Demo Read-only Reviewer", "reviewer"),
            ] {
                self.connection.execute(
                    "INSERT INTO demo_actor(id,display_name,persona,created_at) VALUES(?1,?2,?3,?4)",
                    rusqlite::params![id,name,persona,now_utc()],
                )?;
            }
            self.connection.execute(
                "INSERT INTO app_setting(key,value) VALUES('active_actor_id',?1)",
                rusqlite::params!["00000000-0000-4000-8000-000000000011"],
            )?;
            self.connection.execute(
                "INSERT INTO client(id,firm_id,record_kind,legal_name,display_name,status,created_at,updated_at,created_by,updated_by)
                 VALUES(?1,?2,'Company','Sample Trading PLC','Sample Trading','Active',?3,?3,?4,?4)",
                rusqlite::params![
                    "00000000-0000-4000-8000-000000000101",firm,now_utc(),
                    "00000000-0000-4000-8000-000000000011"
                ],
            )?;
            self.connection.execute(
                "INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json)
                 VALUES(?1,?2,?3,'demo_workspace_initialized','demo_workspace',?2,?4,?5,'{}')",
                rusqlite::params![
                    "00000000-0000-4000-8000-000000000201",firm,
                    "00000000-0000-4000-8000-000000000011",now_utc(),
                    "00000000-0000-4000-8000-000000000202"
                ],
            )?;
        }
        Ok(())
    }
}

pub fn now_utc() -> String {
    use time::format_description::well_known::Rfc3339;
    time::OffsetDateTime::now_utc().format(&Rfc3339).unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_is_idempotent_and_demo_is_seeded_once() {
        let db = Database::memory().expect("in-memory db");
        db.migrate().expect("second migration pass");
        db.seed_demo().expect("second seed pass");
        let clients: i64 = db.connection.query_row("SELECT COUNT(*) FROM client", [], |r| r.get(0)).unwrap();
        let actors: i64 = db.connection.query_row("SELECT COUNT(*) FROM demo_actor", [], |r| r.get(0)).unwrap();
        let migrations: i64 = db.connection.query_row("SELECT COUNT(*) FROM schema_migration", [], |r| r.get(0)).unwrap();
        assert_eq!(clients, 1);
        assert_eq!(actors, 4);
        assert_eq!(migrations, 3);
    }

    #[test]
    fn local_event_history_rejects_update_and_delete() {
        let db = Database::memory().expect("in-memory db");
        let update = db.connection.execute("UPDATE local_event SET event_type='tampered' WHERE event_type='demo_workspace_initialized'", []);
        let delete = db.connection.execute("DELETE FROM local_event WHERE event_type='demo_workspace_initialized'", []);
        assert!(update.is_err());
        assert!(delete.is_err());
    }

    #[test]
    fn local_notes_survive_database_reopen() {
        let directory = tempfile::tempdir().expect("temporary data directory");
        let path = directory.path().join("arm-test.sqlite3");
        {
            let db = Database::open(&path).expect("create test database");
            db.connection.execute(
                "INSERT INTO local_note(id,firm_id,client_id,actor_id,title,body,status,created_at,updated_at)
                 VALUES('note-1','00000000-0000-4000-8000-000000000001',NULL,'00000000-0000-4000-8000-000000000011','Restart test','Saved locally','Local draft','2026-11-23T00:00:00Z','2026-11-23T00:00:00Z')",
                [],
            ).expect("save note");
        }
        let reopened = Database::open(&path).expect("reopen test database");
        let title: String = reopened.connection.query_row("SELECT title FROM local_note WHERE id='note-1'", [], |r| r.get(0)).unwrap();
        assert_eq!(title, "Restart test");
    }


    #[test]
    fn migration_checksum_mismatch_is_rejected() {
        let db = Database::memory().expect("in-memory db");
        db.connection.execute("UPDATE schema_migration SET checksum='changed' WHERE version=2", []).expect("tamper test fixture");
        assert!(db.migrate().is_err());
    }

}
