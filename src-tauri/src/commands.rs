use crate::{db::{now_utc, Database}, import_cost::{calculate_import_cost, CostComponentInput, GoodsLineInput, ImportCostResult}, AppState};
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
        "SELECT payload_json FROM workspace_record WHERE firm_id=?1 AND client_id=?2 AND record_type='task' AND status<>'Draft'"
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

#[derive(Serialize)]
pub struct ClientChange {
    id: String, action: String, changed_at: String, actor_name: String, before_json: String, after_json: String,
}

#[tauri::command]
pub fn list_client_changes(client_id: String, state: State<'_, AppState>) -> Result<Vec<ClientChange>, String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
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
        "invoice" => ["Draft","Needs correction","Ready for review","Reviewed"].contains(&status),
        "historical_record" | "period" => ["Draft","Needs source","Ready for review","Reviewed"].contains(&status),
        "staff" => ["Draft","Active","Inactive"].contains(&status),
        "task" => ["Draft","Assigned","In progress","Blocked","Submitted","Returned","Completed","Cancelled"].contains(&status),
        "import_case" => ["Discovery","Inputs captured","Needs review","Reviewed","Deferred"].contains(&status),
        "import_cost_input" => ["Draft","Needs source","Ready for review","Reviewed"].contains(&status),
        _ => false,
    }
}

fn valid_record_transition(record_type: &str, previous: Option<&str>, next: &str) -> bool {
    match (record_type, previous, next) {
        ("category" | "product", None, "Draft") => true,
        ("category" | "product", Some("Draft"), "Draft" | "Active") => true,
        ("category" | "product", Some("Active"), "Active" | "Archived") => true,
        ("invoice", None, "Draft") => true,
        ("invoice", Some("Draft"), "Draft" | "Ready for review") => true,
        ("invoice", Some("Ready for review"), "Ready for review" | "Needs correction" | "Reviewed") => true,
        ("invoice", Some("Needs correction"), "Needs correction" | "Draft" | "Ready for review") => true,
        ("invoice", Some("Reviewed"), "Reviewed") => true,
        ("historical_record" | "period", None, "Draft") => true,
        ("historical_record" | "period", Some("Draft"), "Draft" | "Needs source" | "Ready for review") => true,
        ("historical_record" | "period", Some("Needs source"), "Needs source" | "Ready for review") => true,
        ("historical_record" | "period", Some("Ready for review"), "Ready for review" | "Needs source" | "Reviewed") => true,
        ("historical_record" | "period", Some("Reviewed"), "Reviewed") => true,
        ("staff", None, "Draft") => true,
        ("staff", Some("Draft"), "Draft" | "Active") => true,
        ("staff", Some("Active"), "Active" | "Inactive") => true,
        ("staff", Some("Inactive"), "Inactive" | "Active") => true,
        ("task", None, "Draft") => true,
        ("task", Some("Draft"), "Draft" | "Assigned" | "Cancelled") => true,
        ("task", Some("Assigned"), "Assigned" | "In progress" | "Cancelled") => true,
        ("task", Some("In progress"), "In progress" | "Blocked" | "Submitted" | "Cancelled") => true,
        ("task", Some("Blocked"), "Blocked" | "In progress" | "Cancelled") => true,
        ("task", Some("Submitted"), "Submitted" | "Returned" | "Completed" | "Cancelled") => true,
        ("task", Some("Returned"), "Returned" | "In progress" | "Cancelled") => true,
        ("task", Some("Completed"), "Completed") | ("task", Some("Cancelled"), "Cancelled") => true,
        ("import_case", None, "Discovery") => true,
        ("import_case", Some("Discovery"), "Discovery" | "Inputs captured" | "Deferred") => true,
        ("import_case", Some("Inputs captured"), "Inputs captured" | "Needs review" | "Deferred") => true,
        ("import_case", Some("Needs review"), "Needs review" | "Inputs captured" | "Reviewed" | "Deferred") => true,
        ("import_case", Some("Reviewed"), "Reviewed" | "Deferred") => true,
        ("import_case", Some("Deferred"), "Deferred" | "Discovery") => true,
        ("import_cost_input", None, "Draft") => true,
        ("import_cost_input", Some("Draft"), "Draft" | "Needs source" | "Ready for review") => true,
        ("import_cost_input", Some("Needs source"), "Needs source" | "Draft" | "Ready for review") => true,
        ("import_cost_input", Some("Ready for review"), "Ready for review" | "Needs source" | "Reviewed") => true,
        ("import_cost_input", Some("Reviewed"), "Reviewed") => true,
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
    let (actor_id,_,_) = active_actor(&db)?;
    let mut stmt = db.connection.prepare("SELECT id,client_id,title,body,status,updated_at FROM local_note WHERE firm_id=?1 AND actor_id=?2 ORDER BY updated_at DESC").map_err(|e|e.to_string())?;
    let rows = stmt.query_map(params![FIRM_ID,actor_id], |r| Ok(LocalNote { id:r.get(0)?,client_id:r.get(1)?,title:r.get(2)?,body:r.get(3)?,status:r.get(4)?,updated_at:r.get(5)? })).map_err(|e|e.to_string())?;
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
        let active: bool = db.connection.query_row("SELECT EXISTS(SELECT 1 FROM client WHERE id=?1 AND firm_id=?2 AND status='Active')",params![client_id,FIRM_ID],|r|r.get(0)).map_err(|e|e.to_string())?;
        if !active { return Err("Selected client is not active in this demo workspace.".into()); }
        }
    let id = input.id.unwrap_or_else(||Uuid::new_v4().to_string());
    let now = now_utc();
    let tx = db.connection.unchecked_transaction().map_err(|e|e.to_string())?;
    let changed = tx.execute("INSERT INTO local_note(id,firm_id,client_id,actor_id,title,body,status,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,'Local draft',?7,?7)
        ON CONFLICT(id) DO UPDATE SET client_id=excluded.client_id,actor_id=excluded.actor_id,title=excluded.title,body=excluded.body,updated_at=excluded.updated_at WHERE local_note.firm_id=excluded.firm_id AND local_note.actor_id=excluded.actor_id",
        params![id,FIRM_ID,input.client_id,actor_id,title,input.body,now]).map_err(|e|e.to_string())?;
    if changed == 0 { return Err("Local note was not found or belongs to another demo persona.".into()); }
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
    let (task_status,raw): (String,String) = db.connection.query_row(
        "SELECT status,payload_json FROM workspace_record WHERE id=?1 AND firm_id=?2 AND record_type='task'",
        params![task_id,FIRM_ID], |r|Ok((r.get(0)?,r.get(1)?))
    ).map_err(|_|"Task not found in this demonstration firm.".to_string())?;
    if task_status == "Draft" && ["auditor","reviewer"].contains(&persona.as_str()) {
        return Err("This task has not been assigned for work or review.".into());
    }
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

fn validate_historical_import_rows(rows: &[HistoricalImportRow]) -> Result<(), String> {
    if rows.is_empty() || rows.len() > 1000 {
        return Err("A batch must contain between 1 and 1,000 data rows.".into());
    }
    let mut seen_rows = std::collections::HashSet::new();
    for row in rows {
        if row.source_row < 2 || row.source_row > 100_001 || !seen_rows.insert(row.source_row) {
            return Err("CSV source row numbers must be unique and start at row 2.".into());
        }
        if row.description.trim().is_empty() || row.amount_text.trim().is_empty() {
            return Err(format!("CSV row {} needs both a description and source amount.", row.source_row));
        }
    }
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
    validate_historical_import_rows(&input.rows)?;
    let digest = Sha256::digest(input.source_content.as_bytes());
    let checksum = digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (actor_id,_,_) = ensure_can_mutate(&db)?;
    let active: bool = db.connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM client WHERE id=?1 AND firm_id=?2 AND status='Active')",
        params![input.client_id,FIRM_ID], |r|r.get(0)
    ).map_err(|e|e.to_string())?;
    if !active { return Err("Choose an active client in this demonstration firm.".into()); }

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
pub fn calculate_import_cost_preview(
    client_id: String,
    case_reference: String,
    base_currency: String,
    state: State<'_, AppState>,
) -> Result<ImportCostResult, String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (_, _, persona) = ensure_can_mutate(&db)?;
    if !["administrator", "manager"].contains(&persona.as_str()) {
        return Err("Only the demo administrator or engagement manager may prepare an import-cost calculation preview.".into());
    }
    let client_active: bool = db.connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM client WHERE id=?1 AND firm_id=?2 AND status='Active')",
        params![client_id, FIRM_ID], |r| r.get(0)
    ).map_err(|e| e.to_string())?;
    if !client_active {
        return Err("Select an active client in this demonstration firm.".into());
    }
    let case_reference = case_reference.trim();
    if case_reference.is_empty() {
        return Err("Enter the import case reference used by its saved cost inputs.".into());
    }
    let case: Option<(String, String)> = db.connection.query_row(
        "SELECT status,payload_json FROM workspace_record
         WHERE firm_id=?1 AND client_id=?2 AND record_type='import_case' AND reference=?3
         ORDER BY updated_at DESC LIMIT 1",
        params![FIRM_ID, client_id, case_reference],
        |r| Ok((r.get(0)?, r.get(1)?))
    ).optional().map_err(|e| e.to_string())?;
    let (case_status, case_payload) = case.ok_or_else(|| "Create an import discovery case and use its exact reference before calculating.".to_string())?;
    if case_status != "Reviewed" {
        return Err("The import discovery case must be marked Reviewed before a draft calculation can be prepared.".into());
    }
    let case_data: serde_json::Value = serde_json::from_str(&case_payload).unwrap_or_else(|_| json!({}));
    let case_base_currency = case_data.get("base_currency").and_then(|v| v.as_str()).unwrap_or("").trim().to_ascii_uppercase();
    if case_base_currency.len() != 3 || case_base_currency != base_currency.trim().to_ascii_uppercase() {
        return Err("The calculation base currency must match the three-letter base currency recorded on the reviewed import case.".into());
    }
    for key in ["component_policy_reference", "fx_policy_reference", "allocation_policy_reference", "synthetic_example_reference"] {
        if case_data.get(key).and_then(|v| v.as_str()).unwrap_or("").trim().is_empty() {
            return Err(format!("The reviewed discovery case is missing {key}. Record the supporting policy or synthetic worked-example reference first."));
        }
    }

    let mut stmt = db.connection.prepare(
        "SELECT title,status,payload_json FROM workspace_record
         WHERE firm_id=?1 AND client_id=?2 AND record_type='import_cost_input' AND reference=?3
         ORDER BY created_at,id"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![FIRM_ID, client_id, case_reference], |r| {
        Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,String>(2)?))
    }).map_err(|e| e.to_string())?;

    let read_text = |payload: &serde_json::Value, key: &str| -> String {
        payload.get(key).and_then(|v| v.as_str()).unwrap_or("").trim().to_string()
    };
    let read_number = |payload: &serde_json::Value, key: &str, required: bool| -> Result<i64, String> {
        let raw = match payload.get(key) {
            Some(value) if value.is_number() => value.to_string(),
            Some(value) if value.is_string() => value.as_str().unwrap_or("").trim().to_string(),
            _ => String::new(),
        };
        if raw.is_empty() && !required { return Ok(0); }
        raw.parse::<i64>().map_err(|_| format!("Import-cost field {key} must be a whole-number minor-unit or scaled quantity value."))
    };

    let mut goods = Vec::new();
    let mut components = Vec::new();
    for row in rows {
        let (title, status, raw) = row.map_err(|e| e.to_string())?;
        let payload: serde_json::Value = serde_json::from_str(&raw).map_err(|_| format!("Import-cost input '{title}' contains invalid metadata."))?;
        if read_text(&payload, "entry_kind") == "calculation_result" {
            continue;
        }
        if !["Ready for review", "Reviewed"].contains(&status.as_str()) {
            return Err(format!("Import-cost input '{title}' must be Ready for review before calculation."));
        }
        if read_text(&payload, "source_reference").is_empty() {
            return Err(format!("Import-cost input '{title}' needs a source document/reference before calculation."));
        }
        match read_text(&payload, "entry_kind").as_str() {
            "Goods line" => {
                goods.push(GoodsLineInput {
                    item_reference: read_text(&payload, "item_reference"),
                    description: read_text(&payload, "description"),
                    quantity_micros: read_number(&payload, "quantity_micros", true)?,
                    unit_price_minor: read_number(&payload, "unit_price_minor", true)?,
                    currency_code: read_text(&payload, "currency_code"),
                    source_reference: read_text(&payload, "source_reference"),
                    fx_rate_source: read_text(&payload, "fx_rate_source"),
                    fx_rate_date: read_text(&payload, "fx_rate_date"),
                    fx_authorized_by: read_text(&payload, "fx_authorized_by"),
                    fx_numerator: read_number(&payload, "fx_numerator", true)?,
                    fx_denominator: read_number(&payload, "fx_denominator", true)?,
                    weight_micros: read_number(&payload, "weight_micros", false)?,
                    volume_micros: read_number(&payload, "volume_micros", false)?,
                });
            }
            "Cost component" => {
                components.push(CostComponentInput {
                    component_reference: read_text(&payload, "component_reference"),
                    category: read_text(&payload, "category"),
                    amount_minor: read_number(&payload, "amount_minor", true)?,
                    currency_code: read_text(&payload, "currency_code"),
                    source_reference: read_text(&payload, "source_reference"),
                    fx_rate_source: read_text(&payload, "fx_rate_source"),
                    fx_rate_date: read_text(&payload, "fx_rate_date"),
                    fx_authorized_by: read_text(&payload, "fx_authorized_by"),
                    fx_numerator: read_number(&payload, "fx_numerator", true)?,
                    fx_denominator: read_number(&payload, "fx_denominator", true)?,
                    allocation_basis: read_text(&payload, "allocation_basis"),
                    direct_item_reference: read_text(&payload, "direct_item_reference"),
                    treatment: read_text(&payload, "treatment"),
                });
            }
            _ => return Err(format!("Import-cost input '{title}' must be classified as Goods line or Cost component.")),
        }
    }
    if goods.is_empty() {
        return Err("Add at least one saved Goods line for this import case.".into());
    }
    calculate_import_cost(&base_currency, &goods, &components)
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
            payload.get("assignee").and_then(|v|v.as_str()) == Some(actor_name.as_str()) && record.status != "Draft"
        });
    } else if record_type == "task" && persona == "reviewer" {
        records.retain(|record| {
            let payload: serde_json::Value = serde_json::from_str(&record.payload_json).unwrap_or_else(|_|json!({}));
            payload.get("reviewer").and_then(|v|v.as_str()) == Some(actor_name.as_str()) && record.status != "Draft"
        });
    }
    Ok(records)
}



