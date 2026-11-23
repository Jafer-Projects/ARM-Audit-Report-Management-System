use crate::{db::{now_utc, Database}, AppState};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
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
    duplicate_acknowledged: Option<bool>,
    duplicate_acknowledgement_reason: Option<String>,
}

#[derive(Serialize)]
pub struct InvoiceDuplicateCandidate { id: String, title: String, status: String, invoice_number: String, invoice_date: String }

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

fn actor_has_client_access(db: &Database, client_id: &str, actor_name: &str, persona: &str) -> Result<bool, String> {
    if ["administrator","manager"].contains(&persona) { return Ok(true); }
    let field = match persona {
        "auditor" => "assignee",
        "reviewer" => "reviewer",
        _ => return Ok(false),
    };
    let mut stmt = db.connection.prepare(
        "SELECT payload_json FROM workspace_record WHERE firm_id=?1 AND client_id=?2 AND record_type='task'"
    ).map_err(|e|e.to_string())?;
    let rows = stmt.query_map(params![FIRM_ID,client_id], |r|r.get::<_,String>(0)).map_err(|e|e.to_string())?;
    for row in rows {
        let raw = row.map_err(|e|e.to_string())?;
        let payload: serde_json::Value = serde_json::from_str(&raw).unwrap_or_else(|_|json!({}));
        if payload.get(field).and_then(|v|v.as_str()) == Some(actor_name) { return Ok(true); }
    }
    Ok(false)
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
    let mut clients = rows.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    let (_, actor_name, persona) = active_actor(&db)?;
    if persona == "auditor" || persona == "reviewer" {
        let field = if persona == "auditor" { "assignee" } else { "reviewer" };
        let mut task_stmt = db.connection.prepare(
            "SELECT client_id,payload_json FROM workspace_record WHERE firm_id=?1 AND record_type='task' AND client_id IS NOT NULL"
        ).map_err(|e|e.to_string())?;
        let task_rows = task_stmt.query_map([FIRM_ID], |r| Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).map_err(|e|e.to_string())?;
        let mut assigned_clients = std::collections::HashSet::new();
        for task_row in task_rows {
            let (client_id,raw) = task_row.map_err(|e|e.to_string())?;
            let payload: serde_json::Value = serde_json::from_str(&raw).unwrap_or_else(|_|json!({}));
            if payload.get(field).and_then(|v|v.as_str()) == Some(actor_name.as_str()) {
                assigned_clients.insert(client_id);
            }
        }
        clients.retain(|client| assigned_clients.contains(&client.id));
    }
    Ok(clients)
}

#[tauri::command]
pub fn find_client_duplicates(legal_name: String, tin_raw: String, exclude_id: Option<String>, state: State<'_, AppState>) -> Result<Vec<DuplicateCandidate>, String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (_,_,persona) = active_actor(&db)?;
    if !["administrator","manager"].contains(&persona.as_str()) { return Err("Client duplicate search is restricted to the demo administrator and manager.".into()); }
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
    let (actor_id, _, persona) = ensure_can_mutate(&db)?;
    if !["administrator","manager"].contains(&persona.as_str()) { return Err("Only the demo administrator or manager can create or edit client profiles.".into()); }
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

#[derive(Serialize)]
pub struct ClientChange {
    id: String, action: String, changed_at: String, actor_name: String, before_json: String, after_json: String,
}

#[tauri::command]
pub fn list_client_changes(client_id: String, state: State<'_, AppState>) -> Result<Vec<ClientChange>, String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (_,actor_name,persona) = active_actor(&db)?;
    if !actor_has_client_access(&db,&client_id,&actor_name,&persona)? {
        return Err("This client is not assigned to your demo persona.".into());
    }
    let client_exists: bool = db.connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM client WHERE id=?1 AND firm_id=?2)",
        params![client_id,FIRM_ID], |r| r.get(0)
    ).map_err(|e|e.to_string())?;
    if !client_exists { return Err("Client not found in this demonstration firm.".into()); }
    let mut stmt = db.connection.prepare(
        "SELECT c.id,c.action,c.changed_at,a.display_name,c.before_json,c.after_json
         FROM client_change c JOIN demo_actor a ON a.id=c.actor_id
         WHERE c.firm_id=?1 AND c.client_id=?2 ORDER BY c.changed_at DESC"
    ).map_err(|e|e.to_string())?;
    let rows = stmt.query_map(params![FIRM_ID,client_id], |r| Ok(ClientChange {
        id:r.get(0)?,action:r.get(1)?,changed_at:r.get(2)?,actor_name:r.get(3)?,before_json:r.get(4)?,after_json:r.get(5)?
    })).map_err(|e|e.to_string())?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())
}

#[tauri::command]
pub fn archive_client(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (actor_id,_,persona) = ensure_can_mutate(&db)?;
    if !["administrator","manager"].contains(&persona.as_str()) { return Err("Only the demo administrator or manager can change client lifecycle state.".into()); }
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
        "invoice" => ["Draft","Submitted for review","Returned","Archived"].contains(&status),
        "historical_record" | "period" => ["Draft","Needs source","Ready for review"].contains(&status),
        "staff" => ["Draft","Active","Inactive"].contains(&status),
        "task" => ["Draft","Assigned","In progress","Blocked","Submitted","Returned","Completed","Cancelled"].contains(&status),
        "import_case" | "import_cost_input" => ["Discovery","Needs practitioner input","Ready for workshop","Deferred"].contains(&status),
        _ => false,
    }
}

fn valid_record_transition(record_type: &str, previous: Option<&str>, next: &str) -> bool {
    match (record_type, previous, next) {
        ("category" | "product", None, "Draft") => true,
        ("category" | "product", Some("Draft"), "Draft" | "Active") => true,
        ("category" | "product", Some("Active"), "Active" | "Archived") => true,
        ("invoice", None, "Draft") => true,
        ("invoice", Some("Draft"), "Draft" | "Submitted for review" | "Archived") => true,
        ("invoice", Some("Submitted for review"), "Submitted for review" | "Returned") => true,
        ("invoice", Some("Returned"), "Returned" | "Draft" | "Archived") => true,
        ("historical_record" | "period", None, "Draft") => true,
        ("historical_record" | "period", Some("Draft"), "Draft" | "Needs source" | "Ready for review") => true,
        ("historical_record" | "period", Some("Needs source"), "Needs source" | "Ready for review") => true,
        ("historical_record" | "period", Some("Ready for review"), "Ready for review") => true,
        ("staff", None, "Draft") => true,
        ("staff", Some("Draft"), "Draft" | "Active") => true,
        ("staff", Some("Active"), "Active" | "Inactive") => true,
        ("task", None, "Draft") => true,
        ("task", Some("Draft"), "Draft" | "Assigned" | "Cancelled") => true,
        ("task", Some("Assigned"), "Assigned" | "In progress" | "Cancelled") => true,
        ("task", Some("In progress"), "In progress" | "Blocked" | "Submitted" | "Cancelled") => true,
        ("task", Some("Blocked"), "Blocked" | "In progress" | "Cancelled") => true,
        ("task", Some("Submitted"), "Submitted" | "Returned" | "Completed" | "Cancelled") => true,
        ("task", Some("Returned"), "Returned" | "In progress" | "Cancelled") => true,
        ("task", Some("Completed"), "Completed") | ("task", Some("Cancelled"), "Cancelled") => true,
        ("import_case", None, "Discovery") => true,
        ("import_case", Some("Discovery"), "Discovery" | "Needs practitioner input" | "Ready for workshop" | "Deferred") => true,
        ("import_case", Some("Needs practitioner input"), "Needs practitioner input" | "Ready for workshop" | "Deferred") => true,
        ("import_case", Some("Ready for workshop"), "Ready for workshop" | "Deferred") => true,
        ("import_case", Some("Deferred"), "Deferred" | "Discovery") => true,
        _ => false,
    }
}

