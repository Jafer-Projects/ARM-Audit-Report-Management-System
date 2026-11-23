use crate::{db::{now_utc, Database}, AppState};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;
use uuid::Uuid;

const FIRM_ID: &str = "00000000-0000-4000-8000-000000000001";

#[derive(Serialize)]
pub struct WorkspaceInfo {
    firm_name: String,
    actor_name: String,
    persona: String,
    clients_count: i64,
    sync_state: &'static str,
}

#[derive(Serialize)]
pub struct Client {
    id: String,
    record_kind: String,
    legal_name: String,
    display_name: String,
    tin_raw: String,
    sector: String,
    contact_name: String,
    contact_email: String,
    contact_phone: String,
    address_text: String,
    internal_reference: String,
    notes: String,
    status: String,
    row_version: i64,
    updated_at: String,
}

#[derive(Deserialize)]
pub struct ClientInput {
    id: Option<String>,
    row_version: Option<i64>,
    record_kind: String,
    legal_name: String,
    display_name: Option<String>,
    tin_raw: Option<String>,
    sector: Option<String>,
    contact_name: Option<String>,
    contact_email: Option<String>,
    contact_phone: Option<String>,
    address_text: Option<String>,
    internal_reference: Option<String>,
    notes: Option<String>,
    duplicate_acknowledged: Option<bool>,
}

#[derive(Serialize)]
pub struct DuplicateCandidate { id: String, record_kind: String, legal_name: String, tin_raw: String, status: String }

#[derive(Serialize)]
pub struct WorkspaceRecord {
    id: String,
    client_id: Option<String>,
    record_type: String,
    title: String,
    status: String,
    reference: String,
    amount_text: String,
    notes: String,
    payload_json: String,
    row_version: i64,
    updated_at: String,
}

#[derive(Deserialize)]
pub struct RecordInput {
    id: Option<String>,
    row_version: Option<i64>,
    client_id: Option<String>,
    record_type: String,
    title: String,
    status: String,
    reference: Option<String>,
    amount_text: Option<String>,
    notes: Option<String>,
    payload_json: Option<String>,
}

#[derive(Serialize)]
pub struct EventItem {
    id: String,
    event_type: String,
    target_type: String,
    target_id: String,
    occurred_at_utc: String,
    actor_name: String,
    payload_json: String,
}

fn active_actor(db: &Database) -> Result<(String, String, String), String> {
    db.connection.query_row(
        "SELECT a.id,a.display_name,a.persona FROM demo_actor a
         JOIN app_setting s ON s.value=a.id WHERE s.key='active_actor_id'",
        [], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))
    ).map_err(|e| e.to_string())
}

fn ensure_can_mutate(db: &Database) -> Result<(String, String, String), String> {
    let actor = active_actor(db)?;
    if actor.2 == "reviewer" { return Err("The read-only demo reviewer cannot change records.".into()); }
    Ok(actor)
}

#[tauri::command]
pub fn initialize_workspace(state: State<'_, AppState>) -> Result<WorkspaceInfo, String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (_, actor_name, persona) = active_actor(&db)?;
    let clients_count = db.connection.query_row(
        "SELECT COUNT(*) FROM client WHERE firm_id=?1 AND status<>'Archived'", [FIRM_ID], |r| r.get(0)
    ).map_err(|e| e.to_string())?;
    Ok(WorkspaceInfo {
        firm_name: "ARM Demonstration Firm".into(), actor_name, persona, clients_count,
        sync_state: "Sync is not enabled in this prototype",
    })
}