fn valid_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if !value.is_ascii() || bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' { return false; }
    let year = match value[0..4].parse::<i32>() { Ok(v) if v > 0 => v, _ => return false };
    let month = match value[5..7].parse::<u32>() { Ok(v) if (1..=12).contains(&v) => v, _ => return false };
    let day = match value[8..10].parse::<u32>() { Ok(v) if v > 0 => v, _ => return false };
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month { 1|3|5|7|8|10|12 => 31, 4|6|9|11 => 30, 2 if leap => 29, 2 => 28, _ => 0 };
    day <= days
}

fn validate_period_assignment(
    db: &Database,
    client_id: &str,
    period_reference: &str,
    transaction_date: Option<&str>,
) -> Result<(), String> {
    let period: Option<(String,String)> = db.connection.query_row(
        "SELECT status,payload_json FROM workspace_record
         WHERE firm_id=?1 AND client_id=?2 AND record_type='period' AND reference=?3
         ORDER BY updated_at DESC LIMIT 1",
        params![FIRM_ID,client_id,period_reference.trim()],
        |r| Ok((r.get(0)?,r.get(1)?))
    ).optional().map_err(|e| e.to_string())?;
    let (status,raw) = period.ok_or_else(|| "The reporting period reference does not exist for this client. Create and review the period first.".to_string())?;
    if status != "Reviewed" {
        return Err("The referenced financial period must be reviewed before transactions can be submitted for review.".into());
    }
    let payload: serde_json::Value = serde_json::from_str(&raw).unwrap_or_else(|_| json!({}));
    let start = payload.get("start_date").and_then(|v| v.as_str()).unwrap_or("").trim();
    let end = payload.get("end_date").and_then(|v| v.as_str()).unwrap_or("").trim();
    if !valid_iso_date(start) || !valid_iso_date(end) || start > end {
        return Err("The reviewed period must have valid ISO start and end dates.".into());
    }
    if let Some(date) = transaction_date {
        let date = date.trim();
        if !valid_iso_date(date) || date < start || date > end {
            return Err(format!("Transaction date {date} falls outside the reviewed period {start} to {end}."));
        }
    }
    Ok(())
}

