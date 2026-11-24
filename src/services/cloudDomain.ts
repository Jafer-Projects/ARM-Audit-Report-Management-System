import { secureSession } from "../secureSession";

export type Engagement={id:string;client_reference:string;title:string;period_start:string;period_end:string;framework:string;status:string;created_at:string};
export type Workpaper={id:string;engagement_id:string;reference:string;title:string;procedure:string;conclusion:string;status:string;version:number;prepared_by:string|null;reviewed_by:string|null;approved_by:string|null;created_at:string;updated_at:string};
export type AuditReport={id:string;engagement_id:string;report_type:string;title:string;status:string;version:number;prepared_by:string|null;reviewed_by:string|null;approved_by:string|null;issued_at:string|null;created_at:string;updated_at:string};
export type Account={id:string;account_code:string;account_name:string;account_type:string;currency:string;active:boolean};
export type Journal={id:string;journal_number:string;effective_date:string;description:string;currency:string;status:string;total_debit:number;total_credit:number;created_by:string;posted_by:string|null;reversal_of:string|null;posted_at:string|null;line_count:number};
export type JournalDetail=Journal&{lines:Array<{line_number:number;account_id:string;account_code:string;account_name:string;description:string;debit:number;credit:number}>};
export type DocumentMeta={id:string;file_name:string;media_type:string;size_bytes:number;sha256:string;scan_status:string;scan_detail?:string;engagement_id:string|null;workpaper_id:string|null;report_id:string|null;created_at:string;scanned_at:string|null};

async function request<T>(path:string,init:RequestInit={}):Promise<T>{
 const saved=await secureSession.load();
 if(!saved||!saved.selectedTenantId)throw new Error("Authenticate and select an organization before opening audit or accounting records.");
 const base=new URL(saved.apiBaseUrl);
 if(base.protocol!=="https:"&&!(base.protocol==="http:"&&["localhost","127.0.0.1"].includes(base.hostname)))throw new Error("Cloud API requires HTTPS except local development.");
 const root=base.toString().replace(/\/$/,"");
 const send=(token:string)=>fetch(root+path,{...init,headers:{"authorization":"Bearer "+token,"x-tenant-id":saved.selectedTenantId,"content-type":"application/json",...init.headers},cache:"no-store"});
 let response=await send(saved.session.accessToken);
 if(response.status===401){
  try{const updated=await secureSession.refresh();response=await send(updated.session.accessToken);}
  catch(error){if(error instanceof Error&&/Cloud identity API (401|403)/.test(error.message))await secureSession.clear();throw new Error("Cloud domain request could not refresh its session; stored credentials are retained when the device is offline.",{cause:error});}
 }
 if(!response.ok){const body=await response.text();throw new Error("Cloud domain API returned "+response.status+": "+body.slice(0,500));}
 if(response.status===204)return undefined as T;
 return response.json() as Promise<T>;
}
export const cloudDomain={
 engagements:async()=> (await request<{engagements:Engagement[]}>("/v1/audit/engagements")).engagements,
 createEngagement:(input:{clientReference:string;title:string;periodStart:string;periodEnd:string;framework:string})=>request<Engagement>("/v1/audit/engagements",{method:"POST",body:JSON.stringify(input)}),
 workpapers:async(engagementId?:string)=> (await request<{workpapers:Workpaper[]}>("/v1/audit/workpapers"+(engagementId?"?engagementId="+encodeURIComponent(engagementId):""))).workpapers,
 createWorkpaper:(input:{engagementId:string;reference:string;title:string;procedure:string})=>request<Workpaper>("/v1/audit/workpapers",{method:"POST",body:JSON.stringify(input)}),
 transitionWorkpaper:(id:string,expectedVersion:number,action:"prepare"|"review"|"approve"|"reopen",conclusion?:string)=>request<Workpaper>("/v1/audit/workpapers/"+encodeURIComponent(id)+"/transition",{method:"POST",body:JSON.stringify({expectedVersion,action,conclusion})}),
 reports:async(engagementId?:string)=> (await request<{reports:AuditReport[]}>("/v1/audit/reports"+(engagementId?"?engagementId="+encodeURIComponent(engagementId):""))).reports,
 createReport:(input:{engagementId:string;reportType:"independent_auditor"|"management_letter"|"internal_audit"|"special_purpose";title:string;body:Record<string,unknown>})=>request<AuditReport>("/v1/audit/reports",{method:"POST",body:JSON.stringify(input)}),
 transitionReport:(id:string,expectedVersion:number,action:"prepare"|"review"|"approve"|"issue"|"withdraw")=>request<AuditReport>("/v1/audit/reports/"+encodeURIComponent(id)+"/transition",{method:"POST",body:JSON.stringify({expectedVersion,action})}),
 accounts:async()=> (await request<{accounts:Account[]}>("/v1/accounting/accounts")).accounts,
 createAccount:(input:{code:string;name:string;type:"asset"|"liability"|"equity"|"income"|"expense";currency:string})=>request<Account>("/v1/accounting/accounts",{method:"POST",body:JSON.stringify(input)}),
 journals:async()=> (await request<{journals:Journal[]}>("/v1/accounting/journals")).journals,
 journal:(id:string)=>request<JournalDetail>("/v1/accounting/journals/"+encodeURIComponent(id)),
 createJournal:(input:{journalNumber:string;date:string;description:string;currency:string;lines:Array<{accountId:string;description?:string;debit:number;credit:number}>})=>request<Journal>("/v1/accounting/journals",{method:"POST",body:JSON.stringify(input)}),
 postJournal:(id:string)=>request<Journal>("/v1/accounting/journals/"+encodeURIComponent(id)+"/post",{method:"POST"}),
 reverseJournal:(id:string,input:{journalNumber:string;date:string;description:string})=>request<Journal>("/v1/accounting/journals/"+encodeURIComponent(id)+"/reverse",{method:"POST",body:JSON.stringify(input)}),
 documents:async(engagementId?:string)=> (await request<{documents:DocumentMeta[]}>("/v1/documents"+(engagementId?"?engagementId="+encodeURIComponent(engagementId):""))).documents,
 uploadDocument:(input:{fileName:string;mediaType:"application/pdf"|"image/jpeg"|"image/png";contentBase64:string;engagementId?:string;workpaperId?:string;reportId?:string})=>request<{id:string;scan_status:string;created_at:string}>("/v1/documents",{method:"POST",body:JSON.stringify(input)}),
 readDocument:(id:string)=>request<{id:string;original_filename:string;media_type:string;size_bytes:number;sha256:string;scan_status:string;contentBase64:string}>("/v1/documents/"+encodeURIComponent(id)+"/content")
};
