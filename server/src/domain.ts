import { randomUUID } from "node:crypto";
import { withTenant } from "./db.js";
import { assertPermission } from "./authorization.js";

type EngagementInput = { clientReference:string; title:string; periodStart:string; periodEnd:string; framework:string };
async function history(client: any,tenantId:string,userId:string,type:string,id:string,action:string,before:unknown,after:unknown) {
  await client.query("INSERT INTO audit_domain_history(tenant_id,actor_id,entity_type,entity_id,action,before_data,after_data) VALUES($1,$2,$3,$4,$5,$6::jsonb,$7::jsonb)",[tenantId,userId,type,id,action,before?JSON.stringify(before):null,after?JSON.stringify(after):null]);
  await client.query("INSERT INTO audit_events(tenant_id,actor_id,action,target_type,target_id,metadata) VALUES($1,$2,$3,$4,$5,$6::jsonb)",[tenantId,userId,`domain.${type}.${action}`,type,id,JSON.stringify({before,after})]);
}

export async function createEngagement(userId:string,tenantId:string,input:EngagementInput) {
  return withTenant(userId,tenantId,async(client)=>{
    await assertPermission(client,userId,tenantId,"engagements.manage");
    const result=await client.query(
      "INSERT INTO audit_engagements(tenant_id,client_reference,title,period_start,period_end,framework,created_by,updated_by) VALUES($1,$2,$3,$4,$5,$6,$7,$7) RETURNING *",
      [tenantId,input.clientReference,input.title,input.periodStart,input.periodEnd,input.framework,userId]
    );
    await history(client,tenantId,userId,"engagement",result.rows[0].id,"created",null,result.rows[0]);
    return result.rows[0];
  });
}

export async function listEngagements(userId:string,tenantId:string) {
  return withTenant(userId,tenantId,async(client)=>{
    await assertPermission(client,userId,tenantId,"engagements.read");
    const rows=await client.query("SELECT * FROM audit_engagements WHERE tenant_id=$1 ORDER BY created_at DESC",[tenantId]);
    return rows.rows;
  });
}

export async function createWorkpaper(userId:string,tenantId:string,input:{engagementId:string;reference:string;title:string;procedure:string}) {
  return withTenant(userId,tenantId,async(client)=>{
    await assertPermission(client,userId,tenantId,"workpapers.write");
    const result=await client.query(
      "INSERT INTO audit_workpapers(tenant_id,engagement_id,reference,title,procedure,prepared_by) VALUES($1,$2,$3,$4,$5,$6) RETURNING *",
      [tenantId,input.engagementId,input.reference,input.title,input.procedure,userId]
    );
    await history(client,tenantId,userId,"workpaper",result.rows[0].id,"created",null,result.rows[0]);
    return result.rows[0];
  });
}

