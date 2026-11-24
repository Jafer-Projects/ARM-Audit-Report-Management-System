import { secureSession } from "../secureSession";
import type { Client, ClientChange, ClientInput, DuplicateCandidate } from "../types";

type CloudClient = Omit<Client,"record_kind"|"status"> & {record_kind:"business"|"individual"|"other";status:"active"|"inactive"|"archived"};
type CloudHistory = {id:string;action:string;actor_id:string;before_data:unknown;after_data:unknown;client_version:number;occurred_at:string};
type CloudDuplicate = {id:string;record_kind:"business"|"individual"|"other";legal_name:string;tin_raw:string;status:"active"|"inactive"|"archived"};

function normalizeBase(value:string){
 const url=new URL(value);
 if(url.protocol!=="https:"&&!(url.protocol==="http:"&&["localhost","127.0.0.1"].includes(url.hostname))) throw new Error("Cloud API requires HTTPS except local development.");
 if(url.username||url.password||url.search||url.hash) throw new Error("Cloud API address cannot contain credentials, query parameters or fragments.");
 return url.toString().replace(/\/$/,"");
}
function mapClient(value:CloudClient):Client{
 return {...value,record_kind:value.record_kind==="business"?"Company":value.record_kind==="individual"?"Individual":"Other organization",status:value.status==="active"?"Active":value.status==="inactive"?"Inactive":"Archived"};
}
async function context(){
 const saved=await secureSession.load();
 if(!saved) throw new Error("No cloud session is active.");
 if(!saved.selectedTenantId) throw new Error("Select an active organization before using its client directory.");
 return {saved,base:normalizeBase(saved.apiBaseUrl),tenantId:saved.selectedTenantId};
}
async function request<T>(path:string,init:RequestInit={}):Promise<T>{
 const current=await context();
 const send=(accessToken:string)=>fetch(current.base+path,{...init,headers:{"authorization":"Bearer "+accessToken,"x-tenant-id":current.tenantId,"content-type":"application/json",...init.headers},cache:"no-store"});
 let response=await send(current.saved.session.accessToken);
 if(response.status===401){
  try{
   const updated=await secureSession.refresh();
   response=await send(updated.session.accessToken);
  }catch(error){
   if(error instanceof Error&&/Cloud identity API (401|403)/.test(error.message))await secureSession.clear();
   throw new Error("Cloud request could not refresh its session. If the device is offline, the saved credentials are retained for retry.",{cause:error});
  }
 }
 if(!response.ok){const body=await response.text();throw new Error("Cloud client directory returned "+response.status+": "+body.slice(0,400));}
 if(response.status===204)return undefined as T;
 return response.json() as Promise<T>;
}
function serverKind(value:string):"business"|"individual"|"other"{
 const normalized=value.trim().toLowerCase();
 if(["company","business"].includes(normalized))return "business";
 if(normalized==="individual")return "individual";
 return "other";
}

export const cloudClients={
 list:async(includeArchived=false):Promise<Client[]>=>{
  const result=await request<{clients:CloudClient[]}>("/v1/clients?includeArchived="+String(includeArchived));
  return result.clients.map(mapClient);
 },
 duplicates:async(legalName:string,tinRaw:string,excludeId?:string):Promise<DuplicateCandidate[]>=>{
  const query=new URLSearchParams({legalName,tinRaw});if(excludeId)query.set("excludeId",excludeId);
  const result=await request<{candidates:CloudDuplicate[]}>("/v1/clients/duplicates?"+query.toString());
  return result.candidates.map(item=>({id:item.id,record_kind:item.record_kind==="business"?"Company":item.record_kind==="individual"?"Individual":"Other organization",legal_name:item.legal_name,tin_raw:item.tin_raw,status:item.status==="active"?"Active":item.status==="inactive"?"Inactive":"Archived"}));
 },
 save:async(input:ClientInput):Promise<Client>=>{
  const body={...input,record_kind:serverKind(input.record_kind)};
  return mapClient(await request<CloudClient>("/v1/clients",{method:"POST",body:JSON.stringify(body)}));
 },
 transition:async(id:string,action:"archive"|"activate"|"restore"):Promise<void>=>{
  const clients=await cloudClients.list(true);const client=clients.find(item=>item.id===id);
  if(!client)throw new Error("Client not found in the selected organization.");
  await request<CloudClient>("/v1/clients/"+encodeURIComponent(id)+"/transition",{method:"POST",body:JSON.stringify({action,expectedVersion:client.row_version})});
 },
 history:async(id:string):Promise<ClientChange[]>=>{
  const result=await request<{history:CloudHistory[]}>("/v1/clients/"+encodeURIComponent(id)+"/history");
  return result.history.map(item=>({id:item.id,action:item.action,changed_at:item.occurred_at,actor_name:item.actor_id,before_json:item.before_data==null?"":JSON.stringify(item.before_data),after_json:item.after_data==null?"":JSON.stringify(item.after_data)}));
 },
 workspace:async()=>{
  const {saved}=await context();
  const tenant=await request<{name?:string;role?:string;permissions?:string[]}>("/v1/tenant/context");
  const role=(tenant.role??"").toLowerCase();
  const persona=(role==="owner"||role==="admin"||role==="administrator"?"administrator":role==="manager"?"manager":role==="auditor"?"auditor":"reviewer") as "administrator"|"manager"|"auditor"|"reviewer";
  const permissions=tenant.permissions??[];
  const clients=permissions.includes("clients.read")?await cloudClients.list(false):[];
  return {firm_name:tenant.name??"Cloud organization",actor_name:saved.session.user?.displayName??saved.session.user?.email??"Authenticated user",persona,clients_count:clients.length,sync_state:"Cloud authoritative; client offline cache and outbox are not enabled",permissions};
 }
};