#[tauri::command]
pub fn save_record(input: RecordInput, state: State<'_, AppState>) -> Result<WorkspaceRecord, String> {
    if !allowed_record_type(&input.record_type) {
        return Err("Unsupported workspace record type.".into());
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
    if input.record_type == "staff" && !["administrator","manager"].contains(&persona.as_str()) { return Err("Only the demo administrator or manager can edit the staff directory.".into()); }
    if input.record_type == "staff" {
        if input.client_id.is_some() { return Err("Staff records belong to the demonstration firm, not a client workspace.".into()); }
    } else {
        let client_id = input.client_id.as_deref().ok_or_else(||"Select the client workspace for this record.".to_string())?;
        let active: bool = db.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM client WHERE id=?1 AND firm_id=?2 AND status='Active')",
            params![client_id,FIRM_ID], |r| r.get(0)
        ).map_err(|e| e.to_string())?;
        if !active { return Err("Choose an active client in this demonstration firm.".into()); }
    }



    if input.record_type == "invoice" && input.status == "Ready for review" {
        let raw_lines = payload.get("invoice_lines_json").and_then(|v| v.as_str()).unwrap_or("").trim();
        let legacy_description = payload.get("line_items_text").and_then(|v| v.as_str()).unwrap_or("").trim();
        if raw_lines.is_empty() || raw_lines == "[]" {
            if legacy_description.is_empty() {
                return Err("Capture at least one structured invoice line or preserve the source line descriptions before review.".into());
            }
        } else {
            let lines: Vec<serde_json::Value> = serde_json::from_str(raw_lines)
                .map_err(|_| "Structured invoice lines must be a valid JSON array.".to_string())?;
            if lines.is_empty() || lines.len() > 200 {
                return Err("An invoice must contain 1–200 structured lines before review.".into());
            }
            for (index, line) in lines.iter().enumerate() {
                if line.get("description").and_then(|v| v.as_str()).unwrap_or("").trim().is_empty()
                    || line.get("line_amount_text").and_then(|v| v.as_str()).unwrap_or("").trim().is_empty() {
                    return Err(format!("Invoice line {} needs a description and the line amount exactly as shown on the source.", index + 1));
                }
            }
        }
        let source_reference = payload.get("source_reference").and_then(|v| v.as_str()).unwrap_or("").trim();
        let has_attachment = if let Some(record_id) = input.id.as_deref() {
            db.connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM source_document WHERE firm_id=?1 AND client_id=?2 AND record_id=?3)",
                params![FIRM_ID,input.client_id.as_deref().unwrap_or(""),record_id], |r| r.get::<_,bool>(0)
            ).unwrap_or(false)
        } else { false };
        if source_reference.is_empty() && !has_attachment {
            return Err("Add a source document reference or attach the original invoice before submitting it for review.".into());
        }
        let period_reference = payload.get("financial_period_reference").and_then(|v| v.as_str()).unwrap_or("").trim();
        let invoice_date = payload.get("invoice_date").and_then(|v| v.as_str()).unwrap_or("").trim();
        if period_reference.is_empty() || invoice_date.is_empty() {
            return Err("Record the invoice date and explicitly assign a reviewed financial period before review.".into());
        }
        validate_period_assignment(&db,input.client_id.as_deref().unwrap_or(""),period_reference,Some(invoice_date))?;
    }

    if input.record_type == "import_cost_input" {
        let entry_kind = payload.get("entry_kind").and_then(|v| v.as_str()).unwrap_or("").trim();
        if !["Goods line", "Cost component", "calculation_result"].contains(&entry_kind) {
            return Err("Classify the import-cost record as a Goods line or Cost component.".into());
        }
        if input.status == "Reviewed" && !["administrator", "manager"].contains(&persona.as_str()) {
            return Err("Only the demo administrator or engagement manager may mark an import-cost record reviewed.".into());
        }
        if entry_kind == "calculation_result" {
            if payload.get("result_json").and_then(|v| v.as_str()).unwrap_or("").trim().is_empty() {
                return Err("A calculation result must retain its complete result snapshot.".into());
            }
        } else {
            let source_reference = payload.get("source_reference").and_then(|v| v.as_str()).unwrap_or("").trim();
            if input.status == "Ready for review" && source_reference.is_empty() {
                return Err("Add a source document/reference before submitting an import-cost input for review.".into());
            }
            if input.status == "Ready for review" {
                let source_date = payload.get("source_date").and_then(|v| v.as_str()).unwrap_or("").trim();
                if !valid_iso_date(source_date) { return Err("Record a valid ISO source document date before submitting an import-cost input for review.".into()); }
            }
            let read_integer = |key: &str| -> Result<i64, String> {
                let raw = payload.get(key).and_then(|v| v.as_str()).unwrap_or("").trim();
                raw.parse::<i64>().map_err(|_| format!("Import-cost field {key} must be a whole-number value."))
            };
            let currency = payload.get("currency_code").and_then(|v| v.as_str()).unwrap_or("").trim();
            if currency.len() != 3 || !currency.chars().all(|ch| ch.is_ascii_alphabetic()) {
                return Err("Enter a three-letter currency code for the source amount.".into());
            }
            if read_integer("fx_numerator")? <= 0 || read_integer("fx_denominator")? <= 0 {
                return Err("Record a positive FX numerator and denominator; use 1 and 1 when source and base currency are the same.".into());
            }
            if input.status == "Ready for review" {
                for key in ["fx_rate_source", "fx_rate_date", "fx_authorized_by"] {
                    if payload.get(key).and_then(|v| v.as_str()).unwrap_or("").trim().is_empty() {
                        return Err(format!("Record {key}; for base-currency 1:1 conversion, document that basis explicitly."));
                    }
                }
                let fx_date = payload.get("fx_rate_date").and_then(|v| v.as_str()).unwrap_or("").trim();
                if !valid_iso_date(fx_date) { return Err("FX rate date must be a valid ISO date.".into()); }
            }
            if entry_kind == "Goods line" {
                if payload.get("item_reference").and_then(|v| v.as_str()).unwrap_or("").trim().is_empty()
                    || read_integer("quantity_micros")? <= 0 || read_integer("unit_price_minor")? < 0 {
                    return Err("A goods line needs an item reference, positive quantity (millionths), and non-negative unit price (minor units).".into());
                }
                if read_integer("weight_micros").unwrap_or(0) < 0 || read_integer("volume_micros").unwrap_or(0) < 0 {
                    return Err("Weight and volume cannot be negative.".into());
                }
            } else {
                if payload.get("component_reference").and_then(|v| v.as_str()).unwrap_or("").trim().is_empty()
                    || payload.get("category").and_then(|v| v.as_str()).unwrap_or("").trim().is_empty()
                    || read_integer("amount_minor")? < 0 {
                    return Err("A cost component needs a reference, category, and non-negative amount in minor units.".into());
                }
                let treatment = payload.get("treatment").and_then(|v| v.as_str()).unwrap_or("");
                if !["Capitalize", "Exclude", "Recoverable tax", "Customs-value-only", "Needs review"].contains(&treatment) {
                    return Err("Select an explicit cost treatment; do not leave it implicit.".into());
                }
                let basis = payload.get("allocation_basis").and_then(|v| v.as_str()).unwrap_or("");
                if !["Direct", "Quantity", "Weight", "Volume", "Value"].contains(&basis) {
                    return Err("Select Direct, Quantity, Weight, Volume, or Value as the allocation basis.".into());
                }
                if treatment == "Capitalize" && basis == "Direct"
                    && payload.get("direct_item_reference").and_then(|v| v.as_str()).unwrap_or("").trim().is_empty() {
                    return Err("A directly allocated component needs its target item reference.".into());
                }
            }
        }
    }



    if input.record_type == "period" {
        let reference = input.reference.as_deref().unwrap_or("").trim();
        if input.status != "Draft" && reference.is_empty() {
            return Err("A reporting period needs a unique reference before it can be reviewed.".into());
        }
        if input.status == "Reviewed" && !["administrator","manager"].contains(&persona.as_str()) {
            return Err("Only the demo administrator or engagement manager may review a financial period.".into());
        }
        if input.status == "Ready for review" {
            for key in ["period_source_reference","calendar_policy_reference"] {
                if payload.get(key).and_then(|v| v.as_str()).unwrap_or("").trim().is_empty() {
                    return Err(format!("Record {key} before submitting the period for review."));
                }
            }
        }
        if input.status != "Draft" {
            let start = payload.get("start_date").and_then(|v| v.as_str()).unwrap_or("").trim();
            let end = payload.get("end_date").and_then(|v| v.as_str()).unwrap_or("").trim();
            if start.len() != 10 || end.len() != 10 || start > end {
                return Err("A reporting period requires valid ISO start and end dates, with start on or before end.".into());
            }
        }
        if !reference.is_empty() {
            let duplicate: bool = db.connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM workspace_record WHERE firm_id=?1 AND client_id=?2 AND record_type='period' AND reference=?3 AND id<>?4)",
                params![FIRM_ID,input.client_id.as_deref().unwrap_or(""),reference,input.id.as_deref().unwrap_or("")],
                |r| r.get(0)
            ).map_err(|e| e.to_string())?;
            if duplicate { return Err("This reporting period reference is already used by another period for this client.".into()); }
        }
    }
    if input.record_type == "historical_record" && input.status == "Ready for review" {
        let client_id = input.client_id.as_deref().unwrap_or("");
        let period_reference = payload.get("period_reference").and_then(|v| v.as_str()).unwrap_or("").trim();
        if period_reference.is_empty() {
            return Err("Assign the historical record to an explicitly reviewed financial period before review.".into());
        }
        let class = payload.get("historical_class").and_then(|v| v.as_str()).unwrap_or("");
        if ["Tax history","VAT carry-forward (source wording)"].contains(&class) {
            let tax_period = payload.get("tax_period_as_stated").and_then(|v| v.as_str()).unwrap_or("").trim();
            if tax_period.is_empty() { return Err("Preserve the tax period exactly as stated on the source.".into()); }
            validate_period_assignment(&db,client_id,period_reference,None)?;
        } else {
            let date = payload.get("transaction_date").and_then(|v| v.as_str()).unwrap_or("").trim();
            if date.is_empty() { return Err("Record the source transaction date before period assignment.".into()); }
            validate_period_assignment(&db,client_id,period_reference,Some(date))?;
        }
        let source_reference = payload.get("source_reference").and_then(|v| v.as_str()).unwrap_or("").trim();
        let has_attachment = if let Some(record_id) = input.id.as_deref() {
            db.connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM source_document WHERE firm_id=?1 AND client_id=?2 AND record_id=?3)",
                params![FIRM_ID,client_id,record_id], |r| r.get::<_,bool>(0)
            ).unwrap_or(false)
        } else { false };
        if source_reference.is_empty() && !has_attachment {
            return Err("Add a source document reference or attach the original evidence before review.".into());
        }
    }

    if input.record_type == "import_case" {
        let reference = input.reference.as_deref().unwrap_or("").trim();
        if input.status != "Discovery" && reference.is_empty() {
            return Err("Add a unique import case reference before moving the case beyond Discovery.".into());
        }
        if !reference.is_empty() {
            let duplicate: bool = db.connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM workspace_record WHERE firm_id=?1 AND client_id=?2 AND record_type='import_case' AND reference=?3 AND id<>?4)",
                params![FIRM_ID,input.client_id.as_deref().unwrap_or(""),reference,input.id.as_deref().unwrap_or("")],
                |r| r.get(0)
            ).map_err(|e| e.to_string())?;
            if duplicate { return Err("This import case reference is already used by another case for this client.".into()); }
        }
    }
    if input.record_type == "import_cost_input" {
        let reference = input.reference.as_deref().unwrap_or("").trim();
        if reference.is_empty() {
            return Err("Link each cost input to its import case using the exact case reference.".into());
        }
        let case_exists: bool = db.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM workspace_record WHERE firm_id=?1 AND client_id=?2 AND record_type='import_case' AND reference=?3)",
            params![FIRM_ID,input.client_id.as_deref().unwrap_or(""),reference],
            |r| r.get(0)
        ).map_err(|e| e.to_string())?;
        if !case_exists { return Err("The referenced import case does not exist in this client workspace.".into()); }
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

    if input.record_type == "import_cost_input" {
        if let Some((_, existing_type, _, existing_payload, _)) = exists.as_ref() {
            if existing_type != "import_cost_input" { return Err("Import-cost record type mismatch.".into()); }
            let previous: serde_json::Value = serde_json::from_str(existing_payload).unwrap_or_else(|_| json!({}));
            if previous.get("entry_kind").and_then(|v| v.as_str()) == Some("calculation_result") {
                return Err("Generated import-cost calculation snapshots are immutable. Create a new calculation version instead.".into());
            }
        }
    }
    if input.record_type == "invoice" {
        let prior_reviewer_note = exists.as_ref().map(|row| serde_json::from_str::<serde_json::Value>(&row.3).unwrap_or_else(|_|json!({})))
            .and_then(|value|value.get("reviewer_note").and_then(|v|v.as_str()).map(str::to_string)).unwrap_or_default();
        let next_reviewer_note = payload.get("reviewer_note").and_then(|v|v.as_str()).unwrap_or("").to_string();
        if prior_reviewer_note != next_reviewer_note {
            let previous_status = exists.as_ref().map(|row|row.2.as_str());
            if !["administrator","manager"].contains(&persona.as_str()) || previous_status != Some("Ready for review") {
                return Err("Only the assigned manager/admin review step may add or change the invoice reviewer note.".into());
            }
        }
        if ["Needs correction","Reviewed"].contains(&input.status.as_str()) && !["administrator","manager"].contains(&persona.as_str()) {
            return Err("Only the demo administrator or engagement manager can record an invoice review decision.".into());
        }
        if input.status == "Needs correction" && payload.get("reviewer_note").and_then(|v|v.as_str()).unwrap_or("").trim().chars().count() < 3 {
            return Err("Record the specific correction required before returning this invoice.".into());
        }
        if let Some((_,existing_type,previous_status,previous_payload,_)) = exists.as_ref() {
            if existing_type != "invoice" { return Err("Invoice record type mismatch.".into()); }
            if ["Ready for review","Reviewed"].contains(&previous_status.as_str())
                || (previous_status == "Needs correction" && persona != "auditor") {
                let previous: serde_json::Value = serde_json::from_str(previous_payload).unwrap_or_else(|_|json!({}));
                let mut previous_compare = previous.clone();
                let mut payload_compare = payload.clone();
                if previous_status == "Ready for review" {
                    if let Some(object) = previous_compare.as_object_mut() { object.remove("reviewer_note"); }
                    if let Some(object) = payload_compare.as_object_mut() { object.remove("reviewer_note"); }
                }
                let old_values: (String,String,String,String) = db.connection.query_row(
                    "SELECT title,reference,amount_text,notes FROM workspace_record WHERE id=?1 AND firm_id=?2",
                    params![id,FIRM_ID], |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))
                ).map_err(|e|e.to_string())?;
                if previous_compare != payload_compare || old_values.0 != input.title.trim()
                    || old_values.1 != input.reference.as_deref().unwrap_or("")
                    || old_values.2 != input.amount_text.as_deref().unwrap_or("")
                    || old_values.3 != input.notes.as_deref().unwrap_or("") {
                    return Err("Invoice source values are locked during review and after review. Return for correction or create a separately traceable revision.".into());
                }
            }
            if ["Needs correction","Reviewed"].contains(&input.status.as_str()) && previous_status != "Ready for review" {
                return Err("An invoice review decision can only be recorded from Ready for review.".into());
            }
            if previous_status == "Reviewed" && input.status != "Reviewed" {
                return Err("Reviewed invoices are immutable in this prototype.".into());
            }
        } else if ["Needs correction","Reviewed"].contains(&input.status.as_str()) {
            return Err("A new invoice must be captured before it can enter review.".into());
        }
    }
    if input.record_type == "historical_record" || input.record_type == "period" {
        if input.status == "Reviewed" && !["administrator","manager"].contains(&persona.as_str()) {
            return Err("Only the demo administrator or engagement manager can mark historical work reviewed.".into());
        }
        if let Some((_,existing_type,previous_status,previous_payload,_)) = exists.as_ref() {
            if existing_type != input.record_type { return Err("Workspace record type mismatch.".into()); }
            if ["Ready for review","Reviewed"].contains(&previous_status.as_str()) {
                let previous: serde_json::Value = serde_json::from_str(previous_payload).unwrap_or_else(|_|json!({}));
                if previous != payload {
                    return Err("Historical source records are locked while awaiting or after review; return to Needs source before correction.".into());
                }
                let old_values: (String,String,String,String) = db.connection.query_row(
                    "SELECT title,reference,amount_text,notes FROM workspace_record WHERE id=?1 AND firm_id=?2",
                    params![id,FIRM_ID], |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))
                ).map_err(|e|e.to_string())?;
                if old_values.0 != input.title.trim() || old_values.1 != input.reference.as_deref().unwrap_or("")
                    || old_values.2 != input.amount_text.as_deref().unwrap_or("") || old_values.3 != input.notes.as_deref().unwrap_or("") {
                    return Err("Historical source records cannot be edited while awaiting or after review.".into());
                }
            }
            if input.status == "Reviewed" && previous_status != "Ready for review" && previous_status != "Reviewed" {
                return Err("Historical work can be marked reviewed only from Ready for review.".into());
            }
            if previous_status == "Reviewed" && input.status != "Reviewed" {
                return Err("Reviewed historical records are immutable in this prototype.".into());
            }
        } else if input.status == "Reviewed" {
            return Err("A historical record must exist before it can be marked reviewed.".into());
        }
    }
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
        if input.status != "Draft" {
            let due_date = payload.get("due_date").and_then(|v|v.as_str()).unwrap_or("").trim();
            if !valid_iso_date(due_date) { return Err("Set a valid due date before assigning or progressing a task.".into()); }
            if reviewer.is_empty() || reviewer == assignee {
                return Err("Assign a separate active reviewer; the preparer cannot review their own task.".into());
            }
            for staff_name in [assignee, reviewer] {
                let active_staff: bool = db.connection.query_row(
                    "SELECT EXISTS(SELECT 1 FROM workspace_record WHERE firm_id=?1 AND record_type='staff' AND status='Active' AND title=?2)",
                    params![FIRM_ID,staff_name], |r| r.get(0)
                ).map_err(|e|e.to_string())?;
                if !active_staff { return Err("Both preparer and reviewer must be active staff in this demo firm.".into()); }
                let demo_identity: bool = db.connection.query_row(
                    "SELECT EXISTS(SELECT 1 FROM demo_actor WHERE display_name=?1)",
                    params![staff_name], |r| r.get(0)
                ).map_err(|e|e.to_string())?;
                if !demo_identity { return Err("Task assignees and reviewers must have a corresponding active demo identity so notifications and workspace routing are meaningful.".into()); }
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
                if reviewer.is_empty() || reviewer == assignee {
                    return Err("The new reviewer must be a different active staff member from the preparer.".into());
                }
                let reviewer_active: bool = db.connection.query_row(
                    "SELECT EXISTS(SELECT 1 FROM workspace_record WHERE firm_id=?1 AND record_type='staff' AND status='Active' AND title=?2)",
                    params![FIRM_ID,reviewer], |r|r.get(0)
                ).map_err(|e|e.to_string())?;
                if !reviewer_active { return Err("The new reviewer must be an active staff member.".into()); }
            } else if prior_reviewer == reviewer {
                let prior_reason = prior.get("reassignment_reason").and_then(|v|v.as_str()).unwrap_or("");
                let next_reason = payload.get("reassignment_reason").and_then(|v|v.as_str()).unwrap_or("");
                if prior_reason != next_reason { return Err("A reassignment reason can only change when the assignee or reviewer changes.".into()); }
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
                if previous_status == "Submitted" {
                    if let Some(object) = previous_compare.as_object_mut() { object.remove("reviewer"); object.remove("reassignment_reason"); }
                    if let Some(object) = payload_compare.as_object_mut() { object.remove("reviewer"); object.remove("reassignment_reason"); }
                    if input.status == "Cancelled" {
                        if let Some(object) = previous_compare.as_object_mut() { object.remove("cancellation_reason"); }
                        if let Some(object) = payload_compare.as_object_mut() { object.remove("cancellation_reason"); }
                    }
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

    if input.record_type == "task" {
        let client_id = input.client_id.as_deref().unwrap_or("");
        let client_label: String = tx.query_row(
            "SELECT COALESCE(NULLIF(display_name,''),legal_name) FROM client WHERE id=?1 AND firm_id=?2",
            params![client_id,FIRM_ID], |r| r.get(0)
        ).map_err(|e|e.to_string())?;
        let previous_payload: serde_json::Value = exists.as_ref()
            .map(|row| serde_json::from_str(&row.3).unwrap_or_else(|_|json!({})))
            .unwrap_or_else(||json!({}));
        let previous_assignee = previous_payload.get("assignee").and_then(|v|v.as_str()).unwrap_or("");
        let previous_reviewer = previous_payload.get("reviewer").and_then(|v|v.as_str()).unwrap_or("");
        let previous_status = exists.as_ref().map(|row|row.2.as_str()).unwrap_or("");
        let assignee = payload.get("assignee").and_then(|v|v.as_str()).unwrap_or("");
        let reviewer = payload.get("reviewer").and_then(|v|v.as_str()).unwrap_or("");
        let task_body = format!("{} · {}", input.title.trim(), client_label);
        let mut emit_notification = |recipient_name: &str, event_type: &str, title: &str, body: &str| -> Result<(),String> {
            if recipient_name.trim().is_empty() { return Ok(()); }
            let recipient_id: Option<String> = tx.query_row(
                "SELECT id FROM demo_actor WHERE display_name=?1",
                params![recipient_name], |r|r.get(0)
            ).optional().map_err(|e|e.to_string())?;
            if let Some(recipient_id) = recipient_id {
                let key = format!("{}:{}:{}:{}",id,event_type,recipient_id,new_version);
                tx.execute(
                    "INSERT OR IGNORE INTO workspace_notification(id,firm_id,recipient_actor_id,client_id,record_id,record_type,event_type,title,body,created_at,idempotency_key)
                     VALUES(?1,?2,?3,?4,?5,'task',?6,?7,?8,?9,?10)",
                    params![Uuid::new_v4().to_string(),FIRM_ID,recipient_id,client_id,id,event_type,title,body,now,key]
                ).map_err(|e|e.to_string())?;
            }
            Ok(())
        };
        if input.status != "Draft" && !assignee.is_empty() && (assignee != previous_assignee || (input.status == "Assigned" && previous_status == "Draft")) {
            emit_notification(assignee,"task_assigned","New task assigned",&task_body)?;
        }
        if input.status != "Draft" && !reviewer.is_empty() && (reviewer != previous_reviewer || (input.status == "Assigned" && previous_status == "Draft")) {
            emit_notification(reviewer,"review_assigned","Review responsibility assigned",&task_body)?;
        }
        if input.status == "Submitted" && previous_status != "Submitted" {
            emit_notification(reviewer,"task_submitted_for_review","Task submitted for review",&task_body)?;
        }
        if input.status == "Blocked" && previous_status != "Blocked" {
            emit_notification(reviewer,"task_blocked","Assigned task is blocked",&task_body)?;
        }
        if input.status == "Returned" && previous_status != "Returned" {
            emit_notification(assignee,"task_returned","Task returned for correction",&task_body)?;
        }
        if input.status == "Completed" && previous_status != "Completed" {
            emit_notification(assignee,"task_review_completed","Task review completed",&task_body)?;
        }
    }
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
    if record_type == "staff" && !["administrator","manager"].contains(&persona.as_str()) {
        return Err("Firm-level staff history is restricted to the demo administrator and manager.".into());
    }
    if record_type == "task" {
        let client_id = client_id.as_deref().ok_or_else(||"Task has no client workspace.".to_string())?;
        if !actor_has_client_access(&db,client_id,&actor_name,&persona)? {
            return Err("This task history is not assigned to your demo persona.".into());
        }
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


#[derive(Serialize, Clone)]
pub struct SourceDocumentMeta {
    id: String,
    record_id: String,
    file_name: String,
    media_type: String,
    size_bytes: i64,
    sha256: String,
    actor_name: String,
    created_at: String,
}

#[derive(Serialize)]
pub struct SourceDocumentContent {
    metadata: SourceDocumentMeta,
    content: Vec<u8>,
}

fn document_record_context(db: &Database, record_id: &str) -> Result<(String, String, String, String), String> {
    db.connection.query_row(
        "SELECT r.client_id,r.record_type,c.status,r.title
         FROM workspace_record r JOIN client c ON c.id=r.client_id
         WHERE r.id=?1 AND r.firm_id=?2 AND c.firm_id=?2",
        params![record_id,FIRM_ID],
        |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))
    ).map_err(|_| "The source document must be attached to an existing client invoice, historical record or import-cost input.".to_string())
}

fn detected_document_type(content: &[u8]) -> Option<&'static str> {
    if content.starts_with(b"%PDF-") { Some("application/pdf") }
    else if content.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) { Some("image/png") }
    else if content.len() >= 3 && content[0..3] == [0xFF, 0xD8, 0xFF] { Some("image/jpeg") }
    else { None }
}

