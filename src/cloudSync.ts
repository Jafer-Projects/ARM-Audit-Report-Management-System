import { invoke } from "@tauri-apps/api/core";

type SyncStatus={configured:boolean;cloudTenantId?:string;cursor:string;pending:number;conflicts:number};
type PushResult={eventId:string;status:"accepted"|"conflict"|"rejected";version:number|null;detail:Record<string,unknown>|null;duplicate:boolean};
type PullEvent={
 received_seq:string|number;event_id:string;aggregate_id:string;base_version:number;
 resulting_version:number|null;outcome:"accepted"|"conflict"|"rejected";
 payload:Record<string,unknown>;received_at:string;
};

function validateApiBaseUrl(value:string):string{
 const url=new URL(value);
 if(url.protocol!=="https:" && !(url.protocol==="http:" && ["localhost","127.0.0.1"].includes(url.hostname))) {
  throw new Error("Cloud synchronization requires HTTPS (HTTP is allowed only for local development).");
 }
 if(url.username||url.password||url.search||url.hash) throw new Error("API base URL must not contain credentials, query parameters or fragments.");
 return url.toString().replace(/\/$/,"");
}
async function api<T>(base:string,token:string,tenantId:string,path:string,init:RequestInit={}):Promise<T>{
 const response=await fetch(`${base}${path}`,{
  ...init,
  headers:{
   "authorization":`Bearer ${token}`,
   "x-tenant-id":tenantId,
   "content-type":"application/json",
   ...init.headers
  },
  cache:"no-store"
 });
 if(!response.ok) {
  const body=await response.text();
  throw new Error(`Cloud API returned ${response.status}: ${body.slice(0,500)}`);
 }
 if(response.status===204) return undefined as T;
 return response.json() as Promise<T>;
}

export async function getLocalSyncStatus():Promise<SyncStatus>{
 return invoke<SyncStatus>("sync_status");
}

/**
 * Sends only the version-1 prototype workspace-record envelope. Domain-specific
 * audit/accounting records use their dedicated API operations and are not
 * serialized through the generic outbox.
 */
export async function synchronizeDesktop(input:{apiBaseUrl:string;tenantId:string;accessToken:string}){
 const base=validateApiBaseUrl(input.apiBaseUrl);
 await invoke("sync_configure",{cloudTenantId:input.tenantId});
 const pending=await invoke<Array<Record<string,unknown>>>("sync_outbox_batch",{limit:100});
 let pushed:PushResult[]=[];
 if(pending.length){
  const result=await api<{results:PushResult[]}>(base,input.accessToken,input.tenantId,"/v1/sync/events",{
   method:"POST",body:JSON.stringify({events:pending})
  });
  pushed=result.results;
  await invoke("sync_ack_outbox",{results:pushed});
 }
 const before=await getLocalSyncStatus();
 const pulled=await api<{events:PullEvent[];nextCursor:string|null}>(base,input.accessToken,input.tenantId,
  `/v1/sync/events?after=${encodeURIComponent(before.cursor)}&limit=100`,{method:"GET",headers:{}});
 const events=pulled.events.map((event)=>({
  receivedSeq:String(event.received_seq),
  eventId:event.event_id,
  aggregateId:event.aggregate_id,
  baseVersion:event.base_version,
  resultingVersion:event.resulting_version,
  outcome:event.outcome,
  payload:event.payload,
  receivedAt:event.received_at
 }));
 if(pulled.nextCursor!==null) {
  await invoke("sync_apply_pull",{events,nextCursor:String(pulled.nextCursor)});
 }
 return {pushed:pushed.length,pulled:events.length,status:await getLocalSyncStatus()};
}