#[tauri::command]
pub fn activate_client(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (actor_id,_,persona) = ensure_can_mutate(&db)?;
    if !["administrator","manager"].contains(&persona.as_str()) { return Err("Only the demo administrator or manager can change client lifecycle state.".into()); }
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
    let (actor_id,_,persona) = ensure_can_mutate(&db)?;
    if !["administrator","manager"].contains(&persona.as_str()) { return Err("Only the demo administrator or manager can change client lifecycle state.".into()); }
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
    let (actor_id,actor_name,persona) = active_actor(&db)?;
    let mut stmt = db.connection.prepare("SELECT id,client_id,title,body,status,updated_at FROM local_note WHERE firm_id=?1 AND actor_id=?2 ORDER BY updated_at DESC").map_err(|e|e.to_string())?;
    let rows = stmt.query_map(params![FIRM_ID,actor_id], |r| Ok(LocalNote { id:r.get(0)?,client_id:r.get(1)?,title:r.get(2)?,body:r.get(3)?,status:r.get(4)?,updated_at:r.get(5)? })).map_err(|e|e.to_string())?;
    let mut notes = rows.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    notes.retain(|note| note.client_id.as_deref().map_or(true, |client_id| actor_has_client_access(&db,client_id,&actor_name,&persona).unwrap_or(false)));
    Ok(notes)
}

#[tauri::command]
pub fn save_note(input: LocalNoteInput, state: State<'_, AppState>) -> Result<LocalNote, String> {
    let title = input.title.trim();
    if title.is_empty() || title.chars().count() > 120 || input.body.chars().count() > 2000 {
        return Err("A note title (1–120 characters) and body (up to 2,000 characters) are required.".into());
    }
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (actor_id,actor_name,persona) = ensure_can_mutate(&db)?;
    if let Some(client_id) = input.client_id.as_deref() {
        let active: bool = db.connection.query_row("SELECT EXISTS(SELECT 1 FROM client WHERE id=?1 AND firm_id=?2 AND status='Active')",params![client_id,FIRM_ID],|r|r.get(0)).map_err(|e|e.to_string())?;
        if !active { return Err("Selected client is not active in this demo workspace.".into()); }
        if !actor_has_client_access(&db,client_id,&actor_name,&persona)? { return Err("This client workspace is not assigned to your demo persona.".into()); }
    }
    let id = input.id.unwrap_or_else(||Uuid::new_v4().to_string());
    let now = now_utc();
    let tx = db.connection.unchecked_transaction().map_err(|e|e.to_string())?;
    tx.execute("INSERT INTO local_note(id,firm_id,client_id,actor_id,title,body,status,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,'Local draft',?7,?7)
        ON CONFLICT(id) DO UPDATE SET client_id=excluded.client_id,actor_id=excluded.actor_id,title=excluded.title,body=excluded.body,updated_at=excluded.updated_at WHERE local_note.firm_id=excluded.firm_id AND local_note.actor_id=excluded.actor_id",
        params![id,FIRM_ID,input.client_id,actor_id,title,input.body,now]).map_err(|e|e.to_string())?;
    tx.execute("INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json) VALUES(?1,?2,?3,'local_note_saved','local_note',?4,?5,?6,?7)",
        params![Uuid::new_v4().to_string(),FIRM_ID,actor_id,id,now,Uuid::new_v4().to_string(),json!({"title":title}).to_string()]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())?;
    db.connection.query_row("SELECT id,client_id,title,body,status,updated_at FROM local_note WHERE id=?1 AND firm_id=?2",params![id,FIRM_ID],|r|Ok(LocalNote{id:r.get(0)?,client_id:r.get(1)?,title:r.get(2)?,body:r.get(3)?,status:r.get(4)?,updated_at:r.get(5)?})).map_err(|e|e.to_string())
}

#[tauri::command]
pub fn find_invoice_duplicates(client_id: String, invoice_number: String, exclude_id: Option<String>, state: State<'_, AppState>) -> Result<Vec<InvoiceDuplicateCandidate>, String> {
    let number = invoice_number.trim();
    if number.is_empty() { return Ok(Vec::new()); }
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let active: bool = db.connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM client WHERE id=?1 AND firm_id=?2 AND status='Active')",
        params![client_id,FIRM_ID], |r|r.get(0)
    ).map_err(|e|e.to_string())?;
    if !active { return Err("Select an active client before checking invoice duplicates.".into()); }
    let (_,actor_name,persona) = active_actor(&db)?;
    if !actor_has_client_access(&db,&client_id,&actor_name,&persona)? { return Err("This client is not assigned to your demo persona.".into()); }
    let mut stmt = db.connection.prepare(
        "SELECT id,title,status,payload_json FROM workspace_record
         WHERE firm_id=?1 AND client_id=?2 AND record_type='invoice' AND id<>?3
         ORDER BY updated_at DESC LIMIT 500"
    ).map_err(|e|e.to_string())?;
    let rows = stmt.query_map(params![FIRM_ID,client_id,exclude_id.unwrap_or_default()], |r| {
        Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?))
    }).map_err(|e|e.to_string())?;
    let mut candidates = Vec::new();
    for row in rows {
        let (id,title,status,raw) = row.map_err(|e|e.to_string())?;
        let payload: serde_json::Value = serde_json::from_str(&raw).unwrap_or_else(|_|json!({}));
        let candidate_number = payload.get("invoice_number").and_then(|v|v.as_str()).unwrap_or("").trim();
        if candidate_number.eq_ignore_ascii_case(number) {
            candidates.push(InvoiceDuplicateCandidate {
                id,title,status,invoice_number:candidate_number.to_string(),
                invoice_date:payload.get("invoice_date").and_then(|v|v.as_str()).unwrap_or("").to_string()
            });
        }
    }
    Ok(candidates)
}

#[derive(Deserialize)]
pub struct HistoricalImportRow {
    source_row: usize,
    transaction_date: String,
    description: String,
    amount_text: String,
    currency: String,
    supplier: String,
    supplier_tin: String,
    vat_amount: String,
    invoice_reference: String,
}

#[derive(Deserialize)]
pub struct HistoricalImportInput {
    client_id: String,
    source_filename: String,
    source_content: String,
    mapping_json: String,
    rows: Vec<HistoricalImportRow>,
}

#[derive(Serialize)]
pub struct HistoricalImportResult {
    batch_id: String,
    source_checksum_sha256: String,
    source_row_count: usize,
    imported_row_count: usize,
}

#[derive(Serialize)]
pub struct HistoricalImportBatch {
    id: String,
    source_filename: String,
    source_checksum_sha256: String,
    source_row_count: i64,
    imported_row_count: i64,
    imported_at: String,
}

#[derive(Serialize)]
pub struct TaskReviewNote {
    id: String,
    task_id: String,
    category: String,
    description: String,
    affected_record_reference: String,
    required_response: String,
    status: String,
    preparer_response: String,
    reviewer_name: String,
    response_actor_name: String,
    disposition: String,
    disposition_actor_name: String,
    created_at: String,
    updated_at: String,
}

