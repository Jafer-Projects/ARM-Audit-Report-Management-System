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
        if applied > 8 {
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
        if applied < 4 {
            let tx = self.connection.unchecked_transaction()?;
            tx.execute_batch(
                "CREATE TABLE client_change (
                    id TEXT PRIMARY KEY NOT NULL,
                    client_id TEXT NOT NULL REFERENCES client(id),
                    firm_id TEXT NOT NULL REFERENCES demo_firm(id),
                    actor_id TEXT NOT NULL REFERENCES demo_actor(id),
                    action TEXT NOT NULL CHECK(action IN ('created','updated')),
                    changed_at TEXT NOT NULL,
                    before_json TEXT NOT NULL,
                    after_json TEXT NOT NULL,
                    correlation_id TEXT NOT NULL
                );
                CREATE INDEX client_change_history ON client_change(firm_id,client_id,changed_at DESC);"
            )?;
            tx.execute(
                "INSERT INTO schema_migration(version, checksum, applied_at) VALUES(4, ?1, ?2)",
                rusqlite::params!["arm-client-change-history-v4", now_utc()],
            )?;
            tx.commit()?;
        }
        if applied < 5 {
            let tx = self.connection.unchecked_transaction()?;
            tx.execute_batch(
                "CREATE TABLE workspace_record_version (
                    id TEXT PRIMARY KEY NOT NULL,
                    firm_id TEXT NOT NULL REFERENCES demo_firm(id),
                    record_id TEXT NOT NULL REFERENCES workspace_record(id) ON DELETE CASCADE,
                    row_version INTEGER NOT NULL,
                    actor_id TEXT NOT NULL REFERENCES demo_actor(id),
                    changed_at TEXT NOT NULL,
                    snapshot_json TEXT NOT NULL,
                    correlation_id TEXT NOT NULL,
                    UNIQUE(record_id,row_version)
                );
                CREATE INDEX workspace_record_version_history ON workspace_record_version(firm_id,record_id,row_version DESC);"
            )?;
            tx.execute(
                "INSERT INTO schema_migration(version, checksum, applied_at) VALUES(5, ?1, ?2)",
                rusqlite::params!["arm-workspace-record-versions-v5", now_utc()],
            )?;
            tx.commit()?;
        }
        if applied < 6 {
            let tx = self.connection.unchecked_transaction()?;
            tx.execute_batch(
                "CREATE TABLE historical_import_batch (
                    id TEXT PRIMARY KEY NOT NULL,
                    firm_id TEXT NOT NULL REFERENCES demo_firm(id),
                    client_id TEXT NOT NULL REFERENCES client(id),
                    actor_id TEXT NOT NULL REFERENCES demo_actor(id),
                    source_filename TEXT NOT NULL,
                    source_checksum_sha256 TEXT NOT NULL CHECK(length(source_checksum_sha256)=64),
                    source_content TEXT NOT NULL,
                    mapping_json TEXT NOT NULL,
                    source_row_count INTEGER NOT NULL,
                    imported_row_count INTEGER NOT NULL,
                    imported_at TEXT NOT NULL,
                    correlation_id TEXT NOT NULL
                );
                CREATE INDEX historical_import_batch_client ON historical_import_batch(firm_id,client_id,imported_at DESC);"
            )?;
            tx.execute(
                "INSERT INTO schema_migration(version, checksum, applied_at) VALUES(6, ?1, ?2)",
                rusqlite::params!["arm-historical-import-batches-v6", now_utc()],
            )?;
            tx.commit()?;
        }
        if applied < 7 {
            let tx = self.connection.unchecked_transaction()?;
            tx.execute_batch(
                "CREATE TABLE task_review_note (
                    id TEXT PRIMARY KEY NOT NULL,
                    firm_id TEXT NOT NULL REFERENCES demo_firm(id),
                    task_id TEXT NOT NULL REFERENCES workspace_record(id),
                    reviewer_actor_id TEXT NOT NULL REFERENCES demo_actor(id),
                    category TEXT NOT NULL,
                    description TEXT NOT NULL,
                    affected_record_reference TEXT NOT NULL DEFAULT '',
                    required_response TEXT NOT NULL,
                    status TEXT NOT NULL CHECK(status IN ('Open','Responded','Cleared','Reopened')),
                    preparer_response TEXT NOT NULL DEFAULT '',
                    response_actor_id TEXT REFERENCES demo_actor(id),
                    responded_at TEXT,
                    disposition TEXT NOT NULL DEFAULT '',
                    disposition_actor_id TEXT REFERENCES demo_actor(id),
                    disposition_at TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    correlation_id TEXT NOT NULL
                );
                CREATE INDEX task_review_note_history ON task_review_note(firm_id,task_id,created_at);
                CREATE TRIGGER task_review_note_no_delete BEFORE DELETE ON task_review_note
                BEGIN SELECT RAISE(ABORT, 'task review notes cannot be deleted'); END;"
            )?;
            tx.execute(
                "INSERT INTO schema_migration(version, checksum, applied_at) VALUES(7, ?1, ?2)",
                rusqlite::params!["arm-task-review-notes-v7", now_utc()],
            )?;
            tx.commit()?;
        }
        if applied < 8 {
            let tx = self.connection.unchecked_transaction()?;
            tx.execute_batch(
                "CREATE TRIGGER client_change_no_update BEFORE UPDATE ON client_change
                 BEGIN SELECT RAISE(ABORT, 'client change history is immutable'); END;
                 CREATE TRIGGER client_change_no_delete BEFORE DELETE ON client_change
                 BEGIN SELECT RAISE(ABORT, 'client change history is immutable'); END;
                 CREATE TRIGGER workspace_record_version_no_update BEFORE UPDATE ON workspace_record_version
                 BEGIN SELECT RAISE(ABORT, 'workspace record versions are immutable'); END;
                 CREATE TRIGGER workspace_record_version_no_delete BEFORE DELETE ON workspace_record_version
                 BEGIN SELECT RAISE(ABORT, 'workspace record versions are immutable'); END;
                 CREATE TRIGGER historical_import_batch_no_update BEFORE UPDATE ON historical_import_batch
                 BEGIN SELECT RAISE(ABORT, 'historical import batches are immutable'); END;
                 CREATE TRIGGER historical_import_batch_no_delete BEFORE DELETE ON historical_import_batch
                 BEGIN SELECT RAISE(ABORT, 'historical import batches are immutable'); END;"
            )?;
            tx.execute(
                "INSERT INTO schema_migration(version, checksum, applied_at) VALUES(8, ?1, ?2)",
                rusqlite::params!["arm-immutable-history-artifacts-v8", now_utc()],
            )?;
            tx.commit()?;
        }
        for (version, expected_checksum) in [
            (1_i64, "arm-initial-schema-v1"),
            (2_i64, "arm-local-notes-v2"),
            (3_i64, "arm-append-only-events-v3"),
            (4_i64, "arm-client-change-history-v4"),
            (5_i64, "arm-workspace-record-versions-v5"),
            (6_i64, "arm-historical-import-batches-v6"),
            (7_i64, "arm-task-review-notes-v7"),
            (8_i64, "arm-immutable-history-artifacts-v8"),
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
            let tx = self.connection.unchecked_transaction()?;
            let firm = "00000000-0000-4000-8000-000000000001";
            tx.execute(
                "INSERT INTO demo_firm(id,name,created_at) VALUES(?1,?2,?3)",
                rusqlite::params![firm, "ARM Demonstration Firm", now_utc()],
            )?;
            for (id, name, persona) in [
                ("00000000-0000-4000-8000-000000000011", "Demo Firm Administrator", "administrator"),
                ("00000000-0000-4000-8000-000000000012", "Demo Engagement Manager", "manager"),
                ("00000000-0000-4000-8000-000000000013", "Demo Auditor", "auditor"),
                ("00000000-0000-4000-8000-000000000014", "Demo Read-only Reviewer", "reviewer"),
            ] {
                tx.execute(
                    "INSERT INTO demo_actor(id,display_name,persona,created_at) VALUES(?1,?2,?3,?4)",
                    rusqlite::params![id,name,persona,now_utc()],
                )?;
            }
            tx.execute(
                "INSERT INTO app_setting(key,value) VALUES('active_actor_id',?1)",
                rusqlite::params!["00000000-0000-4000-8000-000000000011"],
            )?;
            tx.execute(
                "INSERT INTO client(id,firm_id,record_kind,legal_name,display_name,status,created_at,updated_at,created_by,updated_by)
                 VALUES(?1,?2,'Company','Sample Trading PLC','Sample Trading','Active',?3,?3,?4,?4)",
                rusqlite::params![
                    "00000000-0000-4000-8000-000000000101",firm,now_utc(),
                    "00000000-0000-4000-8000-000000000011"
                ],
            )?;
            for (id, title, staff_ref, job_title, discipline) in [
                ("00000000-0000-4000-8000-000000000301","Demo Firm Administrator","DEMO-ADM","Firm administrator","Administration"),
                ("00000000-0000-4000-8000-000000000302","Demo Engagement Manager","DEMO-MGR","Engagement manager","Audit management"),
                ("00000000-0000-4000-8000-000000000303","Demo Auditor","DEMO-AUD","Auditor","Audit preparation"),
            ] {
                let created_at = now_utc();
                let payload = serde_json::json!({"staff_reference":staff_ref,"job_title":job_title,"discipline":discipline});
                tx.execute(
                    "INSERT INTO workspace_record(id,firm_id,client_id,record_type,title,status,reference,payload_json,row_version,created_at,updated_at,created_by,updated_by)
                     VALUES(?1,?2,NULL,'staff',?3,'Active',?4,?5,1,?6,?6,?7,?7)",
                    rusqlite::params![id,firm,title,staff_ref,payload.to_string(),created_at,"00000000-0000-4000-8000-000000000011"],
                )?;
                tx.execute(
                    "INSERT INTO workspace_record_version(id,firm_id,record_id,row_version,actor_id,changed_at,snapshot_json,correlation_id)
                     VALUES(?1,?2,?3,1,?4,?5,?6,?7)",
                    rusqlite::params![uuid::Uuid::new_v4().to_string(),firm,id,"00000000-0000-4000-8000-000000000011",created_at,
                        serde_json::json!({"record_type":"staff","title":title,"status":"Active","reference":staff_ref,"payload":payload}).to_string(),
                        uuid::Uuid::new_v4().to_string()],
                )?;
            }
            tx.execute(
                "INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json)
                 VALUES(?1,?2,?3,'demo_workspace_initialized','demo_workspace',?2,?4,?5,'{}')",
                rusqlite::params![
                    "00000000-0000-4000-8000-000000000201",firm,
                    "00000000-0000-4000-8000-000000000011",now_utc(),
                    "00000000-0000-4000-8000-000000000202"
                ],
            )?;
            tx.commit()?;
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
        let staff: i64 = db.connection.query_row("SELECT COUNT(*) FROM workspace_record WHERE record_type='staff'", [], |r| r.get(0)).unwrap();
        assert_eq!(clients, 1);
        assert_eq!(actors, 4);
        assert_eq!(migrations, 8);
        assert_eq!(staff, 3);
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


    #[test]
    fn client_change_history_stores_before_and_after_snapshots() {
        let db = Database::memory().expect("in-memory db");
        db.connection.execute(
            "INSERT INTO client_change(id,client_id,firm_id,actor_id,action,changed_at,before_json,after_json,correlation_id)
             VALUES('change-1','00000000-0000-4000-8000-000000000101','00000000-0000-4000-8000-000000000001','00000000-0000-4000-8000-000000000011','updated','2026-11-23T00:00:00Z','{\"legal_name\":\"Before\"}','{\"legal_name\":\"After\"}','corr-1')",
            [],
        ).expect("insert client change");
        let (before, after): (String, String) = db.connection.query_row(
            "SELECT before_json,after_json FROM client_change WHERE id='change-1'", [],
            |r| Ok((r.get(0)?,r.get(1)?))
        ).unwrap();
        assert!(before.contains("Before"));
        assert!(after.contains("After"));
    }


    #[test]
    fn workspace_record_versions_preserve_each_snapshot() {
        let db = Database::memory().expect("in-memory db");
        let firm = "00000000-0000-4000-8000-000000000001";
        let actor = "00000000-0000-4000-8000-000000000011";
        let client = "00000000-0000-4000-8000-000000000101";
        db.connection.execute(
            "INSERT INTO workspace_record(id,firm_id,client_id,record_type,title,status,row_version,created_at,updated_at,created_by,updated_by)
             VALUES('record-1',?1,?2,'invoice','Invoice A','Draft',1,'2026-11-23T00:00:00Z','2026-11-23T00:00:00Z',?3,?3)",
            rusqlite::params![firm,client,actor],
        ).expect("create record");
        db.connection.execute(
            "INSERT INTO workspace_record_version(id,firm_id,record_id,row_version,actor_id,changed_at,snapshot_json,correlation_id)
             VALUES('version-1',?1,'record-1',1,?2,'2026-11-23T00:00:00Z','{\"title\":\"Invoice A\"}','corr-1')",
            rusqlite::params![firm,actor],
        ).expect("save first snapshot");
        db.connection.execute("UPDATE workspace_record SET title='Invoice B',row_version=2 WHERE id='record-1'", []).expect("update record");
        db.connection.execute(
            "INSERT INTO workspace_record_version(id,firm_id,record_id,row_version,actor_id,changed_at,snapshot_json,correlation_id)
             VALUES('version-2',?1,'record-1',2,?2,'2026-11-23T00:01:00Z','{\"title\":\"Invoice B\"}','corr-2')",
            rusqlite::params![firm,actor],
        ).expect("save second snapshot");
        let count: i64 = db.connection.query_row("SELECT COUNT(*) FROM workspace_record_version WHERE record_id='record-1'", [], |r| r.get(0)).unwrap();
        let first: String = db.connection.query_row("SELECT snapshot_json FROM workspace_record_version WHERE row_version=1", [], |r| r.get(0)).unwrap();
        let second: String = db.connection.query_row("SELECT snapshot_json FROM workspace_record_version WHERE row_version=2", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 2);
        assert!(first.contains("Invoice A"));
        assert!(second.contains("Invoice B"));
    }


    #[test]
    fn task_review_notes_cannot_be_deleted() {
        let db = Database::memory().expect("in-memory db");
        let firm = "00000000-0000-4000-8000-000000000001";
        let actor = "00000000-0000-4000-8000-000000000012";
        let client = "00000000-0000-4000-8000-000000000101";
        db.connection.execute(
            "INSERT INTO workspace_record(id,firm_id,client_id,record_type,title,status,created_at,updated_at,created_by,updated_by)
             VALUES('task-review-test',?1,?2,'task','Test task','Submitted','now','now',?3,?3)",
            rusqlite::params![firm,client,actor],
        ).expect("create task");
        db.connection.execute(
            "INSERT INTO task_review_note(id,firm_id,task_id,reviewer_actor_id,category,description,required_response,status,created_at,updated_at,correlation_id)
             VALUES('note-test',?1,'task-review-test',?2,'Missing evidence','Evidence is missing','Provide the source reference','Open','now','now','corr')",
            rusqlite::params![firm,actor],
        ).expect("create review note");
        assert!(db.connection.execute("DELETE FROM task_review_note WHERE id='note-test'", []).is_err());
    }

    #[test]
    fn historical_import_batch_retains_source_content_and_mapping() {
        let db = Database::memory().expect("in-memory db");
        let source = "Date,Description,Amount\n2026-01-01,Sample purchase,100.00";
        db.connection.execute(
            "INSERT INTO historical_import_batch(id,firm_id,client_id,actor_id,source_filename,source_checksum_sha256,source_content,mapping_json,source_row_count,imported_row_count,imported_at,correlation_id)
             VALUES('batch-test','00000000-0000-4000-8000-000000000001','00000000-0000-4000-8000-000000000101','00000000-0000-4000-8000-000000000011','sample.csv',?1,?2,'{\"description\":\"Description\",\"amount_text\":\"Amount\"}',1,1,'now','corr')",
            rusqlite::params!["a".repeat(64),source],
        ).expect("create import batch");
        let stored: String = db.connection.query_row("SELECT source_content FROM historical_import_batch WHERE id='batch-test'", [], |r|r.get(0)).unwrap();
        assert_eq!(stored, source);
    }

}