#[tauri::command]
pub fn attach_source_document(
    client_id: String,
    record_id: String,
    file_name: String,
    media_type: String,
    content: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<SourceDocumentMeta, String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (actor_id, actor_name, persona) = ensure_can_mutate(&db)?;
    if content.is_empty() || content.len() > 5 * 1024 * 1024 {
        return Err("Choose a non-empty PDF, JPEG or PNG file no larger than 5 MB.".into());
    }
    let name = file_name.trim();
    if name.is_empty() || name.chars().count() > 180 || name.contains('/') || name.contains('\\') || name.chars().any(char::is_control) {
        return Err("The filename is invalid; paths and control characters are not accepted.".into());
    }
    let detected = detected_document_type(&content).ok_or_else(|| "The file content is not a supported PDF, JPEG or PNG document.".to_string())?;
    let supplied = media_type.trim().to_ascii_lowercase();
    if !supplied.is_empty() && supplied != detected {
        return Err("The file's declared type does not match its actual content.".into());
    }
    let extension = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    let expected_extension = match detected { "application/pdf" => "pdf", "image/jpeg" => "jpg", "image/png" => "png", _ => "" };
    if extension != expected_extension && !(detected == "image/jpeg" && extension == "jpeg") {
        return Err("The filename extension must match the detected file content.".into());
    }
    let (record_client_id, record_type, client_status, _) = document_record_context(&db, &record_id)?;
    let (record_status, record_payload): (String,String) = db.connection.query_row(
        "SELECT status,payload_json FROM workspace_record WHERE id=?1 AND firm_id=?2",
        params![record_id,FIRM_ID], |r|Ok((r.get(0)?,r.get(1)?))
    ).map_err(|e|e.to_string())?;
    if !["Draft","Needs source","Needs correction"].contains(&record_status.as_str()) {
        return Err("Return the record for correction or source completion before adding evidence; reviewed and review-queued evidence is locked.".into());
    }
    if record_type == "import_cost_input" {
        let payload: serde_json::Value = serde_json::from_str(&record_payload).unwrap_or_else(|_|json!({}));
        if payload.get("entry_kind").and_then(|v|v.as_str()) == Some("calculation_result") {
            return Err("Attach evidence to the source goods line or cost component, not to the immutable calculation snapshot.".into());
        }
    }
    if record_client_id != client_id || client_status != "Active" {
        return Err("The record must belong to the selected active client workspace.".into());
    }
    if !["invoice", "historical_record", "import_cost_input"].contains(&record_type.as_str()) {
        return Err("Source documents can be attached only to invoice, historical-record or import-cost input evidence in this prototype.".into());
    }
    if !actor_has_client_access(&db, &client_id, &actor_name, &persona)? {
        return Err("This demo persona does not have access to the selected client evidence.".into());
    }
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let checksum = format!("{:x}", hasher.finalize());
    let duplicate: bool = db.connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM source_document WHERE firm_id=?1 AND record_id=?2 AND sha256=?3)",
        params![FIRM_ID,record_id,checksum], |r| r.get(0)
    ).map_err(|e| e.to_string())?;
    if duplicate { return Err("This exact document is already attached to the selected record.".into()); }

    let id = Uuid::new_v4().to_string();
    let now = now_utc();
    let correlation_id = Uuid::new_v4().to_string();
    let tx = db.connection.unchecked_transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO source_document(id,firm_id,client_id,record_id,actor_id,file_name,media_type,size_bytes,sha256,content,created_at,correlation_id)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        params![id,FIRM_ID,client_id,record_id,actor_id,name,detected,content.len() as i64,checksum,content,now,correlation_id]
    ).map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json)
         VALUES(?1,?2,?3,'source_document_attached','source_document',?4,?5,?6,?7)",
        params![Uuid::new_v4().to_string(),FIRM_ID,actor_id,id,now,correlation_id,json!({"client_id":client_id,"record_id":record_id,"file_name":name,"media_type":detected,"size_bytes":content.len(),"sha256":checksum}).to_string()]
    ).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(SourceDocumentMeta { id, record_id, file_name:name.to_string(), media_type:detected.to_string(), size_bytes:content.len() as i64, sha256:checksum, actor_name, created_at:now })
}