#[tauri::command]
pub fn set_demo_persona(persona: String, state: State<'_, AppState>) -> Result<WorkspaceInfo, String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let actor_id: String = db.connection.query_row(
        "SELECT id FROM demo_actor WHERE persona=?1", [&persona], |r| r.get(0)
    ).map_err(|_| "Unknown demo persona".to_string())?;
    let tx = db.connection.unchecked_transaction().map_err(|e|e.to_string())?;
    tx.execute("UPDATE app_setting SET value=?1 WHERE key='active_actor_id'", [&actor_id]).map_err(|e|e.to_string())?;
    tx.execute(
        "INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json)
         VALUES(?1,?2,?3,'demo_persona_changed','demo_actor',?4,?5,?6,?7)",
        params![Uuid::new_v4().to_string(),FIRM_ID,actor_id,actor_id,now_utc(),Uuid::new_v4().to_string(),json!({"persona":persona}).to_string()]
    ).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())?;
    let (_, actor_name, current_persona) = active_actor(&db)?;
    let clients_count = db.connection.query_row("SELECT COUNT(*) FROM client WHERE firm_id=?1 AND status<>'Archived'", [FIRM_ID], |r| r.get(0)).map_err(|e| e.to_string())?;
    Ok(WorkspaceInfo { firm_name:"ARM Demonstration Firm".into(), actor_name, persona:current_persona, clients_count, sync_state:"Sync is not enabled in this prototype" })
}

#[tauri::command]
pub fn list_clients(state: State<'_, AppState>, include_archived: bool) -> Result<Vec<Client>, String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let sql = if include_archived {
        "SELECT id,record_kind,legal_name,display_name,tin_raw,sector,contact_name,contact_email,contact_phone,address_text,internal_reference,notes,status,row_version,updated_at FROM client WHERE firm_id=?1 ORDER BY updated_at DESC"
    } else {
        "SELECT id,record_kind,legal_name,display_name,tin_raw,sector,contact_name,contact_email,contact_phone,address_text,internal_reference,notes,status,row_version,updated_at FROM client WHERE firm_id=?1 AND status<>'Archived' ORDER BY updated_at DESC"
    };
    let mut stmt = db.connection.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([FIRM_ID], |r| Ok(Client {
        id:r.get(0)?,record_kind:r.get(1)?,legal_name:r.get(2)?,display_name:r.get(3)?,
        tin_raw:r.get(4)?,sector:r.get(5)?,contact_name:r.get(6)?,contact_email:r.get(7)?,
        contact_phone:r.get(8)?,address_text:r.get(9)?,internal_reference:r.get(10)?,
        notes:r.get(11)?,status:r.get(12)?,row_version:r.get(13)?,updated_at:r.get(14)?
    })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn find_client_duplicates(legal_name: String, tin_raw: String, exclude_id: Option<String>, state: State<'_, AppState>) -> Result<Vec<DuplicateCandidate>, String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let mut stmt = db.connection.prepare(
        "SELECT id,record_kind,legal_name,tin_raw,status FROM client
         WHERE firm_id=?1 AND id<>?2
           AND (lower(trim(legal_name))=lower(trim(?3)) OR (?4<>'' AND trim(tin_raw)=trim(?4)))
         ORDER BY updated_at DESC LIMIT 5"
    ).map_err(|e|e.to_string())?;
    let rows = stmt.query_map(params![FIRM_ID,exclude_id.unwrap_or_default(),legal_name.trim(),tin_raw.trim()], |r| Ok(DuplicateCandidate {
        id:r.get(0)?,record_kind:r.get(1)?,legal_name:r.get(2)?,tin_raw:r.get(3)?,status:r.get(4)?
    })).map_err(|e|e.to_string())?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())
}

