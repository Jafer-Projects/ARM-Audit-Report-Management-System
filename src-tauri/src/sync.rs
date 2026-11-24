use crate::{db::{now_utc,Database},AppState};
use rusqlite::{params,OptionalExtension};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use tauri::State;

const FIRM_ID:&str="00000000-0000-4000-8000-000000000001";

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct SyncOutboxEvent {
 event_id:String,idempotency_key:String,aggregate_type:String,aggregate_id:String,base_version:i64,
 event_type:String,schema_version:i64,occurred_at:String,correlation_id:String,payload:Value,
}
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct SyncAcknowledgement {
 event_id:String,status:String,version:Option<i64>,detail:Option<Value>,
}
#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
pub struct PulledSyncEvent {
 received_seq:String,event_id:String,aggregate_id:String,base_version:i64,resulting_version:Option<i64>,
 outcome:String,payload:Value,received_at:Option<String>,
}

#[tauri::command]
pub fn sync_configure(cloud_tenant_id:String,state:State<'_,AppState>)->Result<(),String>{
 let id=cloud_tenant_id.trim();
 if uuid::Uuid::parse_str(id).is_err(){return Err("A valid cloud tenant UUID is required.".into());}
 let db=state.database.lock().map_err(|_|"Database lock unavailable".to_string())?;
 let existing:Option<String>=db.connection.query_row("SELECT cloud_tenant_id FROM cloud_sync_state WHERE firm_id=?1",[FIRM_ID],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
 if let Some(current)=existing { if current!=id { return Err("This local workspace is already paired with a different cloud tenant. Use a separate local profile to switch tenants.".into()); } }
 db.connection.execute(
  "INSERT INTO cloud_sync_state(firm_id,cloud_tenant_id,pull_cursor,updated_at) VALUES(?1,?2,'0',?3) ON CONFLICT(firm_id) DO UPDATE SET updated_at=excluded.updated_at",
  params![FIRM_ID,id,now_utc()]
 ).map_err(|e|e.to_string())?;
 Ok(())
}

#[tauri::command]
pub fn sync_outbox_batch(limit:Option<i64>,state:State<'_,AppState>)->Result<Vec<SyncOutboxEvent>,String>{
 let db=state.database.lock().map_err(|_|"Database lock unavailable".to_string())?;
 let max=limit.unwrap_or(50).clamp(1,100);
 let mut stmt=db.connection.prepare(
  "SELECT event_id,idempotency_key,aggregate_type,aggregate_id,base_version,event_type,schema_version,occurred_at,correlation_id,payload_json FROM sync_outbox WHERE status='pending' ORDER BY created_at LIMIT ?1"
 ).map_err(|e|e.to_string())?;
 let rows=stmt.query_map([max],|row|{
  let payload:String=row.get(9)?;
  Ok(SyncOutboxEvent{
   event_id:row.get(0)?,idempotency_key:row.get(1)?,aggregate_type:row.get(2)?,aggregate_id:row.get(3)?,
   base_version:row.get(4)?,event_type:row.get(5)?,schema_version:row.get(6)?,occurred_at:row.get(7)?,
   correlation_id:row.get(8)?,payload:serde_json::from_str(&payload).unwrap_or(Value::Null)
  })
 }).map_err(|e|e.to_string())?;
 rows.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())
}

