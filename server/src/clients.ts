import { randomUUID } from "node:crypto";
import type { PoolClient } from "pg";
import { withTenant } from "./db.js";
import { assertPermission } from "./authorization.js";

export type ClientInput = {
 id?:string; row_version?:number; record_kind:"business"|"individual"|"other";
 legal_name:string; display_name:string; tin_raw?:string; sector?:string;
 contact_name?:string; contact_email?:string; contact_phone?:string;
 address_text?:string; internal_reference?:string; notes?:string;
 duplicate_acknowledged?:boolean;
};
const fields=["record_kind","legal_name","display_name","tin_raw","sector","contact_name","contact_email","contact_phone","address_text","internal_reference","notes"] as const;
function snapshot(row:Record<string,unknown>){return Object.fromEntries(fields.map(key=>[key,row[key]??""]));}
async function recordHistory(client:PoolClient,tenantId:string,userId:string,id:string,action:string,before:unknown,after:Record<string,unknown>,version:number){
 await client.query("INSERT INTO client_history(tenant_id,client_id,actor_id,action,before_data,after_data,client_version) VALUES($1,$2,$3,$4,$5::jsonb,$6::jsonb,$7)",[tenantId,id,userId,action,before?JSON.stringify(before):null,JSON.stringify(after),version]);
 await client.query("INSERT INTO audit_events(tenant_id,actor_id,action,target_type,target_id,metadata) VALUES($1,$2,$3,'client',$4,$5::jsonb)",[tenantId,userId,"client."+action,id,JSON.stringify({version})]);
}
export async function listClients(userId:string,tenantId:string,includeArchived:boolean){
 return withTenant(userId,tenantId,async(client)=>{
  await assertPermission(client,userId,tenantId,"clients.read");
  const result=await client.query("SELECT id,record_kind,legal_name,display_name,tin_raw,sector,contact_name,contact_email,contact_phone,address_text,internal_reference,notes,status,row_version,updated_at FROM clients WHERE tenant_id=$1 AND ($2::boolean OR status<>'archived') ORDER BY lower(display_name),id",[tenantId,includeArchived]);
  return result.rows;
 });
}
export async function findClientDuplicates(userId:string,tenantId:string,legalName:string,tinRaw:string,excludeId?:string){
 return withTenant(userId,tenantId,async(client)=>{
  await assertPermission(client,userId,tenantId,"clients.read");
  const result=await client.query("SELECT id,record_kind,legal_name,tin_raw,status FROM clients WHERE tenant_id=$1 AND ($4::uuid IS NULL OR id<>$4) AND ((lower(regexp_replace(btrim(legal_name),'\\s+',' ','g'))=lower(regexp_replace(btrim($2),'\\s+',' ','g'))) OR (btrim($3)<>'' AND lower(btrim(tin_raw))=lower(btrim($3)))) ORDER BY lower(legal_name) LIMIT 20",[tenantId,legalName,tinRaw,excludeId??null]);
  return result.rows;
 });
}
export async function saveClient(userId:string,tenantId:string,input:ClientInput){
 return withTenant(userId,tenantId,async(client)=>{
  await assertPermission(client,userId,tenantId,"clients.write");
  const legal=input.legal_name.trim(),display=input.display_name.trim();
  if(legal.length<2||display.length<1) throw Object.assign(new Error("Legal and display names are required"),{statusCode:400});
  const duplicate=await client.query("SELECT id FROM clients WHERE tenant_id=$1 AND ($2::uuid IS NULL OR id<>$2) AND ((lower(regexp_replace(btrim(legal_name),'\\s+',' ','g'))=lower(regexp_replace(btrim($3),'\\s+',' ','g'))) OR (btrim($4)<>'' AND lower(btrim(tin_raw))=lower(btrim($4)))) LIMIT 1",[tenantId,input.id??null,legal,input.tin_raw??""]);
  if(duplicate.rowCount&&!input.duplicate_acknowledged) throw Object.assign(new Error("Possible duplicate client exists; acknowledge the duplicate before saving a separate record"),{statusCode:409,code:"possible_duplicate"});
  const values=[input.record_kind,legal,display,input.tin_raw??"",input.sector??"",input.contact_name??"",input.contact_email??"",input.contact_phone??"",input.address_text??"",input.internal_reference??"",input.notes??""];
  if(!input.id){
   const id=randomUUID();
   const result=await client.query("INSERT INTO clients(id,tenant_id,record_kind,legal_name,display_name,tin_raw,sector,contact_name,contact_email,contact_phone,address_text,internal_reference,notes,created_by,updated_by) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$14) RETURNING *",[id,tenantId,...values,userId]);
   await recordHistory(client,tenantId,userId,id,"created",null,snapshot(result.rows[0]),1);
   return result.rows[0];
  }
  if(!Number.isSafeInteger(input.row_version)||!input.row_version||input.row_version<1) throw Object.assign(new Error("A valid row_version is required for updates"),{statusCode:400});
  const found=await client.query("SELECT * FROM clients WHERE tenant_id=$1 AND id=$2 FOR UPDATE",[tenantId,input.id]);
  if(!found.rowCount) throw Object.assign(new Error("Client not found"),{statusCode:404});
  const before=found.rows[0];
  if(Number(before.row_version)!==input.row_version) throw Object.assign(new Error("Client version conflict"),{statusCode:409,code:"version_conflict",currentVersion:Number(before.row_version)});
  const result=await client.query("UPDATE clients SET record_kind=$3,legal_name=$4,display_name=$5,tin_raw=$6,sector=$7,contact_name=$8,contact_email=$9,contact_phone=$10,address_text=$11,internal_reference=$12,notes=$13,row_version=row_version+1,updated_by=$14,updated_at=now() WHERE tenant_id=$1 AND id=$2 RETURNING *",[tenantId,input.id,...values,userId]);
  await recordHistory(client,tenantId,userId,input.id,"updated",snapshot(before),snapshot(result.rows[0]),Number(result.rows[0].row_version));
  return result.rows[0];
 });
}
export async function transitionClient(userId:string,tenantId:string,id:string,action:"archive"|"activate"|"restore",expectedVersion:number){
 return withTenant(userId,tenantId,async(client)=>{
  await assertPermission(client,userId,tenantId,action==="archive"?"clients.archive":"clients.write");
  const found=await client.query("SELECT * FROM clients WHERE tenant_id=$1 AND id=$2 FOR UPDATE",[tenantId,id]);
  if(!found.rowCount) throw Object.assign(new Error("Client not found"),{statusCode:404});
  const before=found.rows[0];
  if(Number(before.row_version)!==expectedVersion) throw Object.assign(new Error("Client version conflict"),{statusCode:409});
  const next=action==="archive"?"archived":"active";
  if(action==="archive"&&before.status==="archived") throw Object.assign(new Error("Client is already archived"),{statusCode:409});
  if(action==="restore"&&before.status!=="archived") throw Object.assign(new Error("Only archived clients can be restored"),{statusCode:409});
  if(action==="activate"&&before.status!=="inactive") throw Object.assign(new Error("Only inactive clients can be activated"),{statusCode:409});
  const result=await client.query("UPDATE clients SET status=$3,row_version=row_version+1,updated_by=$4,updated_at=now() WHERE tenant_id=$1 AND id=$2 RETURNING *",[tenantId,id,next,userId]);
  const historyAction=action==="archive"?"archived":action==="activate"?"activated":"restored";
  await recordHistory(client,tenantId,userId,id,historyAction,snapshot(before),snapshot(result.rows[0]),Number(result.rows[0].row_version));
  return result.rows[0];
 });
}
export async function listClientHistory(userId:string,tenantId:string,id:string){
 return withTenant(userId,tenantId,async(client)=>{
  await assertPermission(client,userId,tenantId,"clients.read");
  const exists=await client.query("SELECT 1 FROM clients WHERE tenant_id=$1 AND id=$2",[tenantId,id]);
  if(!exists.rowCount) throw Object.assign(new Error("Client not found"),{statusCode:404});
  const result=await client.query("SELECT h.id,h.action,h.actor_id,u.display_name AS actor_name,before_data,after_data,client_version,occurred_at FROM client_history h JOIN users u ON u.id=h.actor_id WHERE h.tenant_id=$1 AND h.client_id=$2 ORDER BY h.occurred_at DESC,h.id DESC",[tenantId,id]);
  return result.rows;
 });
}
