import { randomUUID } from "node:crypto";
import { pool, inTransaction } from "./db.js";
import { config } from "./config.js";
import { scanNextDocument } from "./documents.js";

const workerId=`arm-worker-${randomUUID()}`;
const sleep=(ms:number)=>new Promise((resolve)=>setTimeout(resolve,ms));
type Job={id:string;tenant_id:string|null;job_type:string;payload:Record<string,unknown>;attempts:number};
type Email={id:string;recipient:string;template_key:string;payload:Record<string,unknown>;attempts:number};

async function claimJob():Promise<Job|null>{
 return inTransaction(async(client)=>{
  await client.query("SELECT set_config('app.worker','true',true)");
  const found=await client.query(`SELECT id,tenant_id,job_type,payload,attempts FROM jobs
   WHERE (status='queued' AND available_at<=now()) OR (status='running' AND (locked_at IS NULL OR locked_at<now()-interval '5 minutes'))
   ORDER BY available_at,created_at FOR UPDATE SKIP LOCKED LIMIT 1`);
  const job=found.rows[0] as Job|undefined;
  if(!job) return null;
  await client.query("UPDATE jobs SET status='running',attempts=attempts+1,locked_at=now(),locked_by=$2 WHERE id=$1",[job.id,workerId]);
  return {...job,attempts:job.attempts+1};
 });
}
async function claimEmail():Promise<Email|null>{
 return inTransaction(async(client)=>{
  await client.query("SELECT set_config('app.worker','true',true)");
  const found=await client.query("SELECT id,recipient,template_key,payload,attempts FROM email_outbox WHERE status='queued' AND available_at<=now() AND attempts<8 ORDER BY created_at FOR UPDATE SKIP LOCKED LIMIT 1");
  const row=found.rows[0] as Email|undefined;
  if(!row) return null;
  await client.query("UPDATE email_outbox SET attempts=attempts+1,available_at=now()+interval '5 minutes' WHERE id=$1",[row.id]);
  return {...row,attempts:row.attempts+1};
 });
}
function escapeHtml(value:string){return value.replace(/[&<>"']/g,(c)=>({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;"}[c]??c));}
function renderEmail(row:Email){
 const payload=row.payload;
 const url=typeof payload.verificationUrl==="string"?payload.verificationUrl:typeof payload.resetUrl==="string"?payload.resetUrl:typeof payload.invitationUrl==="string"?payload.invitationUrl:"";
 const templates:Record<string,{subject:string;intro:string;action:string}>={
  email_verification:{subject:"Verify your ARM account",intro:"Confirm your email address to activate your ARM account.",action:"Verify email"},
  password_recovery:{subject:"Reset your ARM password",intro:"Use this single-use link to reset your password. It expires in 30 minutes.",action:"Reset password"},
  organization_invitation:{subject:"You are invited to an ARM organization",intro:"An organization administrator invited you to join their ARM workspace.",action:"Accept invitation"}
 };
 const template=templates[row.template_key];
 if(!template||!url) throw new Error("Unsupported email template or missing action URL");
 return {subject:template.subject,html:`<main><h1>${template.subject}</h1><p>${template.intro}</p><p><a href="${escapeHtml(url)}">${template.action}</a></p><p>If you did not request this, you can ignore this email.</p></main>`,text:`${template.intro} ${url} If you did not request this, ignore this email.`};
}
async function sendEmail(row:Email){
 if(!config.EMAIL_PROVIDER_URL||!config.EMAIL_PROVIDER_API_KEY) throw new Error("Email provider URL and API key are not configured");
 const message=renderEmail(row);
 const response=await fetch(config.EMAIL_PROVIDER_URL,{method:"POST",headers:{"authorization":`Bearer ${config.EMAIL_PROVIDER_API_KEY}`,"content-type":"application/json"},body:JSON.stringify({from:config.EMAIL_FROM,to:[row.recipient],subject:message.subject,html:message.html,text:message.text}),signal:AbortSignal.timeout(15_000)});
 if(!response.ok) throw new Error(`Email provider rejected request (${response.status})`);
}
async function processEmail():Promise<boolean>{
 const email=await claimEmail();
 if(!email) return false;
 try{
  await sendEmail(email);
  await inTransaction(async(client)=>{
   await client.query("SELECT set_config('app.worker','true',true)");
   await client.query("UPDATE email_outbox SET status='sent',sent_at=now(),last_error=NULL WHERE id=$1",[email.id]);
  });
 }catch(error){
  const message=error instanceof Error?error.message.slice(0,500):"Email delivery failed";
  await inTransaction(async(client)=>{
   await client.query("SELECT set_config('app.worker','true',true)");
   await client.query("UPDATE email_outbox SET status=CASE WHEN attempts>=8 THEN 'failed' ELSE 'queued' END,available_at=now()+($2::text||' seconds')::interval,last_error=$3 WHERE id=$1",[email.id,String(Math.min(3600,2**email.attempts*15)),message]);
  });
  console.error(JSON.stringify({level:"error",workerId,event:"email_delivery_failed",outboxId:email.id,attempts:email.attempts,error:message}));
 }
 return true;
}
async function processJob(job:Job){
 if(job.job_type==="system.health_check") return;
 if(job.job_type==="documents.scan") { await scanNextDocument(pool,config.CLAMAV_HOST,config.CLAMAV_PORT); return; }
 throw new Error(`No handler registered for job type: ${job.job_type}`);
}
async function runJob(){
 const job=await claimJob(); if(!job) return false;
 try{
  await processJob(job);
  await inTransaction(async(client)=>{
   await client.query("SELECT set_config('app.worker','true',true)");
   await client.query("UPDATE jobs SET status='completed',completed_at=now(),locked_at=NULL,locked_by=NULL,last_error=NULL WHERE id=$1 AND locked_by=$2",[job.id,workerId]);
  });
 }catch(error){
  const message=error instanceof Error?error.message.slice(0,1000):"Unknown worker error";
  await inTransaction(async(client)=>{
   await client.query("SELECT set_config('app.worker','true',true)");
   await client.query("UPDATE jobs SET status=$2,available_at=now()+($3::text||' seconds')::interval,locked_at=NULL,locked_by=NULL,last_error=$4 WHERE id=$1 AND locked_by=$5",[job.id,job.attempts>=8?"failed":"queued",String(Math.min(3600,2**(job.attempts+1)*5)),message,workerId]);
  });
  console.error(JSON.stringify({level:"error",workerId,event:"job_failed",jobId:job.id,error:message}));
 }
 return true;
}
let stopping=false;
process.on("SIGTERM",()=>{stopping=true;});
process.on("SIGINT",()=>{stopping=true;});
console.log(JSON.stringify({level:"info",workerId,event:"started",nodeEnv:config.NODE_ENV}));
while(!stopping){
 try{
  const emailWorked=await processEmail();
  const documentWorked=await scanNextDocument(pool,config.CLAMAV_HOST,config.CLAMAV_PORT);
  const jobWorked=await runJob();
  if(!emailWorked&&!documentWorked&&!jobWorked) await sleep(1500);
 }catch(error){
  console.error(JSON.stringify({level:"error",workerId,event:"poll_failed",error:error instanceof Error?error.message:"unknown"}));
  await sleep(3000);
 }
}
await pool.end();