#[tauri::command]
pub fn list_task_review_notes(task_id: String, state: State<'_, AppState>) -> Result<Vec<TaskReviewNote>, String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (_, actor_name, persona) = active_actor(&db)?;
    let raw: String = db.connection.query_row(
        "SELECT payload_json FROM workspace_record WHERE id=?1 AND firm_id=?2 AND record_type='task'",
        params![task_id,FIRM_ID], |r|r.get(0)
    ).map_err(|_|"Task not found in this demonstration firm.".to_string())?;
    let payload: serde_json::Value = serde_json::from_str(&raw).unwrap_or_else(|_|json!({}));
    let assignee = payload.get("assignee").and_then(|v|v.as_str()).unwrap_or("");
    let reviewer = payload.get("reviewer").and_then(|v|v.as_str()).unwrap_or("");
    if (persona == "auditor" && assignee != actor_name.as_str())
        || (persona == "reviewer" && reviewer != actor_name.as_str()) {
        return Err("This task is not assigned to your demo persona.".into());
    }
    let mut stmt = db.connection.prepare(
        "SELECT n.id,n.task_id,n.category,n.description,n.affected_record_reference,n.required_response,n.status,n.preparer_response,
                reviewer.display_name,COALESCE(response_actor.display_name,''),n.disposition,COALESCE(disposition_actor.display_name,''),
                n.created_at,n.updated_at
         FROM task_review_note n
         JOIN demo_actor reviewer ON reviewer.id=n.reviewer_actor_id
         LEFT JOIN demo_actor response_actor ON response_actor.id=n.response_actor_id
         LEFT JOIN demo_actor disposition_actor ON disposition_actor.id=n.disposition_actor_id
         WHERE n.firm_id=?1 AND n.task_id=?2 ORDER BY n.created_at"
    ).map_err(|e|e.to_string())?;
    let rows = stmt.query_map(params![FIRM_ID,task_id], |r| Ok(TaskReviewNote {
        id:r.get(0)?,task_id:r.get(1)?,category:r.get(2)?,description:r.get(3)?,
        affected_record_reference:r.get(4)?,required_response:r.get(5)?,status:r.get(6)?,
        preparer_response:r.get(7)?,reviewer_name:r.get(8)?,response_actor_name:r.get(9)?,
        disposition:r.get(10)?,disposition_actor_name:r.get(11)?,created_at:r.get(12)?,updated_at:r.get(13)?
    })).map_err(|e|e.to_string())?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())
}

#[tauri::command]
pub fn add_task_review_note(task_id: String, category: String, description: String, affected_record_reference: String, required_response: String, state: State<'_, AppState>) -> Result<TaskReviewNote, String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (actor_id, actor_name, persona) = ensure_can_mutate(&db)?;
    if !["administrator","manager"].contains(&persona.as_str()) {
        return Err("Only the separately assigned demo reviewer can create a review note.".into());
    }
    if !["Missing evidence","Incorrect data","Unclear explanation","Calculation query","Other"].contains(&category.as_str()) {
        return Err("Choose a supported review-note category.".into());
    }
    if description.trim().chars().count() < 5 || description.chars().count() > 2000
        || required_response.trim().chars().count() < 5 || required_response.chars().count() > 1000 {
        return Err("Review description and required response must be completed within the stated limits.".into());
    }
    let (status, updated_by, raw): (String,String,String) = db.connection.query_row(
        "SELECT status,updated_by,payload_json FROM workspace_record WHERE id=?1 AND firm_id=?2 AND record_type='task'",
        params![task_id,FIRM_ID], |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))
    ).map_err(|_|"Task not found in this demonstration firm.".to_string())?;
    if status != "Submitted" { return Err("Review notes can only be opened on a task submitted for review.".into()); }
    if updated_by == actor_id { return Err("The preparer cannot review their own task.".into()); }
    let payload: serde_json::Value = serde_json::from_str(&raw).unwrap_or_else(|_|json!({}));
    if payload.get("reviewer").and_then(|v|v.as_str()) != Some(actor_name.as_str()) {
        return Err("You are not the reviewer explicitly assigned to this task.".into());
    }
    let id = Uuid::new_v4().to_string();
    let now = now_utc();
    let correlation_id = Uuid::new_v4().to_string();
    let tx = db.connection.unchecked_transaction().map_err(|e|e.to_string())?;
    tx.execute(
        "INSERT INTO task_review_note(id,firm_id,task_id,reviewer_actor_id,category,description,affected_record_reference,required_response,status,created_at,updated_at,correlation_id)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8,'Open',?9,?9,?10)",
        params![id,FIRM_ID,task_id,actor_id,category,description.trim(),affected_record_reference.trim(),required_response.trim(),now,correlation_id]
    ).map_err(|e|e.to_string())?;
    tx.execute(
        "INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json)
         VALUES(?1,?2,?3,'task_review_note_opened','task_review_note',?4,?5,?6,?7)",
        params![Uuid::new_v4().to_string(),FIRM_ID,actor_id,id,now,correlation_id,json!({"task_id":task_id,"category":category}).to_string()]
    ).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())?;
    let mut stmt = db.connection.prepare(
        "SELECT n.id,n.task_id,n.category,n.description,n.affected_record_reference,n.required_response,n.status,n.preparer_response,
                reviewer.display_name,COALESCE(response_actor.display_name,''),n.disposition,COALESCE(disposition_actor.display_name,''),
                n.created_at,n.updated_at
         FROM task_review_note n JOIN demo_actor reviewer ON reviewer.id=n.reviewer_actor_id
         LEFT JOIN demo_actor response_actor ON response_actor.id=n.response_actor_id
         LEFT JOIN demo_actor disposition_actor ON disposition_actor.id=n.disposition_actor_id WHERE n.id=?1"
    ).map_err(|e|e.to_string())?;
    stmt.query_row([id], |r| Ok(TaskReviewNote {
        id:r.get(0)?,task_id:r.get(1)?,category:r.get(2)?,description:r.get(3)?,
        affected_record_reference:r.get(4)?,required_response:r.get(5)?,status:r.get(6)?,
        preparer_response:r.get(7)?,reviewer_name:r.get(8)?,response_actor_name:r.get(9)?,
        disposition:r.get(10)?,disposition_actor_name:r.get(11)?,created_at:r.get(12)?,updated_at:r.get(13)?
    })).map_err(|e|e.to_string())
}