export async function transitionWorkpaper(userId:string,tenantId:string,id:string,expectedVersion:number,action:"prepare"|"review"|"approve"|"reopen",conclusion?:string) {
  return withTenant(userId,tenantId,async(client)=>{
    const permission=action==="review"||action==="approve"?"reports.approve":"workpapers.write";
    await assertPermission(client,userId,tenantId,permission);
    const found=await client.query("SELECT * FROM audit_workpapers WHERE tenant_id=$1 AND id=$2 FOR UPDATE",[tenantId,id]);
    const row=found.rows[0];
    if(!row) throw Object.assign(new Error("Workpaper not found"),{statusCode:404});
    if(Number(row.version)!==expectedVersion) throw Object.assign(new Error("Workpaper version conflict"),{statusCode:409});
    const before={...row};
    if(action==="prepare" && !["draft","reopened"].includes(row.status)) throw Object.assign(new Error("Only draft or reopened workpapers can be prepared"),{statusCode:409});
    if(action==="review" && row.status!=="prepared") throw Object.assign(new Error("Only prepared workpapers can be reviewed"),{statusCode:409});
    if(action==="approve" && (row.status!=="reviewed" || row.prepared_by===userId || row.reviewed_by===userId)) throw Object.assign(new Error("Approval requires independent preparation and review"),{statusCode:403});
    if(action==="reopen" && !["reviewed","approved"].includes(row.status)) throw Object.assign(new Error("Only reviewed or approved workpapers can be reopened"),{statusCode:409});
    const next=action==="prepare"?"prepared":action==="review"?"reviewed":action==="approve"?"approved":"reopened";
    const updated=await client.query(
      `UPDATE audit_workpapers SET status=$3,version=version+1,conclusion=COALESCE($4,conclusion),
       reviewed_by=CASE WHEN $3='reviewed' THEN $5 WHEN $3='reopened' THEN NULL ELSE reviewed_by END,
       approved_by=CASE WHEN $3='approved' THEN $5 WHEN $3='reopened' THEN NULL ELSE approved_by END,updated_at=now()
       WHERE tenant_id=$1 AND id=$2 RETURNING *`,
      [tenantId,id,next,conclusion??null,userId]
    );
    await history(client,tenantId,userId,"workpaper",id,action,before,updated.rows[0]);
    return updated.rows[0];
  });
}

export async function createAuditReport(userId:string,tenantId:string,input:{engagementId:string;reportType:string;title:string;body:Record<string,unknown>}) {
  return withTenant(userId,tenantId,async(client)=>{
    await assertPermission(client,userId,tenantId,"reports.prepare");
    const result=await client.query(
      "INSERT INTO audit_reports(tenant_id,engagement_id,report_type,title,body,prepared_by) VALUES($1,$2,$3,$4,$5::jsonb,$6) RETURNING *",
      [tenantId,input.engagementId,input.reportType,input.title,JSON.stringify(input.body),userId]
    );
    await history(client,tenantId,userId,"audit_report",result.rows[0].id,"created",null,result.rows[0]);
    return result.rows[0];
  });
}

export async function transitionAuditReport(userId:string,tenantId:string,id:string,expectedVersion:number,action:"prepare"|"review"|"approve"|"issue"|"withdraw") {
  return withTenant(userId,tenantId,async(client)=>{
    await assertPermission(client,userId,tenantId,action==="review"||action==="approve"||action==="issue"?"reports.approve":"reports.prepare");
    const found=await client.query("SELECT * FROM audit_reports WHERE tenant_id=$1 AND id=$2 FOR UPDATE",[tenantId,id]);
    const row=found.rows[0];
    if(!row) throw Object.assign(new Error("Audit report not found"),{statusCode:404});
    if(Number(row.version)!==expectedVersion) throw Object.assign(new Error("Audit report version conflict"),{statusCode:409});
    const before={...row};
    const transitions:Record<string,string[]>={prepare:["draft"],review:["prepared"],approve:["under_review"],issue:["approved"],withdraw:["draft","prepared","under_review","approved"]};
    if(!transitions[action].includes(row.status)) throw Object.assign(new Error("Invalid report workflow transition"),{statusCode:409});
    if((action==="approve"||action==="issue") && row.prepared_by===userId) throw Object.assign(new Error("The preparer cannot approve or issue their own report"),{statusCode:403});
    if(action==="approve" && (!row.reviewed_by || row.reviewed_by===userId)) throw Object.assign(new Error("Independent review is required before approval"),{statusCode:403});
    const next=action==="prepare"?"prepared":action==="review"?"under_review":action==="approve"?"approved":action==="issue"?"issued":"withdrawn";
    const updated=await client.query(
      `UPDATE audit_reports SET status=$3,version=version+1,
       reviewed_by=CASE WHEN $3='under_review' THEN $4 ELSE reviewed_by END,
       approved_by=CASE WHEN $3='approved' THEN $4 ELSE approved_by END,
       issued_at=CASE WHEN $3='issued' THEN now() ELSE issued_at END,updated_at=now()
       WHERE tenant_id=$1 AND id=$2 RETURNING *`,
      [tenantId,id,next,userId]
    );
    await history(client,tenantId,userId,"audit_report",id,action,before,updated.rows[0]);
    return updated.rows[0];
  });
}