#[tauri::command]
pub fn list_source_documents(record_id: String, state: State<'_, AppState>) -> Result<Vec<SourceDocumentMeta>, String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (client_id, record_type, client_status, _) = document_record_context(&db, &record_id)?;
    if client_status != "Active" || !["invoice","historical_record","import_cost_input"].contains(&record_type.as_str()) {
        return Err("Source evidence is unavailable for this record.".into());
    }
    let (_, actor_name, persona) = active_actor(&db)?;
    if !actor_has_client_access(&db, &client_id, &actor_name, &persona)? {
        return Err("This demo persona does not have access to the selected client evidence.".into());
    }
    let mut stmt = db.connection.prepare(
        "SELECT d.id,d.record_id,d.file_name,d.media_type,d.size_bytes,d.sha256,a.display_name,d.created_at
         FROM source_document d JOIN demo_actor a ON a.id=d.actor_id
         WHERE d.firm_id=?1 AND d.record_id=?2 ORDER BY d.created_at DESC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![FIRM_ID,record_id], |r| Ok(SourceDocumentMeta {
        id:r.get(0)?,record_id:r.get(1)?,file_name:r.get(2)?,media_type:r.get(3)?,size_bytes:r.get(4)?,sha256:r.get(5)?,actor_name:r.get(6)?,created_at:r.get(7)?
    })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_source_document(id: String, state: State<'_, AppState>) -> Result<SourceDocumentContent, String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (client_id, record_type, client_status, _) = db.connection.query_row(
        "SELECT d.client_id,r.record_type,c.status,r.title
         FROM source_document d JOIN workspace_record r ON r.id=d.record_id
         JOIN client c ON c.id=d.client_id
         WHERE d.id=?1 AND d.firm_id=?2 AND r.firm_id=?2 AND c.firm_id=?2",
        params![id,FIRM_ID], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))
    ).map_err(|_| "Source document not found in this demonstration firm.".to_string())?;
    if client_status != "Active" || !["invoice","historical_record","import_cost_input"].contains(&record_type.as_str()) {
        return Err("Source evidence is unavailable for this record.".into());
    }
    let (_, actor_name, persona) = active_actor(&db)?;
    if !actor_has_client_access(&db, &client_id, &actor_name, &persona)? {
        return Err("This demo persona does not have access to the selected client evidence.".into());
    }
    let (meta, content) = db.connection.query_row(
        "SELECT d.id,d.record_id,d.file_name,d.media_type,d.size_bytes,d.sha256,a.display_name,d.created_at,d.content
         FROM source_document d JOIN demo_actor a ON a.id=d.actor_id
         WHERE d.id=?1 AND d.firm_id=?2",
        params![id,FIRM_ID],
        |r| Ok((
            SourceDocumentMeta { id:r.get(0)?,record_id:r.get(1)?,file_name:r.get(2)?,media_type:r.get(3)?,size_bytes:r.get(4)?,sha256:r.get(5)?,actor_name:r.get(6)?,created_at:r.get(7)? },
            r.get::<_,Vec<u8>>(8)?
        ))
    ).map_err(|e| e.to_string())?;
    Ok(SourceDocumentContent { metadata:meta, content })
}