#[tauri::command]
pub fn respond_task_review_note(note_id: String, response: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (actor_id, actor_name, persona) = ensure_can_mutate(&db)?;
    if persona != "auditor" { return Err("Only the assigned preparer responds to a review note.".into()); }
    if response.trim().chars().count() < 3 || response.chars().count() > 2000 {
        return Err("A response of 3–2,000 characters is required.".into());
    }
    let (task_id,status,raw): (String,String,String) = db.connection.query_row(
        "SELECT n.task_id,n.status,t.payload_json FROM task_review_note n JOIN workspace_record t ON t.id=n.task_id
         WHERE n.id=?1 AND n.firm_id=?2 AND t.firm_id=?2 AND t.record_type='task'",
        params![note_id,FIRM_ID], |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))
    ).map_err(|_|"Review note not found.".to_string())?;
    if !["Open","Reopened"].contains(&status.as_str()) { return Err("Only an open or reopened review note can receive a response.".into()); }
    let payload: serde_json::Value = serde_json::from_str(&raw).unwrap_or_else(|_|json!({}));
    if payload.get("assignee").and_then(|v|v.as_str()) != Some(actor_name.as_str()) {
        return Err("Only the assigned preparer can respond to this review note.".into());
    }
    let now = now_utc();
    let correlation_id = Uuid::new_v4().to_string();
    let tx = db.connection.unchecked_transaction().map_err(|e|e.to_string())?;
    tx.execute("UPDATE task_review_note SET status='Responded',preparer_response=?1,response_actor_id=?2,responded_at=?3,updated_at=?3 WHERE id=?4 AND firm_id=?5",
        params![response.trim(),actor_id,now,note_id,FIRM_ID]).map_err(|e|e.to_string())?;
    tx.execute("INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json) VALUES(?1,?2,?3,'task_review_note_responded','task_review_note',?4,?5,?6,?7)",
        params![Uuid::new_v4().to_string(),FIRM_ID,actor_id,note_id,now,correlation_id,json!({"task_id":task_id}).to_string()]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn disposition_task_review_note(note_id: String, decision: String, reason: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (actor_id, actor_name, persona) = ensure_can_mutate(&db)?;
    if !["administrator","manager"].contains(&persona.as_str()) { return Err("Only the assigned demo reviewer can dispose of a review note.".into()); }
    if !["Cleared","Reopened"].contains(&decision.as_str()) || reason.trim().chars().count() < 3 || reason.chars().count() > 1000 {
        return Err("Choose Clear or Reopen and record a 3–1,000 character reason.".into());
    }
    let (task_id,status,raw): (String,String,String) = db.connection.query_row(
        "SELECT n.task_id,n.status,t.payload_json FROM task_review_note n JOIN workspace_record t ON t.id=n.task_id
         WHERE n.id=?1 AND n.firm_id=?2 AND t.firm_id=?2 AND t.record_type='task'",
        params![note_id,FIRM_ID], |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))
    ).map_err(|_|"Review note not found.".to_string())?;
    if status != "Responded" { return Err("A review note must have a preparer response before it can be cleared or reopened.".into()); }
    let payload: serde_json::Value = serde_json::from_str(&raw).unwrap_or_else(|_|json!({}));
    if payload.get("reviewer").and_then(|v|v.as_str()) != Some(actor_name.as_str()) {
        return Err("You are not the reviewer assigned to this task.".into());
    }
    let now = now_utc();
    let correlation_id = Uuid::new_v4().to_string();
    let tx = db.connection.unchecked_transaction().map_err(|e|e.to_string())?;
    tx.execute("UPDATE task_review_note SET status=?1,disposition=?2,disposition_actor_id=?3,disposition_at=?4,updated_at=?4 WHERE id=?5 AND firm_id=?6",
        params![decision,reason.trim(),actor_id,now,note_id,FIRM_ID]).map_err(|e|e.to_string())?;
    tx.execute("INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json) VALUES(?1,?2,?3,?4,'task_review_note',?5,?6,?7,?8)",
        params![Uuid::new_v4().to_string(),FIRM_ID,actor_id,if decision=="Cleared"{"task_review_note_cleared"}else{"task_review_note_reopened"},note_id,now,correlation_id,json!({"task_id":task_id,"reason":reason.trim()}).to_string()]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn import_historical_batch(input: HistoricalImportInput, state: State<'_, AppState>) -> Result<HistoricalImportResult, String> {
    if input.source_content.len() > 5 * 1024 * 1024 {
        return Err("CSV source must be no larger than 5 MB.".into());
    }
    if input.source_filename.trim().is_empty() || input.source_filename.len() > 255
        || input.source_filename.contains('/') || input.source_filename.contains('\\')
        || input.source_filename.contains("..") || !input.source_filename.to_ascii_lowercase().ends_with(".csv") {
        return Err("Choose a CSV filename without path components.".into());
    }
    if input.rows.is_empty() || input.rows.len() > 1000 {
        return Err("A batch must contain between 1 and 1,000 data rows.".into());
    }
    if input.mapping_json.len() > 16_384 || serde_json::from_str::<serde_json::Value>(&input.mapping_json).is_err() {
        return Err("Column mapping must be valid JSON under 16 KB.".into());
    }
    let mut seen_rows = std::collections::HashSet::new();
    for row in &input.rows {
        if row.source_row < 2 || row.source_row > 100_001 || !seen_rows.insert(row.source_row) {
            return Err("CSV source row numbers must be unique and start at row 2.".into());
        }
        if row.description.trim().is_empty() || row.amount_text.trim().is_empty() {
            return Err(format!("CSV row {} needs both a description and source amount.", row.source_row));
        }
    }
    let digest = Sha256::digest(input.source_content.as_bytes());
    let checksum = digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (actor_id,actor_name,persona) = ensure_can_mutate(&db)?;
    let active: bool = db.connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM client WHERE id=?1 AND firm_id=?2 AND status='Active')",
        params![input.client_id,FIRM_ID], |r|r.get(0)
    ).map_err(|e|e.to_string())?;
    if !active { return Err("Choose an active client in this demonstration firm.".into()); }
    if !actor_has_client_access(&db,&input.client_id,&actor_name,&persona)? {
        return Err("This client workspace is not assigned to your demo persona.".into());
    }

    let batch_id = Uuid::new_v4().to_string();
    let correlation_id = Uuid::new_v4().to_string();
    let imported_at = now_utc();
    let tx = db.connection.unchecked_transaction().map_err(|e|e.to_string())?;
    tx.execute(
        "INSERT INTO historical_import_batch(id,firm_id,client_id,actor_id,source_filename,source_checksum_sha256,source_content,mapping_json,source_row_count,imported_row_count,imported_at,correlation_id)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?9,?10,?11)",
        params![batch_id,FIRM_ID,input.client_id,actor_id,input.source_filename,checksum,input.source_content,input.mapping_json,input.rows.len() as i64,imported_at,correlation_id]
    ).map_err(|e|e.to_string())?;

    for row in &input.rows {
        let record_id = Uuid::new_v4().to_string();
        let mut payload = json!({
            "historical_class":"Imported transaction",
            "transaction_date":row.transaction_date,
            "supplier":row.supplier,
            "supplier_tin":row.supplier_tin,
            "currency":row.currency,
            "vat_amount":row.vat_amount,
            "invoice_reference":row.invoice_reference,
            "entry_method":"Imported",
            "source_filename":input.source_filename,
            "source_checksum_sha256":checksum,
            "source_row":row.source_row,
            "import_batch_id":batch_id,
            "mapping_version":1
        });
        let possible_duplicate: bool = if row.invoice_reference.trim().is_empty() {
            false
        } else {
            tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM workspace_record WHERE firm_id=?1 AND client_id=?2 AND record_type='historical_record' AND reference=?3)",
                params![FIRM_ID,input.client_id,row.invoice_reference.trim()], |r|r.get(0)
            ).map_err(|e|e.to_string())?
        };
        payload["possible_duplicate_reference"] = json!(possible_duplicate);
        let notes = format!("Imported from {} · source row {}", input.source_filename, row.source_row);
        tx.execute(
            "INSERT INTO workspace_record(id,firm_id,client_id,record_type,title,status,reference,amount_text,notes,payload_json,row_version,created_at,updated_at,created_by,updated_by)
             VALUES(?1,?2,?3,'historical_record',?4,'Draft',?5,?6,?7,?8,1,?9,?9,?10,?10)",
            params![record_id,FIRM_ID,input.client_id,row.description.trim(),row.invoice_reference.trim(),row.amount_text.trim(),notes,payload.to_string(),imported_at,actor_id]
        ).map_err(|e|e.to_string())?;
        let snapshot = json!({
            "client_id":input.client_id,"record_type":"historical_record","title":row.description,
            "status":"Draft","reference":row.invoice_reference,"amount_text":row.amount_text,
            "notes":notes,"payload":payload
        });
        tx.execute(
            "INSERT INTO workspace_record_version(id,firm_id,record_id,row_version,actor_id,changed_at,snapshot_json,correlation_id)
             VALUES(?1,?2,?3,1,?4,?5,?6,?7)",
            params![Uuid::new_v4().to_string(),FIRM_ID,record_id,actor_id,imported_at,snapshot.to_string(),correlation_id]
        ).map_err(|e|e.to_string())?;
    }
    tx.execute(
        "INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json)
         VALUES(?1,?2,?3,'historical_import_batch_created','historical_import_batch',?4,?5,?6,?7)",
        params![Uuid::new_v4().to_string(),FIRM_ID,actor_id,batch_id,imported_at,correlation_id,
            json!({"source_filename":input.source_filename,"source_checksum_sha256":checksum,"source_row_count":input.rows.len(),"imported_row_count":input.rows.len()}).to_string()]
    ).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())?;
    Ok(HistoricalImportResult {
        batch_id,source_checksum_sha256:checksum,source_row_count:input.rows.len(),imported_row_count:input.rows.len()
    })
}

