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
        if applied > 12 {
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
        if applied < 9 {
            let tx = self.connection.unchecked_transaction()?;
            tx.execute_batch(
                "CREATE TRIGGER task_review_note_content_immutable BEFORE UPDATE ON task_review_note
                 WHEN NEW.firm_id<>OLD.firm_id OR NEW.task_id<>OLD.task_id OR NEW.reviewer_actor_id<>OLD.reviewer_actor_id
                   OR NEW.category<>OLD.category OR NEW.description<>OLD.description
                   OR NEW.affected_record_reference<>OLD.affected_record_reference
                   OR NEW.required_response<>OLD.required_response OR NEW.created_at<>OLD.created_at
                   OR NEW.correlation_id<>OLD.correlation_id
                 BEGIN SELECT RAISE(ABORT, 'task review note content is immutable'); END;
                 CREATE TRIGGER task_review_note_status_transition BEFORE UPDATE OF status ON task_review_note
                 WHEN NOT ((OLD.status IN ('Open','Reopened') AND NEW.status='Responded')
                    OR (OLD.status='Responded' AND NEW.status IN ('Cleared','Reopened')))
                 BEGIN SELECT RAISE(ABORT, 'invalid task review note transition'); END;"
            )?;
            tx.execute(
                "INSERT INTO schema_migration(version, checksum, applied_at) VALUES(9, ?1, ?2)",
                rusqlite::params!["arm-review-note-transition-guards-v9", now_utc()],
            )?;
            tx.commit()?;
        }

        if applied < 10 {
            let tx = self.connection.unchecked_transaction()?;
            tx.execute_batch(
                "CREATE TABLE source_document (
                    id TEXT PRIMARY KEY NOT NULL,
                    firm_id TEXT NOT NULL REFERENCES demo_firm(id),
                    client_id TEXT NOT NULL REFERENCES client(id),
                    record_id TEXT NOT NULL REFERENCES workspace_record(id),
                    actor_id TEXT NOT NULL REFERENCES demo_actor(id),
                    file_name TEXT NOT NULL,
                    media_type TEXT NOT NULL CHECK(media_type IN ('application/pdf','image/jpeg','image/png')),
                    size_bytes INTEGER NOT NULL CHECK(size_bytes > 0 AND size_bytes <= 5242880),
                    sha256 TEXT NOT NULL CHECK(length(sha256)=64),
                    content BLOB NOT NULL,
                    created_at TEXT NOT NULL,
                    correlation_id TEXT NOT NULL,
                    UNIQUE(firm_id,record_id,sha256)
                );
                CREATE INDEX source_document_record ON source_document(firm_id,client_id,record_id,created_at DESC);
                CREATE TRIGGER source_document_no_update BEFORE UPDATE ON source_document
                BEGIN SELECT RAISE(ABORT, 'source document evidence is immutable'); END;
                CREATE TRIGGER source_document_no_delete BEFORE DELETE ON source_document
                BEGIN SELECT RAISE(ABORT, 'source document evidence is immutable'); END;"
            )?;
            tx.execute(
                "INSERT INTO schema_migration(version, checksum, applied_at) VALUES(10, ?1, ?2)",
                rusqlite::params!["arm-source-documents-v10", now_utc()],
            )?;
            tx.commit()?;
        }

        if applied < 11 {
            let tx = self.connection.unchecked_transaction()?;
            tx.execute_batch(
                "CREATE TABLE workspace_notification (
                    id TEXT PRIMARY KEY NOT NULL,
                    firm_id TEXT NOT NULL REFERENCES demo_firm(id),
                    recipient_actor_id TEXT NOT NULL REFERENCES demo_actor(id),
                    client_id TEXT NOT NULL REFERENCES client(id),
                    record_id TEXT NOT NULL REFERENCES workspace_record(id),
                    record_type TEXT NOT NULL,
                    event_type TEXT NOT NULL,
                    title TEXT NOT NULL,
                    body TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    read_at TEXT,
                    idempotency_key TEXT NOT NULL UNIQUE
                );
                CREATE INDEX workspace_notification_recipient ON workspace_notification(firm_id,recipient_actor_id,created_at DESC);
                CREATE TRIGGER workspace_notification_content_immutable BEFORE UPDATE ON workspace_notification
                WHEN NEW.id<>OLD.id OR NEW.firm_id<>OLD.firm_id OR NEW.recipient_actor_id<>OLD.recipient_actor_id
                  OR NEW.client_id<>OLD.client_id OR NEW.record_id<>OLD.record_id OR NEW.record_type<>OLD.record_type
                  OR NEW.event_type<>OLD.event_type OR NEW.title<>OLD.title OR NEW.body<>OLD.body
                  OR NEW.created_at<>OLD.created_at OR NEW.idempotency_key<>OLD.idempotency_key
                BEGIN SELECT RAISE(ABORT, 'notification content is immutable'); END;
                CREATE TRIGGER workspace_notification_read_once BEFORE UPDATE OF read_at ON workspace_notification
                WHEN OLD.read_at IS NOT NULL OR NEW.read_at IS NULL
                BEGIN SELECT RAISE(ABORT, 'notification read state can only be set once'); END;
                CREATE TRIGGER workspace_notification_no_delete BEFORE DELETE ON workspace_notification
                BEGIN SELECT RAISE(ABORT, 'notification history is immutable'); END;"
            )?;
            tx.execute(
                "INSERT INTO schema_migration(version, checksum, applied_at) VALUES(11, ?1, ?2)",
                rusqlite::params!["arm-workspace-notifications-v11", now_utc()],
            )?;
            tx.commit()?;
        }

        if applied < 12 {
            let tx = self.connection.unchecked_transaction()?;
            tx.execute_batch(
                "CREATE TABLE sync_outbox (
                    event_id TEXT PRIMARY KEY NOT NULL,
                    idempotency_key TEXT NOT NULL UNIQUE,
                    aggregate_type TEXT NOT NULL CHECK(aggregate_type='prototype.workspace_record'),
                    aggregate_id TEXT NOT NULL,
                    base_version INTEGER NOT NULL CHECK(base_version>=0),
                    event_type TEXT NOT NULL CHECK(event_type='aggregate.upsert'),
                    schema_version INTEGER NOT NULL DEFAULT 1,
                    occurred_at TEXT NOT NULL,
                    correlation_id TEXT NOT NULL,
                    payload_json TEXT NOT NULL CHECK(json_valid(payload_json)),
                    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending','accepted','conflict','rejected')),
                    attempts INTEGER NOT NULL DEFAULT 0,
                    server_version INTEGER,
                    response_json TEXT NOT NULL DEFAULT '{}',
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );
                CREATE INDEX sync_outbox_pending ON sync_outbox(status,created_at);
                CREATE TABLE cloud_sync_state (
                    firm_id TEXT PRIMARY KEY NOT NULL REFERENCES demo_firm(id),
                    cloud_tenant_id TEXT NOT NULL,
                    pull_cursor TEXT NOT NULL DEFAULT '0',
                    last_success_at TEXT,
                    updated_at TEXT NOT NULL
                );
                CREATE TABLE cloud_sync_aggregate_state (
                    aggregate_id TEXT PRIMARY KEY NOT NULL,
                    server_version INTEGER NOT NULL CHECK(server_version>=0),
                    updated_at TEXT NOT NULL
                );
                CREATE TABLE cloud_sync_inbox (
                    received_seq INTEGER PRIMARY KEY NOT NULL,
                    event_id TEXT NOT NULL UNIQUE,
                    aggregate_id TEXT NOT NULL,
                    outcome TEXT NOT NULL CHECK(outcome IN ('accepted','conflict','rejected')),
                    resulting_version INTEGER,
                    payload_json TEXT NOT NULL CHECK(json_valid(payload_json)),
                    received_at TEXT NOT NULL
                );
                CREATE TRIGGER workspace_record_outbox_enqueue
                AFTER INSERT ON local_event
                WHEN NEW.target_type='workspace_record'
                BEGIN
                  INSERT OR IGNORE INTO sync_outbox(
                    event_id,idempotency_key,aggregate_type,aggregate_id,base_version,event_type,schema_version,
                    occurred_at,correlation_id,payload_json,created_at,updated_at
                  )
                  SELECT NEW.id,NEW.id,'prototype.workspace_record',r.id,
                    COALESCE(
                      (SELECT MAX(o.base_version)+1 FROM sync_outbox o WHERE o.aggregate_id=r.id AND o.status='pending'),
                      (SELECT a.server_version FROM cloud_sync_aggregate_state a WHERE a.aggregate_id=r.id),
                      0
                    ),
                    'aggregate.upsert',1,NEW.occurred_at_utc,NEW.correlation_id,
                    json_object('record',json_object(
                      'id',r.id,'client_id',r.client_id,'record_type',r.record_type,'title',r.title,
                      'status',r.status,'reference',r.reference,'amount_text',r.amount_text,'notes',r.notes,
                      'payload_json',r.payload_json,'row_version',r.row_version,'updated_at',r.updated_at
                    )),NEW.occurred_at_utc,NEW.occurred_at_utc
                  FROM workspace_record r WHERE r.id=NEW.target_id AND r.firm_id=NEW.firm_id;
                END;

                INSERT OR IGNORE INTO sync_outbox(
                  event_id,idempotency_key,aggregate_type,aggregate_id,base_version,event_type,schema_version,
                  occurred_at,correlation_id,payload_json,created_at,updated_at
                )
                SELECT
                  lower(hex(randomblob(4)))||'-'||lower(hex(randomblob(2)))||'-4'||substr(lower(hex(randomblob(2))),2)||'-a'||substr(lower(hex(randomblob(2))),2)||'-'||lower(hex(randomblob(6))),
                  lower(hex(randomblob(4)))||'-'||lower(hex(randomblob(2)))||'-4'||substr(lower(hex(randomblob(2))),2)||'-a'||substr(lower(hex(randomblob(2))),2)||'-'||lower(hex(randomblob(6))),
                  'prototype.workspace_record',r.id,COALESCE(a.server_version,0),'aggregate.upsert',1,r.updated_at,
                  lower(hex(randomblob(4)))||'-'||lower(hex(randomblob(2)))||'-4'||substr(lower(hex(randomblob(2))),2)||'-a'||substr(lower(hex(randomblob(2))),2)||'-'||lower(hex(randomblob(6))),
                  json_object('record',json_object('id',r.id,'client_id',r.client_id,'record_type',r.record_type,'title',r.title,'status',r.status,'reference',r.reference,'amount_text',r.amount_text,'notes',r.notes,'payload_json',r.payload_json,'row_version',r.row_version,'updated_at',r.updated_at)),
                  r.updated_at,r.updated_at
                FROM workspace_record r LEFT JOIN cloud_sync_aggregate_state a ON a.aggregate_id=r.id;
                CREATE TRIGGER sync_outbox_response_update
                BEFORE UPDATE ON sync_outbox
                WHEN NEW.event_id<>OLD.event_id OR NEW.idempotency_key<>OLD.idempotency_key
                  OR NEW.aggregate_id<>OLD.aggregate_id OR NEW.payload_json<>OLD.payload_json
                  OR NEW.base_version<>OLD.base_version OR NEW.created_at<>OLD.created_at
                BEGIN SELECT RAISE(ABORT,'sync outbox event envelope is immutable'); END;
                CREATE TRIGGER sync_inbox_no_delete BEFORE DELETE ON cloud_sync_inbox
                BEGIN SELECT RAISE(ABORT,'cloud sync inbox history is immutable'); END;"
            )?;
            tx.execute(
                "INSERT INTO schema_migration(version,checksum,applied_at) VALUES(12,?1,?2)",
                rusqlite::params!["arm-cloud-sync-outbox-v12",now_utc()],
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
            (9_i64, "arm-review-note-transition-guards-v9"),
            (10_i64, "arm-source-documents-v10"),
            (11_i64, "arm-workspace-notifications-v11"),
            (12_i64, "arm-cloud-sync-outbox-v12"),
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
                ("00000000-0000-4000-8000-000000000304","Demo Read-only Reviewer","DEMO-REV","Read-only reviewer","Read-only task observation"),
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
        assert_eq!(migrations, 12);
        assert_eq!(staff, 4);
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
        assert!(db.connection.execute("UPDATE workspace_record_version SET snapshot_json='{}' WHERE id='version-1'", []).is_err());
        assert!(db.connection.execute("DELETE FROM workspace_record_version WHERE id='version-1'", []).is_err());
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
        assert!(db.connection.execute("UPDATE task_review_note SET description='rewritten' WHERE id='note-test'", []).is_err());
        assert!(db.connection.execute("UPDATE task_review_note SET status='Cleared' WHERE id='note-test'", []).is_err());
        db.connection.execute("UPDATE task_review_note SET status='Responded',preparer_response='Evidence attached',response_actor_id='00000000-0000-4000-8000-000000000013',responded_at='now',updated_at='now' WHERE id='note-test'", []).expect("valid response transition");
        db.connection.execute("UPDATE task_review_note SET status='Cleared',disposition='Reviewed and accepted',disposition_actor_id='00000000-0000-4000-8000-000000000012',disposition_at='now',updated_at='now' WHERE id='note-test'", []).expect("valid disposition transition");
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
        assert!(db.connection.execute("UPDATE historical_import_batch SET source_content='changed' WHERE id='batch-test'", []).is_err());
        assert!(db.connection.execute("DELETE FROM historical_import_batch WHERE id='batch-test'", []).is_err());
    }


    #[test]
    fn client_change_history_is_immutable() {
        let db = Database::memory().expect("in-memory db");
        db.connection.execute(
            "INSERT INTO client_change(id,client_id,firm_id,actor_id,action,changed_at,before_json,after_json,correlation_id)
             VALUES('change-test','00000000-0000-4000-8000-000000000101','00000000-0000-4000-8000-000000000001','00000000-0000-4000-8000-000000000011','created','now','{}','{\"legal_name\":\"Sample\"}','corr')",
            [],
        ).expect("create client change");
        assert!(db.connection.execute("UPDATE client_change SET after_json='{}' WHERE id='change-test'", []).is_err());
        assert!(db.connection.execute("DELETE FROM client_change WHERE id='change-test'", []).is_err());
    }


    #[test]
    fn uncommitted_workspace_write_rolls_back_when_transaction_is_dropped() {
        let db = Database::memory().expect("in-memory db");
        {
            let tx = db.connection.unchecked_transaction().expect("begin transaction");
            tx.execute(
                "INSERT INTO local_note(id,firm_id,client_id,actor_id,title,body,status,created_at,updated_at)
                 VALUES('rollback-note','00000000-0000-4000-8000-000000000001',NULL,'00000000-0000-4000-8000-000000000011','Rollback','Must not persist','Local draft','now','now')",
                [],
            ).expect("write inside transaction");
        }
        let count: i64 = db.connection.query_row("SELECT COUNT(*) FROM local_note WHERE id='rollback-note'", [], |row|row.get(0)).unwrap();
        assert_eq!(count,0);
    }

    #[test]
    fn foreign_key_constraints_reject_orphaned_workspace_history() {
        let db = Database::memory().expect("in-memory db");
        let result = db.connection.execute(
            "INSERT INTO workspace_record_version(id,firm_id,record_id,row_version,actor_id,changed_at,snapshot_json,correlation_id)
             VALUES('orphan-version','00000000-0000-4000-8000-000000000001','missing-record',1,'00000000-0000-4000-8000-000000000011','now','{}','corr')",
            [],
        );
        assert!(result.is_err());
    }

    #[test]
    fn local_event_actor_and_scope_are_retained_for_material_mutations() {
        let db = Database::memory().expect("in-memory db");
        let (actor,firm,target): (String,String,String) = db.connection.query_row(
            "SELECT actor_id,firm_id,target_type FROM local_event WHERE event_type='demo_workspace_initialized' LIMIT 1",
            [], |row|Ok((row.get(0)?,row.get(1)?,row.get(2)?))
        ).expect("seed event");
        assert!(!actor.is_empty());
        assert!(!firm.is_empty());
        assert_eq!(target,"workspace");
    }


    #[test]
    fn workspace_record_events_create_versioned_durable_cloud_outbox_entries() {
        let db=Database::memory().expect("in-memory db");
        let record="00000000-0000-4000-8000-000000000301";
        db.connection.execute(
            "INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json)
             VALUES('10000000-0000-4000-8000-000000000001',?1,?2,'workspace_record_updated','workspace_record',?3,'2026-11-24T00:00:00Z','corr-one','{}')",
            rusqlite::params!["00000000-0000-4000-8000-000000000001","00000000-0000-4000-8000-000000000011",record]
        ).expect("append first workspace event");
        let first:(i64,String)=db.connection.query_row("SELECT base_version,payload_json FROM sync_outbox WHERE event_id='10000000-0000-4000-8000-000000000001'",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
        assert_eq!(first.0,0);
        assert!(first.1.contains("Demo Firm Administrator"));
        db.connection.execute("UPDATE workspace_record SET row_version=2,title='Updated administrator' WHERE id=?1",[record]).unwrap();
        db.connection.execute(
            "INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json)
             VALUES('10000000-0000-4000-8000-000000000002',?1,?2,'workspace_record_updated','workspace_record',?3,'2026-11-24T00:01:00Z','corr-two','{}')",
            rusqlite::params!["00000000-0000-4000-8000-000000000001","00000000-0000-4000-8000-000000000011",record]
        ).expect("append second workspace event");
        let second:(i64,String)=db.connection.query_row("SELECT base_version,payload_json FROM sync_outbox WHERE event_id='10000000-0000-4000-8000-000000000002'",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
        assert_eq!(second.0,1);
        assert!(second.1.contains("Updated administrator"));
    }

}