#[tauri::command]
pub fn list_events(state: State<'_, AppState>) -> Result<Vec<EventItem>, String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let mut stmt = db.connection.prepare("SELECT e.id,e.event_type,e.target_type,e.target_id,e.occurred_at_utc,a.display_name,e.payload_json FROM local_event e JOIN demo_actor a ON a.id=e.actor_id WHERE e.firm_id=?1 ORDER BY e.occurred_at_utc DESC LIMIT 250").map_err(|e|e.to_string())?;
    let rows = stmt.query_map([FIRM_ID], |r| Ok(EventItem { id:r.get(0)?,event_type:r.get(1)?,target_type:r.get(2)?,target_id:r.get(3)?,occurred_at_utc:r.get(4)?,actor_name:r.get(5)?,payload_json:r.get(6)? })).map_err(|e|e.to_string())?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())
}


#[derive(Serialize, Clone)]
pub struct WorkspaceNotification {
    id: String,
    client_id: String,
    record_id: String,
    record_type: String,
    event_type: String,
    title: String,
    body: String,
    created_at: String,
    read_at: Option<String>,
}

#[tauri::command]
pub fn list_workspace_notifications(state: State<'_, AppState>) -> Result<Vec<WorkspaceNotification>, String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (actor_id, actor_name, persona) = active_actor(&db)?;
    let mut stmt = db.connection.prepare(
        "SELECT id,client_id,record_id,record_type,event_type,title,body,created_at,read_at
         FROM workspace_notification
         WHERE firm_id=?1 AND recipient_actor_id=?2
         ORDER BY created_at DESC LIMIT 100"
    ).map_err(|e|e.to_string())?;
    let rows = stmt.query_map(params![FIRM_ID,actor_id], |r| Ok(WorkspaceNotification {
        id:r.get(0)?,client_id:r.get(1)?,record_id:r.get(2)?,record_type:r.get(3)?,
        event_type:r.get(4)?,title:r.get(5)?,body:r.get(6)?,created_at:r.get(7)?,read_at:r.get(8)?
    })).map_err(|e|e.to_string())?;
    let candidates = rows.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    let mut visible = Vec::new();
    for notification in candidates {
        if actor_has_client_access(&db,&notification.client_id,&actor_name,&persona)? {
            visible.push(notification);
        }
    }
    Ok(visible)
}

#[tauri::command]
pub fn mark_workspace_notification_read(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (actor_id,_,_) = active_actor(&db)?;
    let changed = db.connection.execute(
        "UPDATE workspace_notification SET read_at=?1
         WHERE id=?2 AND firm_id=?3 AND recipient_actor_id=?4 AND read_at IS NULL",
        params![now_utc(),id,FIRM_ID,actor_id]
    ).map_err(|e|e.to_string())?;
    if changed == 0 {
        let owned: bool = db.connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM workspace_notification WHERE id=?1 AND firm_id=?2 AND recipient_actor_id=?3)",
            params![id,FIRM_ID,actor_id], |r|r.get(0)
        ).map_err(|e|e.to_string())?;
        if !owned { return Err("Notification not found for the active demo user.".into()); }
    }
    Ok(())
}