#[tauri::command]
pub fn save_client(input: ClientInput, state: State<'_, AppState>) -> Result<Client, String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (actor_id, _, _) = ensure_can_mutate(&db)?;
    let legal_name = input.legal_name.trim();
    if legal_name.chars().count() < 2 || legal_name.chars().count() > 200 {
        return Err("Legal or full name must contain 2–200 characters.".into());
    }
    if !["Company","Individual","Other organization"].contains(&input.record_kind.as_str()) {
        return Err("Choose a supported record kind.".into());
    }
    if let Some(email) = input.contact_email.as_deref() {
        let email = email.trim();
        if !email.is_empty() && (!email.contains('@') || email.starts_with('@') || email.ends_with('@') || email.chars().any(char::is_whitespace)) {
            return Err("Enter a valid contact email address or leave it blank.".into());
        }
    }
    let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let mut duplicate_stmt = db.connection.prepare(
        "SELECT id,legal_name FROM client WHERE firm_id=?1 AND id<>?2
         AND (lower(trim(legal_name))=lower(trim(?3)) OR (?4<>'' AND trim(tin_raw)=trim(?4))) LIMIT 5"
    ).map_err(|e|e.to_string())?;
    let duplicate_rows = duplicate_stmt.query_map(params![FIRM_ID,id,legal_name,input.tin_raw.as_deref().unwrap_or("").trim()], |r| Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).map_err(|e|e.to_string())?;
    let duplicates = duplicate_rows.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    drop(duplicate_stmt);
    if !duplicates.is_empty() && input.duplicate_acknowledged != Some(true) {
        return Err("Possible duplicate client found. Compare the candidate and explicitly acknowledge before saving.".into());
    }
    let now = now_utc();
    let exists: Option<i64> = db.connection.query_row("SELECT row_version FROM client WHERE id=?1 AND firm_id=?2", params![id,FIRM_ID], |r| r.get(0)).optional().map_err(|e| e.to_string())?;
    let before_snapshot: Option<serde_json::Value> = if exists.is_some() {
        Some(db.connection.query_row(
            "SELECT record_kind,legal_name,display_name,tin_raw,sector,contact_name,contact_email,contact_phone,address_text,internal_reference,notes FROM client WHERE id=?1 AND firm_id=?2",
            params![id,FIRM_ID],
            |r| Ok(json!({
                "record_kind":r.get::<_,String>(0)?,"legal_name":r.get::<_,String>(1)?,"display_name":r.get::<_,String>(2)?,
                "tin_raw":r.get::<_,String>(3)?,"sector":r.get::<_,String>(4)?,"contact_name":r.get::<_,String>(5)?,
                "contact_email":r.get::<_,String>(6)?,"contact_phone":r.get::<_,String>(7)?,"address_text":r.get::<_,String>(8)?,
                "internal_reference":r.get::<_,String>(9)?,"notes":r.get::<_,String>(10)?
            }))
        ).map_err(|e|e.to_string())?)
    } else { None };
    let after_snapshot = json!({
        "record_kind":input.record_kind,"legal_name":legal_name,"display_name":input.display_name.as_deref().unwrap_or(""),
        "tin_raw":input.tin_raw.as_deref().unwrap_or("").trim(),"sector":input.sector.as_deref().unwrap_or(""),
        "contact_name":input.contact_name.as_deref().unwrap_or(""),"contact_email":input.contact_email.as_deref().unwrap_or("").trim(),
        "contact_phone":input.contact_phone.as_deref().unwrap_or(""),"address_text":input.address_text.as_deref().unwrap_or(""),
        "internal_reference":input.internal_reference.as_deref().unwrap_or(""),"notes":input.notes.as_deref().unwrap_or("")
    });
    let tx = db.connection.unchecked_transaction().map_err(|e| e.to_string())?;
    if let Some(current_version) = exists {
        if input.row_version != Some(current_version) { return Err("This client changed since it was opened. Reload or copy your changes before saving.".into()); }
        tx.execute(
            "UPDATE client SET record_kind=?1,legal_name=?2,display_name=?3,tin_raw=?4,sector=?5,contact_name=?6,contact_email=?7,contact_phone=?8,address_text=?9,internal_reference=?10,notes=?11,row_version=row_version+1,updated_at=?12,updated_by=?13 WHERE id=?14 AND firm_id=?15",
            params![input.record_kind,legal_name,input.display_name.unwrap_or_default(),input.tin_raw.unwrap_or_default().trim(),input.sector.unwrap_or_default(),input.contact_name.unwrap_or_default(),input.contact_email.unwrap_or_default().trim(),input.contact_phone.unwrap_or_default(),input.address_text.unwrap_or_default(),input.internal_reference.unwrap_or_default(),input.notes.unwrap_or_default(),now,actor_id,id,FIRM_ID]
        ).map_err(|e| e.to_string())?;
        tx.execute("INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json) VALUES(?1,?2,?3,'client_updated','client',?4,?5,?6,?7)",
            params![Uuid::new_v4().to_string(),FIRM_ID,actor_id,id,now,Uuid::new_v4().to_string(),json!({"legal_name":legal_name}).to_string()]).map_err(|e| e.to_string())?;
    } else {
        tx.execute(
            "INSERT INTO client(id,firm_id,record_kind,legal_name,display_name,tin_raw,sector,contact_name,contact_email,contact_phone,address_text,internal_reference,notes,status,row_version,created_at,updated_at,created_by,updated_by)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,'Draft',1,?14,?14,?15,?15)",
            params![id,FIRM_ID,input.record_kind,legal_name,input.display_name.unwrap_or_default(),input.tin_raw.unwrap_or_default().trim(),input.sector.unwrap_or_default(),input.contact_name.unwrap_or_default(),input.contact_email.unwrap_or_default().trim(),input.contact_phone.unwrap_or_default(),input.address_text.unwrap_or_default(),input.internal_reference.unwrap_or_default(),input.notes.unwrap_or_default(),now,actor_id]
        ).map_err(|e| e.to_string())?;
        tx.execute("INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json) VALUES(?1,?2,?3,'client_created','client',?4,?5,?6,'{}')",
            params![Uuid::new_v4().to_string(),FIRM_ID,actor_id,id,now,Uuid::new_v4().to_string()]).map_err(|e| e.to_string())?;
    }
    let change_correlation_id = Uuid::new_v4().to_string();
    tx.execute(
        "INSERT INTO client_change(id,client_id,firm_id,actor_id,action,changed_at,before_json,after_json,correlation_id) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![
            Uuid::new_v4().to_string(),id,FIRM_ID,actor_id,
            if exists.is_some() { "updated" } else { "created" },now,
            before_snapshot.unwrap_or_else(||json!({})).to_string(),after_snapshot.to_string(),change_correlation_id
        ]
    ).map_err(|e|e.to_string())?;
    for (candidate_id, candidate_name) in duplicates {
        tx.execute("INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json) VALUES(?1,?2,?3,'possible_duplicate_acknowledged','client',?4,?5,?6,?7)",
            params![Uuid::new_v4().to_string(),FIRM_ID,actor_id,id,now,Uuid::new_v4().to_string(),json!({"candidate_id":candidate_id,"candidate_name":candidate_name}).to_string()]
        ).map_err(|e|e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    let mut stmt = db.connection.prepare("SELECT id,record_kind,legal_name,display_name,tin_raw,sector,contact_name,contact_email,contact_phone,address_text,internal_reference,notes,status,row_version,updated_at FROM client WHERE id=?1").map_err(|e| e.to_string())?;
    stmt.query_row([id], |r| Ok(Client {
        id:r.get(0)?,record_kind:r.get(1)?,legal_name:r.get(2)?,display_name:r.get(3)?,tin_raw:r.get(4)?,sector:r.get(5)?,contact_name:r.get(6)?,contact_email:r.get(7)?,contact_phone:r.get(8)?,address_text:r.get(9)?,internal_reference:r.get(10)?,notes:r.get(11)?,status:r.get(12)?,row_version:r.get(13)?,updated_at:r.get(14)?
    })).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn archive_client(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (actor_id,_,_) = ensure_can_mutate(&db)?;
    let tx = db.connection.unchecked_transaction().map_err(|e| e.to_string())?;
    let changed = tx.execute("UPDATE client SET status='Archived',archived_at=?1,updated_at=?1,updated_by=?2,row_version=row_version+1 WHERE id=?3 AND firm_id=?4 AND status<>'Archived'",
        params![now_utc(),actor_id,id,FIRM_ID]).map_err(|e| e.to_string())?;
    if changed == 0 { return Err("Client was not found or is already archived.".into()); }
    tx.execute("INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json) VALUES(?1,?2,?3,'client_archived','client',?4,?5,?6,'{}')",
        params![Uuid::new_v4().to_string(),FIRM_ID,actor_id,id,now_utc(),Uuid::new_v4().to_string()]).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

fn allowed_record_type(value: &str) -> bool {
    matches!(value,"category"|"product"|"invoice"|"historical_record"|"period"|"staff"|"task"|"import_case"|"import_cost_input")
}

fn allowed_status(record_type: &str, status: &str) -> bool {
    match record_type {
        "category" | "product" => ["Draft","Active","Archived"].contains(&status),
        "invoice" => ["Draft","Submitted for review","Reviewed","Returned","Archived"].contains(&status),
        "historical_record" | "period" => ["Draft","Needs source","Ready for review","Reviewed","Locked"].contains(&status),
        "staff" => ["Draft","Active","Inactive"].contains(&status),
        "task" => ["Draft","Assigned","In progress","Blocked","Submitted","Returned","Completed","Cancelled"].contains(&status),
        "import_case" | "import_cost_input" => ["Discovery","Inputs captured","Needs review","Reviewed","Returned"].contains(&status),
        _ => false,
    }
}

#[tauri::command]
pub fn activate_client(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (actor_id,_,_) = ensure_can_mutate(&db)?;
    let tx = db.connection.unchecked_transaction().map_err(|e|e.to_string())?;
    let changed = tx.execute("UPDATE client SET status='Active',updated_at=?1,updated_by=?2,row_version=row_version+1 WHERE id=?3 AND firm_id=?4 AND status='Draft' AND length(trim(legal_name))>=2",
        params![now_utc(),actor_id,id,FIRM_ID]).map_err(|e|e.to_string())?;
    if changed == 0 { return Err("Only a valid draft client can be activated.".into()); }
    tx.execute("INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json) VALUES(?1,?2,?3,'client_activated','client',?4,?5,?6,'{}')",
        params![Uuid::new_v4().to_string(),FIRM_ID,actor_id,id,now_utc(),Uuid::new_v4().to_string()]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())
}

#[tauri::command]
pub fn restore_client(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (actor_id,_,_) = ensure_can_mutate(&db)?;
    let tx = db.connection.unchecked_transaction().map_err(|e|e.to_string())?;
    let changed = tx.execute("UPDATE client SET status='Active',archived_at=NULL,updated_at=?1,updated_by=?2,row_version=row_version+1 WHERE id=?3 AND firm_id=?4 AND status='Archived'",
        params![now_utc(),actor_id,id,FIRM_ID]).map_err(|e|e.to_string())?;
    if changed == 0 { return Err("Archived client was not found.".into()); }
    tx.execute("INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json) VALUES(?1,?2,?3,'client_restored','client',?4,?5,?6,'{}')",
        params![Uuid::new_v4().to_string(),FIRM_ID,actor_id,id,now_utc(),Uuid::new_v4().to_string()]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())
}

#[derive(Serialize)]
pub struct LocalNote {
    id: String, client_id: Option<String>, title: String, body: String, status: String, updated_at: String,
}

#[derive(Deserialize)]
pub struct LocalNoteInput {
    id: Option<String>, client_id: Option<String>, title: String, body: String,
}

#[tauri::command]
pub fn list_notes(state: State<'_, AppState>) -> Result<Vec<LocalNote>, String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let mut stmt = db.connection.prepare("SELECT id,client_id,title,body,status,updated_at FROM local_note WHERE firm_id=?1 ORDER BY updated_at DESC").map_err(|e|e.to_string())?;
    let rows = stmt.query_map([FIRM_ID], |r| Ok(LocalNote { id:r.get(0)?,client_id:r.get(1)?,title:r.get(2)?,body:r.get(3)?,status:r.get(4)?,updated_at:r.get(5)? })).map_err(|e|e.to_string())?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())
}

#[tauri::command]
pub fn save_note(input: LocalNoteInput, state: State<'_, AppState>) -> Result<LocalNote, String> {
    let title = input.title.trim();
    if title.is_empty() || title.chars().count() > 120 || input.body.chars().count() > 2000 {
        return Err("A note title (1–120 characters) and body (up to 2,000 characters) are required.".into());
    }
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (actor_id,_,_) = ensure_can_mutate(&db)?;
    if let Some(client_id) = input.client_id.as_deref() {
        let found: bool = db.connection.query_row("SELECT EXISTS(SELECT 1 FROM client WHERE id=?1 AND firm_id=?2)",params![client_id,FIRM_ID],|r|r.get(0)).map_err(|e|e.to_string())?;
        if !found { return Err("Selected client is not part of this demo workspace.".into()); }
    }
    let id = input.id.unwrap_or_else(||Uuid::new_v4().to_string());
    let now = now_utc();
    let tx = db.connection.unchecked_transaction().map_err(|e|e.to_string())?;
    tx.execute("INSERT INTO local_note(id,firm_id,client_id,actor_id,title,body,status,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,'Local draft',?7,?7)
        ON CONFLICT(id) DO UPDATE SET client_id=excluded.client_id,actor_id=excluded.actor_id,title=excluded.title,body=excluded.body,updated_at=excluded.updated_at WHERE local_note.firm_id=excluded.firm_id",
        params![id,FIRM_ID,input.client_id,actor_id,title,input.body,now]).map_err(|e|e.to_string())?;
    tx.execute("INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json) VALUES(?1,?2,?3,'local_note_saved','local_note',?4,?5,?6,?7)",
        params![Uuid::new_v4().to_string(),FIRM_ID,actor_id,id,now,Uuid::new_v4().to_string(),json!({"title":title}).to_string()]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())?;
    db.connection.query_row("SELECT id,client_id,title,body,status,updated_at FROM local_note WHERE id=?1 AND firm_id=?2",params![id,FIRM_ID],|r|Ok(LocalNote{id:r.get(0)?,client_id:r.get(1)?,title:r.get(2)?,body:r.get(3)?,status:r.get(4)?,updated_at:r.get(5)?})).map_err(|e|e.to_string())
}

#[tauri::command]
pub fn list_records(record_type: String, state: State<'_, AppState>) -> Result<Vec<WorkspaceRecord>, String> {
    if !allowed_record_type(&record_type) { return Err("Unsupported workspace record type.".into()); }
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let mut stmt = db.connection.prepare("SELECT id,client_id,record_type,title,status,reference,amount_text,notes,payload_json,row_version,updated_at FROM workspace_record WHERE firm_id=?1 AND record_type=?2 ORDER BY updated_at DESC").map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![FIRM_ID,record_type], |r| Ok(WorkspaceRecord {
        id:r.get(0)?,client_id:r.get(1)?,record_type:r.get(2)?,title:r.get(3)?,status:r.get(4)?,reference:r.get(5)?,amount_text:r.get(6)?,notes:r.get(7)?,payload_json:r.get(8)?,row_version:r.get(9)?,updated_at:r.get(10)?
    })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_record(input: RecordInput, state: State<'_, AppState>) -> Result<WorkspaceRecord, String> {
    if !allowed_record_type(&input.record_type) { return Err("Unsupported workspace record type.".into()); }
    if input.record_type == "import_cost_input" { return Err("Import-cost inputs remain locked until Gate 5-A discovery is approved. Record discovery questions in the import case instead.".into()); }
    if !allowed_status(&input.record_type, &input.status) { return Err("Unsupported status for this record type.".into()); }
    if let Some(payload) = input.payload_json.as_deref() { if payload.len() > 16_384 || serde_json::from_str::<serde_json::Value>(payload).is_err() { return Err("Record metadata must be valid JSON under 16 KB.".into()); } }
    if input.title.trim().is_empty() || input.title.chars().count() > 200 { return Err("A title of 1–200 characters is required.".into()); }
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (actor_id,_,_) = ensure_can_mutate(&db)?;
    if let Some(client_id) = input.client_id.as_deref() {
        let active: bool = db.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM client WHERE id=?1 AND firm_id=?2 AND status='Active')",
            params![client_id,FIRM_ID], |r| r.get(0)
        ).map_err(|e| e.to_string())?;
        if !active { return Err("Choose an active client in this demonstration firm.".into()); }
    } else if ["category","product","invoice","historical_record","period","task","import_case"].contains(&input.record_type.as_str()) {
        return Err("Select the client workspace for this record.".into());
    }
    let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let now = now_utc();
    let exists: Option<(i64,String)> = db.connection.query_row("SELECT row_version,record_type FROM workspace_record WHERE id=?1 AND firm_id=?2",params![id,FIRM_ID],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|e|e.to_string())?;
    let tx = db.connection.unchecked_transaction().map_err(|e|e.to_string())?;
    if let Some((version, existing_type)) = exists.as_ref() {
        if existing_type != &input.record_type { return Err("A record cannot be changed into a different record type.".into()); }
        if input.row_version != Some(*version) { return Err("This record changed since it was opened. Reload before saving.".into()); }
        tx.execute("UPDATE workspace_record SET client_id=?1,title=?2,status=?3,reference=?4,amount_text=?5,notes=?6,payload_json=?7,row_version=row_version+1,updated_at=?8,updated_by=?9 WHERE id=?10 AND firm_id=?11",
            params![input.client_id,input.title.trim(),input.status,input.reference.unwrap_or_default(),input.amount_text.unwrap_or_default(),input.notes.unwrap_or_default(),input.payload_json.unwrap_or_else(||"{}".into()),now,actor_id,id,FIRM_ID]).map_err(|e|e.to_string())?;
    } else {
        tx.execute("INSERT INTO workspace_record(id,firm_id,client_id,record_type,title,status,reference,amount_text,notes,payload_json,row_version,created_at,updated_at,created_by,updated_by) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,1,?11,?11,?12,?12)",
            params![id,FIRM_ID,input.client_id,input.record_type,input.title.trim(),input.status,input.reference.unwrap_or_default(),input.amount_text.unwrap_or_default(),input.notes.unwrap_or_default(),input.payload_json.unwrap_or_else(||"{}".into()),now,actor_id]).map_err(|e|e.to_string())?;
    }
    tx.execute("INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json) VALUES(?1,?2,?3,?4,'workspace_record',?5,?6,?7,?8)",
        params![Uuid::new_v4().to_string(),FIRM_ID,actor_id,if exists.is_some(){"workspace_record_updated"}else{"workspace_record_created"},id,now,Uuid::new_v4().to_string(),json!({"record_type":input.record_type,"status":input.status}).to_string()]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())?;
    let mut stmt = db.connection.prepare("SELECT id,client_id,record_type,title,status,reference,amount_text,notes,payload_json,row_version,updated_at FROM workspace_record WHERE id=?1").map_err(|e|e.to_string())?;
    stmt.query_row([id], |r| Ok(WorkspaceRecord {
        id:r.get(0)?,client_id:r.get(1)?,record_type:r.get(2)?,title:r.get(3)?,status:r.get(4)?,reference:r.get(5)?,amount_text:r.get(6)?,notes:r.get(7)?,payload_json:r.get(8)?,row_version:r.get(9)?,updated_at:r.get(10)?
    })).map_err(|e|e.to_string())
}

#[tauri::command]
pub fn list_events(state: State<'_, AppState>) -> Result<Vec<EventItem>, String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let mut stmt = db.connection.prepare("SELECT e.id,e.event_type,e.target_type,e.target_id,e.occurred_at_utc,a.display_name,e.payload_json FROM local_event e JOIN demo_actor a ON a.id=e.actor_id WHERE e.firm_id=?1 ORDER BY e.occurred_at_utc DESC LIMIT 250").map_err(|e|e.to_string())?;
    let rows = stmt.query_map([FIRM_ID], |r| Ok(EventItem { id:r.get(0)?,event_type:r.get(1)?,target_type:r.get(2)?,target_id:r.get(3)?,occurred_at_utc:r.get(4)?,actor_name:r.get(5)?,payload_json:r.get(6)? })).map_err(|e|e.to_string())?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())
}