#[tauri::command]
pub fn list_historical_import_batches(client_id: String, state: State<'_, AppState>) -> Result<Vec<HistoricalImportBatch>, String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let active: bool = db.connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM client WHERE id=?1 AND firm_id=?2 AND status='Active')",
        params![client_id,FIRM_ID], |r|r.get(0)
    ).map_err(|e|e.to_string())?;
    if !active { return Err("Choose an active client in this demonstration firm.".into()); }
    let (_,actor_name,persona) = active_actor(&db)?;
    if !actor_has_client_access(&db,&client_id,&actor_name,&persona)? { return Err("This client workspace is not assigned to your demo persona.".into()); }
    let mut stmt = db.connection.prepare(
        "SELECT id,source_filename,source_checksum_sha256,source_row_count,imported_row_count,imported_at
         FROM historical_import_batch WHERE firm_id=?1 AND client_id=?2 ORDER BY imported_at DESC"
    ).map_err(|e|e.to_string())?;
    let rows = stmt.query_map(params![FIRM_ID,client_id], |r| Ok(HistoricalImportBatch {
        id:r.get(0)?,source_filename:r.get(1)?,source_checksum_sha256:r.get(2)?,
        source_row_count:r.get(3)?,imported_row_count:r.get(4)?,imported_at:r.get(5)?
    })).map_err(|e|e.to_string())?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())
}

#[tauri::command]
pub fn list_records(record_type: String, client_id: Option<String>, state: State<'_, AppState>) -> Result<Vec<WorkspaceRecord>, String> {
    if !allowed_record_type(&record_type) { return Err("Unsupported workspace record type.".into()); }
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let client_filter = if record_type == "staff" {
        None
    } else {
        let selected = client_id.ok_or_else(||"Select a client workspace before viewing these records.".to_string())?;
        let active: bool = db.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM client WHERE id=?1 AND firm_id=?2 AND status='Active')",
            params![selected,FIRM_ID], |r|r.get(0)
        ).map_err(|e|e.to_string())?;
        if !active { return Err("Selected client is not active in this demonstration firm.".into()); }
        let (_,actor_name,persona) = active_actor(&db)?;
        if !actor_has_client_access(&db,&selected,&actor_name,&persona)? {
            return Err("This client workspace is not assigned to your demo persona.".into());
        }
        Some(selected)
    };
    if record_type == "staff" {
        let (_,_,persona) = active_actor(&db)?;
        if !["administrator","manager"].contains(&persona.as_str()) { return Err("The staff directory is restricted to the demo administrator and manager.".into()); }
    }
    let mut stmt = db.connection.prepare(
        "SELECT id,client_id,record_type,title,status,reference,amount_text,notes,payload_json,row_version,updated_at
         FROM workspace_record WHERE firm_id=?1 AND record_type=?2 AND client_id IS ?3 ORDER BY updated_at DESC"
    ).map_err(|e|e.to_string())?;
    let rows = stmt.query_map(params![FIRM_ID,record_type,client_filter], |r| Ok(WorkspaceRecord {
        id:r.get(0)?,client_id:r.get(1)?,record_type:r.get(2)?,title:r.get(3)?,status:r.get(4)?,
        reference:r.get(5)?,amount_text:r.get(6)?,notes:r.get(7)?,payload_json:r.get(8)?,row_version:r.get(9)?,updated_at:r.get(10)?
    })).map_err(|e|e.to_string())?;
    let mut records = rows.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    let (_, actor_name, persona) = active_actor(&db)?;
    if record_type == "task" && persona == "auditor" {
        records.retain(|record| {
            let payload: serde_json::Value = serde_json::from_str(&record.payload_json).unwrap_or_else(|_|json!({}));
            payload.get("assignee").and_then(|v|v.as_str()) == Some(actor_name.as_str())
        });
    } else if record_type == "task" && persona == "reviewer" {
        records.retain(|record| {
            let payload: serde_json::Value = serde_json::from_str(&record.payload_json).unwrap_or_else(|_|json!({}));
            payload.get("reviewer").and_then(|v|v.as_str()) == Some(actor_name.as_str())
        });
    }
    Ok(records)
}