export async function createJournal(userId:string,tenantId:string,input:{journalNumber:string;date:string;description:string;currency:string;lines:Array<{accountId:string;description?:string;debit:number;credit:number}>}) {
  return withTenant(userId,tenantId,async(client)=>{
    await assertPermission(client,userId,tenantId,"accounting.post");
    if(input.lines.length<2 || input.lines.length>500) throw Object.assign(new Error("A journal requires between 2 and 500 lines"),{statusCode:400});
    const debit=input.lines.reduce((sum,line)=>sum+line.debit,0),credit=input.lines.reduce((sum,line)=>sum+line.credit,0);
    if(!Number.isFinite(debit)||!Number.isFinite(credit)||debit<=0||Math.abs(debit-credit)>0.00005) throw Object.assign(new Error("Journal debits and credits must balance and be positive"),{statusCode:400});
    const accounts=await client.query("SELECT id FROM accounting_accounts WHERE tenant_id=$1 AND id=ANY($2::uuid[]) AND active=true AND currency=$3",[tenantId,input.lines.map(x=>x.accountId),input.currency]);
    if(accounts.rowCount!==new Set(input.lines.map(x=>x.accountId)).size) throw Object.assign(new Error("One or more accounts are inactive, missing or use a different currency"),{statusCode:400});
    const id=randomUUID();
    const journal=await client.query(
      "INSERT INTO accounting_journals(id,tenant_id,journal_number,effective_date,description,currency,total_debit,total_credit,created_by) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9) RETURNING *",
      [id,tenantId,input.journalNumber,input.date,input.description,input.currency,debit,credit,userId]
    );
    for(let i=0;i<input.lines.length;i++){
      const line=input.lines[i];
      await client.query("INSERT INTO accounting_journal_lines(tenant_id,journal_id,account_id,line_number,description,debit,credit) VALUES($1,$2,$3,$4,$5,$6,$7)",[tenantId,id,line.accountId,i+1,line.description??"",line.debit,line.credit]);
    }
    await history(client,tenantId,userId,"accounting_journal",id,"draft_created",null,journal.rows[0]);
    return journal.rows[0];
  });
}

export async function postJournal(userId:string,tenantId:string,id:string) {
  return withTenant(userId,tenantId,async(client)=>{
    await assertPermission(client,userId,tenantId,"accounting.post");
    const found=await client.query("SELECT * FROM accounting_journals WHERE tenant_id=$1 AND id=$2 FOR UPDATE",[tenantId,id]);
    const row=found.rows[0];
    if(!row || row.status!=="draft") throw Object.assign(new Error("Only an existing draft journal can be posted"),{statusCode:409});
    if(row.created_by===userId) throw Object.assign(new Error("Journal preparer cannot post their own journal"),{statusCode:403});
    const totals=await client.query("SELECT count(*)::int AS line_count,coalesce(sum(debit),0) AS debit,coalesce(sum(credit),0) AS credit FROM accounting_journal_lines WHERE tenant_id=$1 AND journal_id=$2",[tenantId,id]);
    if(totals.rows[0].line_count<2 || Number(totals.rows[0].debit)<=0 || Math.abs(Number(totals.rows[0].debit)-Number(totals.rows[0].credit))>0.00005) throw Object.assign(new Error("Journal is not balanced"),{statusCode:409});
    const result=await client.query("UPDATE accounting_journals SET status='posted',posted_by=$3,posted_at=now() WHERE tenant_id=$1 AND id=$2 RETURNING *",[tenantId,id,userId]);
    await history(client,tenantId,userId,"accounting_journal",id,"posted",row,result.rows[0]);
    return result.rows[0];
  });
}