#[tauri::command]
pub fn reset_demo_data(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (actor_id,_,_) = ensure_can_mutate(&db)?;
    let tx = db.connection.unchecked_transaction().map_err(|e|e.to_string())?;
    tx.execute("DELETE FROM workspace_record WHERE firm_id=?1",[FIRM_ID]).map_err(|e|e.to_string())?;
    tx.execute("DELETE FROM local_note WHERE firm_id=?1",[FIRM_ID]).map_err(|e|e.to_string())?;
    tx.execute("DELETE FROM client WHERE firm_id=?1",[FIRM_ID]).map_err(|e|e.to_string())?;
    tx.execute("INSERT INTO client(id,firm_id,record_kind,legal_name,display_name,status,created_at,updated_at,created_by,updated_by) VALUES(?1,?2,'Company','Sample Trading PLC','Sample Trading','Active',?3,?3,?4,?4)",
        params!["00000000-0000-4000-8000-000000000101",FIRM_ID,now_utc(),actor_id]).map_err(|e|e.to_string())?;
    tx.execute("INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json) VALUES(?1,?2,?3,'demo_data_reset','demo_workspace',?2,?4,?5,'{}')",
        params![Uuid::new_v4().to_string(),FIRM_ID,actor_id,now_utc(),Uuid::new_v4().to_string()]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())
}