#[tauri::command]
pub fn save_record(input: RecordInput, state: State<'_, AppState>) -> Result<WorkspaceRecord, String> {
    if !allowed_record_type(&input.record_type) {
        return Err("Unsupported workspace record type.".into());
    }
    if input.record_type == "import_cost_input" {
        return Err("Import-cost inputs remain locked until Gate 5-A discovery is approved. Record discovery questions in the import case instead.".into());
    }
    if !allowed_status(&input.record_type, &input.status) {
        return Err("Unsupported status for this record type.".into());
    }
    if input.title.trim().is_empty() || input.title.chars().count() > 200 {
        return Err("A title of 1–200 characters is required.".into());
    }
    let payload: serde_json::Value = match input.payload_json.as_deref() {
        Some(raw) if raw.len() <= 16_384 => serde_json::from_str(raw).map_err(|_| "Record metadata must be valid JSON.".to_string())?,
        Some(_) => return Err("Record metadata must be under 16 KB.".into()),
        None => json!({}),
    };

    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (actor_id, actor_name, persona) = ensure_can_mutate(&db)?;
    if input.record_type == "staff" && !["administrator","manager"].contains(&persona.as_str()) {
        return Err("Only the demo administrator or manager can maintain staff records.".into());
    }
    if input.record_type == "staff" {
        if input.client_id.is_some() { return Err("Staff records belong to the demonstration firm, not a client workspace.".into()); }
    } else {
        let client_id = input.client_id.as_deref().ok_or_else(||"Select the client workspace for this record.".to_string())?;
        let active: bool = db.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM client WHERE id=?1 AND firm_id=?2 AND status='Active')",
            params![client_id,FIRM_ID], |r| r.get(0)
        ).map_err(|e| e.to_string())?;
        if !active { return Err("Choose an active client in this demonstration firm.".into()); }
        if !actor_has_client_access(&db,client_id,&actor_name,&persona)? {
            return Err("This client workspace is not assigned to your demo persona.".into());
        }
    }

    let id = input.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
    let invoice_duplicate_candidates: Vec<(String,String)> = if input.record_type == "invoice" {
        let invoice_number = payload.get("invoice_number").and_then(|v|v.as_str()).unwrap_or("").trim();
        if invoice_number.is_empty() {
            Vec::new()
        } else {
            let mut stmt = db.connection.prepare(
                "SELECT id,payload_json FROM workspace_record WHERE firm_id=?1 AND client_id=?2 AND record_type='invoice' AND id<>?3 ORDER BY updated_at DESC LIMIT 500"
            ).map_err(|e|e.to_string())?;
            let rows = stmt.query_map(params![FIRM_ID,input.client_id.as_deref().unwrap_or(""),input.id.as_deref().unwrap_or("")], |r| Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).map_err(|e|e.to_string())?;
            let mut matches = Vec::new();
            for row in rows {
                let (candidate_id,raw) = row.map_err(|e|e.to_string())?;
                let candidate: serde_json::Value = serde_json::from_str(&raw).unwrap_or_else(|_|json!({}));
                let candidate_number = candidate.get("invoice_number").and_then(|v|v.as_str()).unwrap_or("").trim();
                if candidate_number.eq_ignore_ascii_case(invoice_number) {
                    matches.push((candidate_id,candidate_number.to_string()));
                }
            }
            matches
        }
    } else { Vec::new() };
    if !invoice_duplicate_candidates.is_empty() {
        let reason = input.duplicate_acknowledgement_reason.as_deref().unwrap_or("").trim();
        if input.duplicate_acknowledged != Some(true) || reason.chars().count() < 3 || reason.chars().count() > 500 {
            return Err("A possible duplicate invoice exists for this client. Compare it and record a 3–500 character reason before saving as a separate invoice.".into());
        }
    }
    let exists: Option<(i64,String,String,String,Option<String>)> = db.connection.query_row(
        "SELECT row_version,record_type,status,payload_json,client_id FROM workspace_record WHERE id=?1 AND firm_id=?2",
        params![id,FIRM_ID],
        |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))
    ).optional().map_err(|e|e.to_string())?;

    if input.record_type == "task" {
        let description = payload.get("description").and_then(|v|v.as_str()).unwrap_or("").trim();
        let assignee = payload.get("assignee").and_then(|v|v.as_str()).unwrap_or("").trim();
        let reviewer = payload.get("reviewer").and_then(|v|v.as_str()).unwrap_or("").trim();
        if description.is_empty() || (input.status != "Draft" && assignee.is_empty()) {
            return Err("A task needs a description; assign an active preparer before leaving Draft.".into());
        }
        if input.status == "Submitted" && payload.get("completion_summary").and_then(|v|v.as_str()).unwrap_or("").trim().is_empty() {
            return Err("Add a completion summary before submitting the task for review.".into());
        }
        if input.status == "Blocked" && payload.get("blocker_explanation").and_then(|v|v.as_str()).unwrap_or("").trim().is_empty() {
            return Err("Explain the blocker before moving the task to Blocked.".into());
        }
        if input.status == "Assigned" {
            if reviewer.is_empty() || reviewer == assignee {
                return Err("Assign a separate active reviewer; the preparer cannot review their own task.".into());
            }
            for staff_name in [assignee, reviewer] {
                let active_staff: bool = db.connection.query_row(
                    "SELECT EXISTS(SELECT 1 FROM workspace_record WHERE firm_id=?1 AND record_type='staff' AND status='Active' AND title=?2)",
                    params![FIRM_ID,staff_name], |r| r.get(0)
                ).map_err(|e|e.to_string())?;
                if !active_staff { return Err("Both preparer and reviewer must be active staff in this demo firm.".into()); }
            }
        }
        if let Some((_, existing_type, _, existing_payload, _)) = exists.as_ref() {
            let prior: serde_json::Value = serde_json::from_str(existing_payload).unwrap_or_else(|_|json!({}));
            let prior_assignee = prior.get("assignee").and_then(|v|v.as_str()).unwrap_or("");
            let prior_reviewer = prior.get("reviewer").and_then(|v|v.as_str()).unwrap_or("");
            if existing_type != "task" { return Err("Task record type mismatch.".into()); }
            if (!prior_assignee.is_empty() && prior_assignee != assignee) || (!prior_reviewer.is_empty() && prior_reviewer != reviewer) {
                if !["administrator","manager"].contains(&persona.as_str()) {
                    return Err("Only a manager can reassign a task or reviewer.".into());
                }
                if payload.get("reassignment_reason").and_then(|v|v.as_str()).unwrap_or("").trim().chars().count() < 3 {
                    return Err("Record a reason of at least 3 characters when changing the assignee or reviewer.".into());
                }
            }
            if persona == "auditor" {
                if prior_assignee != actor_name.as_str() || assignee != actor_name.as_str() || prior_reviewer != reviewer {
                    return Err("You can only update your own assigned task; assignment and reviewer are controlled by the manager.".into());
                }
                if !["In progress","Blocked","Submitted"].contains(&input.status.as_str()) {
                    return Err("The preparer may start, block or submit their assigned task; managers/reviewers control review outcomes.".into());
                }
            }
        } else if persona == "auditor" {
            return Err("Demo auditors can only work on tasks already assigned to them.".into());
        }
        if input.status == "Completed" || input.status == "Returned" {
            if !["administrator","manager"].contains(&persona.as_str()) || reviewer != actor_name.as_str() {
                return Err("Only the separately assigned demo reviewer can return or clear this task.".into());
            }
            if input.status == "Returned" {
                let open_notes: i64 = db.connection.query_row(
                    "SELECT COUNT(*) FROM task_review_note WHERE task_id=?1 AND status IN ('Open','Responded','Reopened')",
                    params![id], |r|r.get(0)
                ).map_err(|e|e.to_string())?;
                if open_notes == 0 { return Err("Add a specific review note before returning the task.".into()); }
            }
            if input.status == "Completed" {
                let open_notes: i64 = db.connection.query_row(
                    "SELECT COUNT(*) FROM task_review_note WHERE task_id=?1 AND status IN ('Open','Responded','Reopened')",
                    params![id], |r|r.get(0)
                ).map_err(|e|e.to_string())?;
                if open_notes > 0 { return Err("Clear or reopen every review note before accepting task completion.".into()); }
            }
        }
    }

    if input.record_type == "task" && input.status == "Cancelled" {
        if !["administrator","manager"].contains(&persona.as_str()) {
            return Err("Only a manager can cancel a task.".into());
        }
        if payload.get("cancellation_reason").and_then(|v|v.as_str()).unwrap_or("").trim().chars().count() < 3 {
            return Err("Record a cancellation reason of at least 3 characters.".into());
        }
        let open_notes: i64 = db.connection.query_row(
            "SELECT COUNT(*) FROM task_review_note WHERE task_id=?1 AND status IN ('Open','Responded','Reopened')",
            params![id], |r|r.get(0)
        ).map_err(|e|e.to_string())?;
        if open_notes > 0 { return Err("Resolve every task review note before cancellation.".into()); }
    }
    if input.record_type == "task" {
        if let Some((_,_,previous_status,previous_payload,_)) = exists.as_ref() {
            if ["Submitted","Completed","Cancelled"].contains(&previous_status.as_str()) {
                let previous: serde_json::Value = serde_json::from_str(previous_payload).unwrap_or_else(|_|json!({}));
                let mut previous_compare = previous.clone();
                let mut payload_compare = payload.clone();
                if previous_status == "Submitted" && input.status == "Cancelled" {
                    if let Some(object) = previous_compare.as_object_mut() { object.remove("cancellation_reason"); }
                    if let Some(object) = payload_compare.as_object_mut() { object.remove("cancellation_reason"); }
                }
                let old_values: (String,String,String,String) = db.connection.query_row(
                    "SELECT title,reference,amount_text,notes FROM workspace_record WHERE id=?1 AND firm_id=?2",
                    params![id,FIRM_ID], |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))
                ).map_err(|e|e.to_string())?;
                if previous_compare != payload_compare || old_values.0 != input.title.trim()
                    || old_values.1 != input.reference.as_deref().unwrap_or("")
                    || old_values.2 != input.amount_text.as_deref().unwrap_or("")
                    || old_values.3 != input.notes.as_deref().unwrap_or("") {
                    return Err("Submitted or terminal task content is immutable. Record a review note and return the task for correction instead of editing it.".into());
                }
                if previous_status != "Submitted" && input.status != previous_status {
                    return Err("Completed and cancelled tasks are terminal and cannot be reopened or rewritten.".into());
                }
            }
        }
    }

    if input.record_type == "invoice" && input.status == "Submitted for review"
        && payload.get("invoice_date").and_then(|v|v.as_str()).unwrap_or("").is_empty() {
        return Err("Enter the invoice date before submitting it for review.".into());
    }
    if input.record_type == "period" && input.status == "Ready for review"
        && (payload.get("start_date").and_then(|v|v.as_str()).unwrap_or("").is_empty()
            || payload.get("end_date").and_then(|v|v.as_str()).unwrap_or("").is_empty()) {
        return Err("Enter both period dates before marking the period ready for review.".into());
    }
    if input.record_type == "historical_record" && input.status == "Ready for review"
        && input.reference.as_deref().unwrap_or("").trim().is_empty()
            && payload.get("source_filename").and_then(|v|v.as_str()).unwrap_or("").trim().is_empty() {
        return Err("Add a source reference before marking historical data ready for review.".into());
    }

    if let Some((version, existing_type, previous_status, _, existing_client_id)) = exists.as_ref() {
        if existing_type != &input.record_type {
            return Err("A record cannot be changed into a different record type.".into());
        }
        if existing_client_id.as_deref() != input.client_id.as_deref() {
            return Err("A record cannot be moved between client workspaces. Create a new record under the intended client instead.".into());
        }
        if input.row_version != Some(*version) {
            return Err("This record changed since it was opened. Reload before saving.".into());
        }
        if !valid_record_transition(&input.record_type, Some(previous_status), &input.status) {
            return Err("This workflow status transition is not permitted.".into());
        }
    } else if !valid_record_transition(&input.record_type, None, &input.status) {
        return Err("A new record must start in its initial draft/discovery state.".into());
    }

    let now = now_utc();
    let tx = db.connection.unchecked_transaction().map_err(|e|e.to_string())?;
    if exists.is_some() {
        tx.execute(
            "UPDATE workspace_record SET client_id=?1,title=?2,status=?3,reference=?4,amount_text=?5,notes=?6,payload_json=?7,row_version=row_version+1,updated_at=?8,updated_by=?9 WHERE id=?10 AND firm_id=?11",
            params![input.client_id.as_deref(),input.title.trim(),input.status,input.reference.as_deref().unwrap_or(""),input.amount_text.as_deref().unwrap_or(""),input.notes.as_deref().unwrap_or(""),payload.to_string(),now,actor_id,id,FIRM_ID]
        ).map_err(|e|e.to_string())?;
    } else {
        tx.execute(
            "INSERT INTO workspace_record(id,firm_id,client_id,record_type,title,status,reference,amount_text,notes,payload_json,row_version,created_at,updated_at,created_by,updated_by)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,1,?11,?11,?12,?12)",
            params![id,FIRM_ID,input.client_id.as_deref(),input.record_type,input.title.trim(),input.status,input.reference.as_deref().unwrap_or(""),input.amount_text.as_deref().unwrap_or(""),input.notes.as_deref().unwrap_or(""),payload.to_string(),now,actor_id]
        ).map_err(|e|e.to_string())?;
    }
    let new_version = exists.as_ref().map(|(version,_,_,_,_)| *version + 1).unwrap_or(1);
    let snapshot = json!({
        "client_id":input.client_id.as_deref(),"record_type":input.record_type,"title":input.title,
        "status":input.status,"reference":input.reference.as_deref().unwrap_or(""),
        "amount_text":input.amount_text.as_deref().unwrap_or(""),"notes":input.notes.as_deref().unwrap_or(""),
        "payload":payload,"duplicate_acknowledgement_reason":input.duplicate_acknowledgement_reason.as_deref().unwrap_or("")
    });
    let correlation_id = Uuid::new_v4().to_string();
    tx.execute(
        "INSERT INTO workspace_record_version(id,firm_id,record_id,row_version,actor_id,changed_at,snapshot_json,correlation_id)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
        params![Uuid::new_v4().to_string(),FIRM_ID,id,new_version,actor_id,now,snapshot.to_string(),correlation_id]
    ).map_err(|e|e.to_string())?;
    tx.execute(
        "INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json)
         VALUES(?1,?2,?3,?4,'workspace_record',?5,?6,?7,?8)",
        params![Uuid::new_v4().to_string(),FIRM_ID,actor_id,if exists.is_some(){"workspace_record_updated"}else{"workspace_record_created"},id,now,correlation_id,json!({"record_type":input.record_type,"status":input.status,"row_version":new_version,"possible_duplicate_count":invoice_duplicate_candidates.len(),"duplicate_acknowledgement_reason":input.duplicate_acknowledgement_reason.as_deref().unwrap_or("")}).to_string()]
    ).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())?;

    let mut stmt = db.connection.prepare(
        "SELECT id,client_id,record_type,title,status,reference,amount_text,notes,payload_json,row_version,updated_at FROM workspace_record WHERE id=?1"
    ).map_err(|e|e.to_string())?;
    stmt.query_row([id], |r| Ok(WorkspaceRecord {
        id:r.get(0)?,client_id:r.get(1)?,record_type:r.get(2)?,title:r.get(3)?,status:r.get(4)?,
        reference:r.get(5)?,amount_text:r.get(6)?,notes:r.get(7)?,payload_json:r.get(8)?,row_version:r.get(9)?,updated_at:r.get(10)?
    })).map_err(|e|e.to_string())
}