#[tauri::command]
pub fn reset_demo_data(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.database.lock().map_err(|_| "Database lock unavailable".to_string())?;
    let (actor_id,_,persona) = ensure_can_mutate(&db)?;
    if persona != "administrator" { return Err("Only the demo firm administrator can reset synthetic workspace data.".into()); }
    let tx = db.connection.unchecked_transaction().map_err(|e|e.to_string())?;
    tx.execute_batch("DROP TRIGGER IF EXISTS task_review_note_no_delete; DROP TRIGGER IF EXISTS task_review_note_content_immutable; DROP TRIGGER IF EXISTS task_review_note_status_transition;
        DROP TRIGGER IF EXISTS client_change_no_update; DROP TRIGGER IF EXISTS client_change_no_delete;
        DROP TRIGGER IF EXISTS workspace_record_version_no_update; DROP TRIGGER IF EXISTS workspace_record_version_no_delete;
        DROP TRIGGER IF EXISTS historical_import_batch_no_update; DROP TRIGGER IF EXISTS historical_import_batch_no_delete; DROP TRIGGER IF EXISTS source_document_no_update; DROP TRIGGER IF EXISTS source_document_no_delete; DROP TRIGGER IF EXISTS workspace_notification_content_immutable; DROP TRIGGER IF EXISTS workspace_notification_read_once; DROP TRIGGER IF EXISTS workspace_notification_no_delete;").map_err(|e|e.to_string())?;
    tx.execute("DELETE FROM workspace_notification WHERE firm_id=?1",[FIRM_ID]).map_err(|e|e.to_string())?;
    tx.execute("DELETE FROM source_document WHERE firm_id=?1",[FIRM_ID]).map_err(|e|e.to_string())?;
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
        CREATE TRIGGER historical_import_batch_no_delete BEFORE DELETE ON historical_import_batch BEGIN SELECT RAISE(ABORT, 'historical import batches are immutable'); END;\n        CREATE TRIGGER source_document_no_update BEFORE UPDATE ON source_document BEGIN SELECT RAISE(ABORT, 'source document evidence is immutable'); END;\n        CREATE TRIGGER source_document_no_delete BEFORE DELETE ON source_document BEGIN SELECT RAISE(ABORT, 'source document evidence is immutable'); END;\n        CREATE TRIGGER workspace_notification_content_immutable BEFORE UPDATE ON workspace_notification WHEN NEW.id<>OLD.id OR NEW.firm_id<>OLD.firm_id OR NEW.recipient_actor_id<>OLD.recipient_actor_id OR NEW.client_id<>OLD.client_id OR NEW.record_id<>OLD.record_id OR NEW.record_type<>OLD.record_type OR NEW.event_type<>OLD.event_type OR NEW.title<>OLD.title OR NEW.body<>OLD.body OR NEW.created_at<>OLD.created_at OR NEW.idempotency_key<>OLD.idempotency_key BEGIN SELECT RAISE(ABORT, 'notification content is immutable'); END;\n        CREATE TRIGGER workspace_notification_read_once BEFORE UPDATE OF read_at ON workspace_notification WHEN OLD.read_at IS NOT NULL OR NEW.read_at IS NULL BEGIN SELECT RAISE(ABORT, 'notification read state can only be set once'); END;\n        CREATE TRIGGER workspace_notification_no_delete BEFORE DELETE ON workspace_notification BEGIN SELECT RAISE(ABORT, 'notification history is immutable'); END;").map_err(|e|e.to_string())?;
    tx.execute("INSERT INTO client(id,firm_id,record_kind,legal_name,display_name,status,created_at,updated_at,created_by,updated_by) VALUES(?1,?2,'Company','Sample Trading PLC','Sample Trading','Active',?3,?3,?4,?4)",
        params!["00000000-0000-4000-8000-000000000101",FIRM_ID,now_utc(),actor_id]).map_err(|e|e.to_string())?;
    tx.execute("INSERT INTO local_event(id,firm_id,actor_id,event_type,target_type,target_id,occurred_at_utc,correlation_id,payload_json) VALUES(?1,?2,?3,'demo_data_reset','demo_workspace',?2,?4,?5,'{}')",
        params![Uuid::new_v4().to_string(),FIRM_ID,actor_id,now_utc(),Uuid::new_v4().to_string()]).map_err(|e|e.to_string())?;
    tx.commit().map_err(|e|e.to_string())
}


#[cfg(test)]
mod tests {
    use super::*;



    #[test]
    fn reporting_dates_must_be_real_iso_calendar_dates() {
        assert!(valid_iso_date("2024-02-29"));
        assert!(valid_iso_date("2025-12-31"));
        assert!(!valid_iso_date("2025-02-29"));
        assert!(!valid_iso_date("2025-13-01"));
        assert!(!valid_iso_date("2025-04-31"));
        assert!(!valid_iso_date("2025/04/30"));
        assert!(!valid_iso_date("２０２５-04-30"));
    }

    #[test]
    fn source_document_type_uses_file_signature_not_declared_extension() {
        assert_eq!(detected_document_type(b"%PDF-1.7 synthetic"), Some("application/pdf"));
        assert_eq!(detected_document_type(&[0x89,b'P',b'N',b'G',0x0D,0x0A,0x1A,0x0A]), Some("image/png"));
        assert_eq!(detected_document_type(&[0xFF,0xD8,0xFF,0x00]), Some("image/jpeg"));
        assert_eq!(detected_document_type(b"<html>not an invoice</html>"), None);
    }

    #[test]
    fn invoice_and_tax_workflows_cannot_skip_review_or_claim_approval() {
        assert!(valid_record_transition("invoice", None, "Draft"));
        assert!(valid_record_transition("invoice", Some("Draft"), "Ready for review"));
        assert!(!valid_record_transition("invoice", None, "Reviewed"));
        assert!(!valid_record_transition("invoice", Some("Draft"), "Reviewed"));
        assert!(valid_record_transition("historical_record", Some("Ready for review"), "Reviewed"));
    }

    #[test]
    fn task_lifecycle_requires_submission_before_completion() {
        assert!(valid_record_transition("task", None, "Draft"));
        assert!(valid_record_transition("task", Some("Assigned"), "In progress"));
        assert!(valid_record_transition("task", Some("In progress"), "Submitted"));
        assert!(valid_record_transition("task", Some("Submitted"), "Completed"));
        assert!(valid_record_transition("task", Some("Submitted"), "Returned"));
        assert!(valid_record_transition("task", Some("Submitted"), "Cancelled"));
        assert!(valid_record_transition("task", Some("Returned"), "In progress"));
        assert!(!valid_record_transition("task", Some("In progress"), "Completed"));
        assert!(!valid_record_transition("task", Some("Completed"), "In progress"));
    }

    #[test]
    fn import_cost_calculation_input_cannot_bypass_discovery_gate() {
        assert!(valid_record_transition("import_case", None, "Discovery"));
        assert!(valid_record_transition("import_case", Some("Discovery"), "Inputs captured"));
        assert!(valid_record_transition("import_cost_input", None, "Draft"));
        assert!(!valid_record_transition("import_cost_input", None, "Ready for review"));
        assert!(valid_record_transition("import_cost_input", Some("Draft"), "Ready for review"));
    }

    #[test]
    fn client_workspace_access_requires_a_non_draft_task_assignment() {
        use crate::db::Database;
        let db = Database::memory().expect("in-memory database");
        let client_id = "00000000-0000-4000-8000-000000000101";
        assert!(!actor_has_client_access(&db,client_id,"Demo Auditor","auditor").expect("access check"));
        db.connection.execute(
            "INSERT INTO workspace_record(id,firm_id,client_id,record_type,title,status,payload_json,created_at,updated_at,created_by,updated_by)
             VALUES('access-task',?1,?2,'task','Assigned work','Draft','{\"assignee\":\"Demo Auditor\",\"reviewer\":\"Demo Engagement Manager\"}','now','now','00000000-0000-4000-8000-000000000012','00000000-0000-4000-8000-000000000012')",
            rusqlite::params![FIRM_ID,client_id],
        ).expect("insert draft task");
        assert!(!actor_has_client_access(&db,client_id,"Demo Auditor","auditor").expect("draft is not assigned"));
        db.connection.execute("UPDATE workspace_record SET status='Assigned' WHERE id='access-task'", []).expect("assign task");
        assert!(actor_has_client_access(&db,client_id,"Demo Auditor","auditor").expect("preparer access"));
        assert!(actor_has_client_access(&db,client_id,"Demo Engagement Manager","reviewer").expect("reviewer access"));
        assert!(!actor_has_client_access(&db,client_id,"Other Persona","auditor").expect("other persona denied"));
    }



    #[test]
    fn read_only_reviewer_is_rejected_by_the_shared_mutation_guard() {
        let db = crate::db::Database::memory().expect("seeded in-memory database");
        db.connection.execute(
            "UPDATE app_setting SET value=(SELECT id FROM demo_actor WHERE persona='reviewer') WHERE key='active_actor_id'",
            [],
        ).expect("select reviewer persona");
        assert!(ensure_can_mutate(&db).is_err());
    }

    #[test]
    fn manager_and_auditor_are_allowed_through_the_shared_mutation_guard() {
        let db = crate::db::Database::memory().expect("seeded in-memory database");
        for persona in ["manager", "auditor", "administrator"] {
            db.connection.execute(
                "UPDATE app_setting SET value=(SELECT id FROM demo_actor WHERE persona=?1) WHERE key='active_actor_id'",
                [persona],
            ).expect("select writable persona");
            assert_eq!(ensure_can_mutate(&db).unwrap().2, persona);
        }
    }

    #[test]
    fn period_assignment_requires_review_and_inclusive_valid_date_window() {
        let db = crate::db::Database::memory().expect("seeded in-memory database");
        let client_id: String = db.connection.query_row(
            "SELECT id FROM client WHERE firm_id=?1 AND status='Active' LIMIT 1",
            [FIRM_ID], |row| row.get(0)
        ).expect("seeded active client");
        db.connection.execute(
            "INSERT INTO workspace_record(id,firm_id,client_id,record_type,title,status,reference,payload_json,created_at,updated_at,created_by,updated_by)
             VALUES('period-test',?1,?2,'period','FY 2026','Reviewed','FY2026','{\"start_date\":\"2026-01-01\",\"end_date\":\"2026-12-31\"}','now','now','00000000-0000-4000-8000-000000000011','00000000-0000-4000-8000-000000000011')",
            params![FIRM_ID,client_id],
        ).unwrap();
        assert!(validate_period_assignment(&db,&client_id,"FY2026",Some("2026-01-01")).is_ok());
        assert!(validate_period_assignment(&db,&client_id,"FY2026",Some("2026-12-31")).is_ok());
        assert!(validate_period_assignment(&db,&client_id,"FY2026",Some("2025-12-31")).is_err());
        assert!(validate_period_assignment(&db,&client_id,"FY2026",Some("2027-01-01")).is_err());
        assert!(validate_period_assignment(&db,&client_id,"FY2026",Some("2026-02-30")).is_err());
        assert!(validate_period_assignment(&db,&client_id,"MISSING",Some("2026-06-01")).is_err());
    }

    #[test]
    fn record_type_allowlist_and_initial_state_guards_reject_untrusted_values() {
        for allowed in ["category","product","invoice","historical_record","period","staff","task","import_case","import_cost_input"] {
            assert!(allowed_record_type(allowed), "expected {allowed} to be allowed");
        }
        for rejected in ["users","client; DROP TABLE client","../client","", "invoice\\u0000"] {
            assert!(!allowed_record_type(rejected), "expected {rejected:?} to be rejected");
        }
        assert!(!allowed_status("invoice","Filed"));
        assert!(!valid_record_transition("invoice",None,"Reviewed"));
        assert!(!valid_record_transition("task",Some("In progress"),"Completed"));
        assert!(!valid_record_transition("task",Some("Completed"),"In progress"));
    }

    #[test]
    fn client_workspace_access_is_scoped_to_assigned_persona_and_non_draft_task() {
        let db = crate::db::Database::memory().expect("seeded in-memory database");
        let client_id: String = db.connection.query_row(
            "SELECT id FROM client WHERE firm_id=?1 AND status='Active' LIMIT 1",
            [FIRM_ID], |row| row.get(0)
        ).expect("seeded active client");
        assert!(!actor_has_client_access(&db,&client_id,"Demo Auditor","auditor").unwrap());
        db.connection.execute(
            "INSERT INTO workspace_record(id,firm_id,client_id,record_type,title,status,payload_json,created_at,updated_at,created_by,updated_by)
             VALUES('access-task',?1,?2,'task','Assigned work','Draft','{\"assignee\":\"Demo Auditor\",\"reviewer\":\"Demo Engagement Manager\"}','now','now','00000000-0000-4000-8000-000000000012','00000000-0000-4000-8000-000000000012')",
            params![FIRM_ID,client_id],
        ).unwrap();
        assert!(!actor_has_client_access(&db,&client_id,"Demo Auditor","auditor").unwrap());
        db.connection.execute("UPDATE workspace_record SET status='Assigned' WHERE id='access-task'", []).unwrap();
        assert!(actor_has_client_access(&db,&client_id,"Demo Auditor","auditor").unwrap());
        assert!(actor_has_client_access(&db,&client_id,"Demo Engagement Manager","reviewer").unwrap());
        assert!(!actor_has_client_access(&db,&client_id,"Other Persona","auditor").unwrap());
        assert!(actor_has_client_access(&db,"another-client","Other Persona","manager").unwrap());
    }

    #[test]
    fn all_workspace_record_types_enforce_their_declared_state_machines() {
        let valid = [
            ("category", None, "Draft"),
            ("category", Some("Draft"), "Active"),
            ("category", Some("Active"), "Archived"),
            ("product", None, "Draft"),
            ("product", Some("Draft"), "Active"),
            ("product", Some("Active"), "Archived"),
            ("invoice", None, "Draft"),
            ("invoice", Some("Draft"), "Ready for review"),
            ("invoice", Some("Ready for review"), "Needs correction"),
            ("invoice", Some("Ready for review"), "Reviewed"),
            ("historical_record", None, "Draft"),
            ("historical_record", Some("Draft"), "Needs source"),
            ("historical_record", Some("Ready for review"), "Reviewed"),
            ("period", None, "Draft"),
            ("period", Some("Draft"), "Ready for review"),
            ("period", Some("Ready for review"), "Reviewed"),
            ("staff", None, "Draft"),
            ("staff", Some("Draft"), "Active"),
            ("staff", Some("Active"), "Inactive"),
            ("staff", Some("Inactive"), "Active"),
            ("task", None, "Draft"),
            ("task", Some("Draft"), "Assigned"),
            ("task", Some("Assigned"), "In progress"),
            ("task", Some("In progress"), "Blocked"),
            ("task", Some("Blocked"), "In progress"),
            ("task", Some("In progress"), "Submitted"),
            ("task", Some("Submitted"), "Returned"),
            ("task", Some("Returned"), "In progress"),
            ("task", Some("Submitted"), "Completed"),
            ("task", Some("Submitted"), "Cancelled"),
            ("import_case", None, "Discovery"),
            ("import_case", Some("Discovery"), "Inputs captured"),
            ("import_case", Some("Inputs captured"), "Needs review"),
            ("import_case", Some("Needs review"), "Reviewed"),
            ("import_case", Some("Reviewed"), "Deferred"),
            ("import_cost_input", None, "Draft"),
            ("import_cost_input", Some("Draft"), "Needs source"),
            ("import_cost_input", Some("Draft"), "Ready for review"),
            ("import_cost_input", Some("Ready for review"), "Reviewed"),
        ];
        for (record_type, previous, next) in valid {
            assert!(valid_record_transition(record_type, previous, next), "expected {record_type}: {previous:?} -> {next} to be valid");
            assert!(allowed_status(record_type, next), "expected {next} to be an allowed {record_type} status");
        }

        let invalid = [
            ("category", Some("Draft"), "Archived"),
            ("product", Some("Archived"), "Active"),
            ("invoice", None, "Reviewed"),
            ("invoice", Some("Reviewed"), "Draft"),
            ("historical_record", Some("Draft"), "Reviewed"),
            ("period", Some("Draft"), "Reviewed"),
            ("staff", Some("Inactive"), "Draft"),
            ("task", Some("In progress"), "Completed"),
            ("task", Some("Completed"), "In progress"),
            ("import_case", Some("Discovery"), "Reviewed"),
            ("import_cost_input", None, "Ready for review"),
            ("import_cost_input", Some("Reviewed"), "Draft"),
        ];
        for (record_type, previous, next) in invalid {
            assert!(!valid_record_transition(record_type, previous, next), "expected {record_type}: {previous:?} -> {next} to be rejected");
        }
    }

    #[test]
    fn historical_import_row_validation_rejects_duplicates_empty_values_and_invalid_row_numbers() {
        let make_row = |source_row, description: &str, amount_text: &str| HistoricalImportRow {
            source_row, transaction_date: "2026-01-01".into(), description: description.into(),
            amount_text: amount_text.into(), currency: "ETB".into(), supplier: "Synthetic supplier".into(),
            supplier_tin: "".into(), vat_amount: "".into(), invoice_reference: "".into(),
        };
        assert!(validate_historical_import_rows(&[make_row(2,"Purchase","100")]).is_ok());
        assert!(validate_historical_import_rows(&[make_row(2,"Purchase","100"),make_row(2,"Second","200")]).is_err());
        assert!(validate_historical_import_rows(&[make_row(1,"Purchase","100")]).is_err());
        assert!(validate_historical_import_rows(&[make_row(2," ","100")]).is_err());
        assert!(validate_historical_import_rows(&[make_row(2,"Purchase"," ")]).is_err());
        assert!(validate_historical_import_rows(&[]).is_err());
    }

}