#[tauri::command]
pub fn sync_ack_outbox(results:Vec<SyncAcknowledgement>,state:State<'_,AppState>)->Result<(),String>{
 if results.len()>100{return Err("At most 100 synchronization acknowledgements may be applied.".into());}
 let db=state.database.lock().map_err(|_|"Database lock unavailable".to_string())?;
 let tx=db.connection.unchecked_transaction().map_err(|e|e.to_string())?;
 for result in results{
  if !["accepted","conflict","rejected"].contains(&result.status.as_str()){return Err("Unsupported synchronization result.".into());}
  let detail=result.detail.unwrap_or(Value::Null);
  let aggregate:Option<String>=tx.query_row("SELECT aggregate_id FROM sync_outbox WHERE event_id=?1",[&result.event_id],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
  tx.execute("UPDATE sync_outbox SET status=?1,server_version=?2,response_json=?3,attempts=attempts+1,updated_at=?4 WHERE event_id=?5",
   params![result.status,result.version,detail.to_string(),now_utc(),result.event_id]
  ).map_err(|e|e.to_string())?;
  if let Some(aggregate_id)=aggregate {
   let server_version=if result.status=="accepted" { result.version } else if result.status=="conflict" { detail.get("expectedVersion").and_then(Value::as_i64) } else { None };
   if let Some(version)=server_version {
    tx.execute("INSERT INTO cloud_sync_aggregate_state(aggregate_id,server_version,updated_at) VALUES(?1,?2,?3) ON CONFLICT(aggregate_id) DO UPDATE SET server_version=excluded.server_version,updated_at=excluded.updated_at WHERE excluded.server_version>=cloud_sync_aggregate_state.server_version",
     params![aggregate_id,version,now_utc()]
    ).map_err(|e|e.to_string())?;
   }
  }
 }
 tx.commit().map_err(|e|e.to_string())
}

#[tauri::command]
pub fn sync_apply_pull(events:Vec<PulledSyncEvent>,next_cursor:String,state:State<'_,AppState>)->Result<usize,String>{
 if events.len()>100{return Err("At most 100 pulled events may be applied.".into());}
 if next_cursor.len()>19 || !next_cursor.chars().all(|c|c.is_ascii_digit()){return Err("Invalid synchronization cursor.".into());}
 let db=state.database.lock().map_err(|_|"Database lock unavailable".to_string())?;
 let tx=db.connection.unchecked_transaction().map_err(|e|e.to_string())?;
 let actor:String=tx.query_row("SELECT value FROM app_setting WHERE key='active_actor_id'",[],|r|r.get(0)).map_err(|e|e.to_string())?;
 let configured:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM cloud_sync_state WHERE firm_id=?1)",[FIRM_ID],|r|r.get(0)).map_err(|e|e.to_string())?;
 if !configured{return Err("Configure the cloud tenant before applying pull events.".into());}
 let mut applied=0usize;
 for event in events{
  if !["accepted","conflict","rejected"].contains(&event.outcome.as_str()){return Err("Unsupported pulled event outcome.".into());}
  tx.execute("INSERT OR IGNORE INTO cloud_sync_inbox(received_seq,event_id,aggregate_id,outcome,resulting_version,payload_json,received_at) VALUES(?1,?2,?3,?4,?5,?6,?7)",
   params![event.received_seq,event.event_id,event.aggregate_id,event.outcome,event.resulting_version,event.payload.to_string(),event.received_at.clone().unwrap_or_else(now_utc)]
  ).map_err(|e|e.to_string())?;
  if event.outcome=="accepted"{
   if let (Some(record),Some(version))=(event.payload.get("record"),event.resulting_version){
    let current_cloud_version:Option<i64>=tx.query_row("SELECT server_version FROM cloud_sync_aggregate_state WHERE aggregate_id=?1",[&event.aggregate_id],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
    let has_pending:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM sync_outbox WHERE aggregate_id=?1 AND status IN ('pending','conflict'))",[&event.aggregate_id],|r|r.get(0)).map_err(|e|e.to_string())?;
    if version>current_cloud_version.unwrap_or(0) && !has_pending {
     let record_id=record.get("id").and_then(Value::as_str).unwrap_or(&event.aggregate_id);
     let record_type=record.get("record_type").and_then(Value::as_str).unwrap_or("historical_record");
     let title=record.get("title").and_then(Value::as_str).unwrap_or("Synced record");
     let status=record.get("status").and_then(Value::as_str).unwrap_or("Draft");
     let client_id=record.get("client_id").and_then(Value::as_str);
     let client_exists=if let Some(client)=client_id{
      tx.query_row("SELECT 1 FROM client WHERE id=?1 AND firm_id=?2",params![client,FIRM_ID],|_|Ok(())).optional().map_err(|e|e.to_string())?.is_some()
     }else{true};
     if !client_exists { return Err("Remote workspace record references a client that is not present locally; synchronize the client directory before applying this cursor.".into()); }
     {
      let reference=record.get("reference").and_then(Value::as_str).unwrap_or("");
      let amount=record.get("amount_text").and_then(Value::as_str).unwrap_or("");
      let notes=record.get("notes").and_then(Value::as_str).unwrap_or("");
      let payload_json=record.get("payload_json").and_then(Value::as_str).unwrap_or("{}");
      let updated=record.get("updated_at").and_then(Value::as_str).unwrap_or(event.received_at.as_deref().unwrap_or("1970-01-01T00:00:00Z"));
      let source_version=record.get("row_version").and_then(Value::as_i64).unwrap_or(1).max(1);
      let existing_version:Option<i64>=tx.query_row("SELECT row_version FROM workspace_record WHERE id=?1 AND firm_id=?2",params![record_id,FIRM_ID],|r|r.get(0)).optional().map_err(|e|e.to_string())?;
      let local_version=existing_version.map(|v|v.saturating_add(1)).unwrap_or(source_version);
      tx.execute(
       "INSERT INTO workspace_record(id,firm_id,client_id,record_type,title,status,reference,amount_text,notes,payload_json,row_version,created_at,updated_at,created_by,updated_by) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?12,?13,?13) ON CONFLICT(id) DO UPDATE SET title=excluded.title,status=excluded.status,reference=excluded.reference,amount_text=excluded.amount_text,notes=excluded.notes,payload_json=excluded.payload_json,row_version=excluded.row_version,updated_at=excluded.updated_at,updated_by=excluded.updated_by WHERE workspace_record.firm_id=excluded.firm_id",
       params![record_id,FIRM_ID,client_id,record_type,title,status,reference,amount,notes,payload_json,local_version,updated,actor]
      ).map_err(|e|e.to_string())?;
      let snapshot=json!({"recordType":record_type,"title":title,"status":status,"reference":reference,"amountText":amount,"notes":notes,"payload":payload_json,"updatedAt":updated,"sourceEventId":event.event_id});
      tx.execute(
       "INSERT OR IGNORE INTO workspace_record_version(id,firm_id,record_id,row_version,actor_id,changed_at,snapshot_json,correlation_id) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
       params![uuid::Uuid::new_v4().to_string(),FIRM_ID,record_id,local_version,actor,updated,snapshot.to_string(),event.event_id]
      ).map_err(|e|e.to_string())?;
      applied+=1;
     }
    }
    tx.execute("INSERT INTO cloud_sync_aggregate_state(aggregate_id,server_version,updated_at) VALUES(?1,?2,?3) ON CONFLICT(aggregate_id) DO UPDATE SET server_version=MAX(server_version,excluded.server_version),updated_at=excluded.updated_at",[&event.aggregate_id,version,now_utc()]).map_err(|e|e.to_string())?;
   }
  }
 }
 tx.execute("UPDATE cloud_sync_state SET pull_cursor=CASE WHEN CAST(?2 AS INTEGER)>CAST(pull_cursor AS INTEGER) THEN ?2 ELSE pull_cursor END,last_success_at=?3,updated_at=?3 WHERE firm_id=?1",
  params![FIRM_ID,next_cursor,now_utc()]
 ).map_err(|e|e.to_string())?;
 tx.commit().map_err(|e|e.to_string())?;
 Ok(applied)
}

#[tauri::command]
pub fn sync_status(state:State<'_,AppState>)->Result<Value,String>{
 let db=state.database.lock().map_err(|_|"Database lock unavailable".to_string())?;
 let pending:i64=db.connection.query_row("SELECT count(*) FROM sync_outbox WHERE status IN ('pending','conflict')",[],|r|r.get(0)).map_err(|e|e.to_string())?;
 let conflicts:i64=db.connection.query_row("SELECT count(*) FROM sync_outbox WHERE status='conflict'",[],|r|r.get(0)).map_err(|e|e.to_string())?;
 let state:Option<(String,String)>=db.connection.query_row("SELECT cloud_tenant_id,pull_cursor FROM cloud_sync_state WHERE firm_id=?1",[FIRM_ID],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|e|e.to_string())?;
 Ok(serde_json::json!({"configured":state.is_some(),"cloudTenantId":state.as_ref().map(|x|x.0.clone()),"cursor":state.as_ref().map(|x|x.1.clone()).unwrap_or_else(||"0".into()),"pending":pending,"conflicts":conflicts}))
}