#[derive(Serialize)]
pub struct WorkspaceRecordVersion {
    id: String,
    row_version: i64,
    actor_name: String,
    changed_at: String,
    snapshot_json: String,
}

#[tauri::command]
pub fn list_workspace_record_versions(record_id: String, state: State<'_, AppState>) -> Result<Vec<WorkspaceRecordVersion>, String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (_, actor_name, persona) = active_actor(&db)?;
    let (record_type, client_id, raw): (String,Option<String>,String) = db.connection.query_row(
        "SELECT record_type,client_id,payload_json FROM workspace_record WHERE id=?1 AND firm_id=?2",
        params![record_id,FIRM_ID], |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))
    ).map_err(|_|"Record not found in this demonstration firm.".to_string())?;
    if let Some(client_id) = client_id.as_deref() {
        if !actor_has_client_access(&db,client_id,&actor_name,&persona)? {
            return Err("This client workspace is not assigned to your demo persona.".into());
        }
    } else if !["administrator","manager"].contains(&persona.as_str()) {
        return Err("Firm-level record history is restricted to the demo administrator and manager.".into());
    }
    if record_type == "task" {
        let payload: serde_json::Value = serde_json::from_str(&raw).unwrap_or_else(|_|json!({}));
        let assignee = payload.get("assignee").and_then(|v|v.as_str()).unwrap_or("");
        let reviewer = payload.get("reviewer").and_then(|v|v.as_str()).unwrap_or("");
        if (persona == "auditor" && assignee != actor_name.as_str())
            || (persona == "reviewer" && reviewer != actor_name.as_str()) {
            return Err("This task history is not assigned to your demo persona.".into());
        }
    }
    let mut stmt = db.connection.prepare(
        "SELECT v.id,v.row_version,a.display_name,v.changed_at,v.snapshot_json
         FROM workspace_record_version v JOIN demo_actor a ON a.id=v.actor_id
         WHERE v.firm_id=?1 AND v.record_id=?2 ORDER BY v.row_version DESC"
    ).map_err(|e|e.to_string())?;
    let rows = stmt.query_map(params![FIRM_ID,record_id], |r| Ok(WorkspaceRecordVersion {
        id:r.get(0)?,row_version:r.get(1)?,actor_name:r.get(2)?,changed_at:r.get(3)?,snapshot_json:r.get(4)?
    })).map_err(|e|e.to_string())?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())
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
    tx.execute_batch("DROP TRIGGER IF EXISTS task_review_note_no_delete; DROP TRIGGER IF EXISTS task_review_note_content_immutable; DROP TRIGGER IF EXISTS task_review_note_status_transition;
        DROP TRIGGER IF EXISTS client_change_no_update; DROP TRIGGER IF EXISTS client_change_no_delete;
        DROP TRIGGER IF EXISTS workspace_record_version_no_update; DROP TRIGGER IF EXISTS workspace_record_version_no_delete;
        DROP TRIGGER IF EXISTS historical_import_batch_no_update; DROP TRIGGER IF EXISTS historical_import_batch_no_delete;").map_err(|e|e.to_string())?;
    tx.execute("DELETE FROM task_review_note WHERE firm_id=?1",[FIRM_ID]).map_err(|e|e.to_string())?;
    tx.execute("DELETE FROM workspace_record WHERE firm_id=?1 AND record_type<>'staff'",[FIRM_ID]).map_err(|e|e.to_string())?;
    tx.execute("DELETE FROM local_note WHERE firm_id=?1",[FIRM_ID]).map_err(|e|e.to_string())?;
    tx.execute("DELETE FROM historical_import_batch WHERE firm_id=?1",[FIRM_ID]).map_err(|e|e.to_string())?;
    tx.execute("DELETE FROM client_change WHERE firm_id=?1",[FIRM_ID]).map_err(|e|e.to_string())?;
    tx.execute("DELETE FROM client WHERE firm_id=?1",[FIRM_ID]).map_err(|e|e.to_string())?;
    tx.execute_batch("CREATE TRIGGER task_review_note_no_delete BEFORE DELETE ON task_review_note BEGIN SELECT RAISE(ABORT, 'task review notes cannot be deleted'); END;
        CREATE TRIGGER task_review_note_content_immutable BEFORE UPDATE ON task_review_note
        WHEN NEW.firm_id<>OLD.firm_id OR NEW.task_id<>OLD.task_id OR NEW.reviewer_actor_id<>OLD.reviewer_actor_id
          OR NEW.category<>OLD.category OR NEW.description<>OLD.description OR NEW.affected_record_reference<>OLD.affected_record_reference
          OR NEW.required_response<>OLD.required_response OR NEW.created_at<>OLD.created_at OR NEW.correlation_id<>OLD.correlation_id
        BEGIN SELECT RAISE(ABORT, 'task review note content is immutable'); END;
        CREATE TRIGGER task_review_note_status_transition BEFORE UPDATE OF status ON task_review_note
        WHEN NOT ((OLD.status IN ('Open','Reopened') AND NEW.status='Responded') OR (OLD.status='Responded' AND NEW.status IN ('Cleared','Reopened')))
        BEGIN SELECT RAISE(ABORT, 'invalid task review note transition'); END;
        CREATE TRIGGER client_change_no_update BEFORE UPDATE ON client_change BEGIN SELECT RAISE(ABORT, 'client change history is immutable'); END;
        CREATE TRIGGER client_change_no_delete BEFORE DELETE ON client_change BEGIN SELECT RAISE(ABORT, 'client change history is immutable'); END;
        CREATE TRIGGER workspace_record_version_no_update BEFORE UPDATE ON workspace_record_version BEGIN SELECT RAISE(ABORT, 'workspace record versions are immutable'); END;
        CREATE TRIGGER workspace_record_version_no_delete BEFORE DELETE ON workspace_record_version BEGIN SELECT RAISE(ABORT, 'workspace record versions are immutable'); END;
        CREATE TRIGGER historical_import_batch_no_update BEFORE UPDATE ON historical_import_batch BEGIN SELECT RAISE(ABORT, 'historical import batches are immutable'); END;
        CREATE TRIGGER historical_import_batch_no_delete BEFORE DELETE ON historical_import_batch BEGIN SELECT RAISE(ABORT, 'historical import batches are immutable'); END;").map_err(|e|e.to_string())?;
    tx.execute("INSERT INTO client(id,firm_id,record_kind,legal_name,display_name,status,created_at,updated_at,created_by,updated_by) VALUES(?1,?2,'Company','Sample Trading PLC','Sample Trading','Active',?3,?3,?4,?4)",
        params!["00000000-0000-4000-8000-000000000101",FIRM_ID,now_utc(),actor_id]).map_err(|e|e.to_string())?;
    tx.execute("INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json) VALUES(?1,?2,?3,'demo_data_reset','demo_workspace',?2,?4,?5,'{}')",
        params![Uuid::new_v4().to_string(),FIRM_ID,actor_id,now_utc(),Uuid::new_v4().to_string()]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())
}


#[cfg(test)]
mod tests {
    use super::valid_record_transition;

    #[test]
    fn invoice_and_tax_workflows_cannot_skip_review_or_claim_approval() {
        assert!(valid_record_transition("invoice", None, "Draft"));
        assert!(valid_record_transition("invoice", Some("Draft"), "Submitted for review"));
        assert!(!valid_record_transition("invoice", None, "Reviewed"));
        assert!(!valid_record_transition("invoice", Some("Draft"), "Reviewed"));
        assert!(!valid_record_transition("historical_record", Some("Ready for review"), "Reviewed"));
    }

    #[test]
    fn task_lifecycle_requires_submission_before_completion() {
        assert!(valid_record_transition("task", None, "Draft"));
        assert!(valid_record_transition("task", Some("Assigned"), "In progress"));
        assert!(valid_record_transition("task", Some("In progress"), "Submitted"));
        assert!(valid_record_transition("task", Some("Submitted"), "Completed"));
        assert!(!valid_record_transition("task", Some("In progress"), "Completed"));
        assert!(!valid_record_transition("task", Some("Completed"), "In progress"));
    }

    #[test]
    fn import_cost_calculation_input_cannot_bypass_discovery_gate() {
        assert!(valid_record_transition("import_case", None, "Discovery"));
        assert!(valid_record_transition("import_case", Some("Discovery"), "Ready for workshop"));
        assert!(!valid_record_transition("import_cost_input", None, "Discovery"));
    }
}
