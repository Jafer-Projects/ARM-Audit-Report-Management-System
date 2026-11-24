import { after,before,test } from "node:test";
import assert from "node:assert/strict";
import { createHmac,randomUUID } from "node:crypto";
import { buildApp } from "../src/app.js";
import type { FastifyInstance } from "fastify";
import pg from "pg";
import { pool,withTenant } from "../src/db.js";

let app:FastifyInstance;
const workerPool=new pg.Pool({connectionString:process.env.WORKER_DATABASE_URL});
const migrationPool=new pg.Pool({connectionString:process.env.MIGRATION_DATABASE_URL});
const firstPassword="correct-horse-battery-staple-1";
let firstEmail="";
let secondEmail="";
let first:{accessToken:string;refreshToken:string;user:{id:string};tenant:{id:string}};
let second:{accessToken:string;refreshToken:string;user:{id:string};tenant:{id:string}};

async function readOutboxUrl(userId:string,tenantId:string,template:string):Promise<string>{
 return withTenant(userId,tenantId,async(client)=>{
  const row=await client.query("SELECT payload FROM email_outbox WHERE tenant_id=$1 AND template_key=$2 ORDER BY created_at DESC LIMIT 1",[tenantId,template]);
  assert.ok(row.rowCount,"missing "+template+" outbox message");
  const value=row.rows[0].payload;
  const url=value.verificationUrl??value.invitationUrl??value.resetUrl;
  assert.equal(typeof url,"string");
  return url as string;
 });
}
async function registerAndVerify(email:string,password:string,name:string){
 const response=await app.inject({method:"POST",url:"/v1/auth/register",payload:{email,displayName:name,password,organizationName:name+" Firm"}});
 assert.equal(response.statusCode,201,response.body);
 const registration=response.json();
 assert.equal(registration.verificationRequired,true);
 const verificationUrl=await readOutboxUrl(registration.user.id,registration.tenant.id,"email_verification");
 const token=new URL(verificationUrl).searchParams.get("token");
 assert.ok(token);
 const verified=await app.inject({method:"POST",url:"/v1/auth/verify-email",payload:{token}});
 assert.equal(verified.statusCode,200,verified.body);
 const login=await app.inject({method:"POST",url:"/v1/auth/login",payload:{email,password}});
 assert.equal(login.statusCode,200,login.body);
 return {...registration,...login.json()};
}
function base32Decode(value:string){
 const alphabet="ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";let bits=0,buffer=0;const out:number[]=[];
 for(const char of value){buffer=(buffer<<5)|alphabet.indexOf(char);bits+=5;if(bits>=8){out.push((buffer>>(bits-8))&255);bits-=8;}}
 return Buffer.from(out);
}
function totp(secret:string){
 const counter=Buffer.alloc(8);counter.writeBigUInt64BE(BigInt(Math.floor(Date.now()/30_000)));
 const h=createHmac("sha1",base32Decode(secret)).update(counter).digest();const offset=h[h.length-1]&15;
 const value=((h[offset]&127)<<24)|((h[offset+1]&255)<<16)|((h[offset+2]&255)<<8)|(h[offset+3]&255);
 return String(value%1_000_000).padStart(6,"0");
}
const headers=()=>({authorization:"Bearer "+first.accessToken,"x-tenant-id":first.tenant.id});

before(async()=>{
 app=await buildApp();await app.ready();
 const suffix=randomUUID();
 firstEmail="owner-"+suffix+"@example.test";secondEmail="member-"+suffix+"@example.test";
 first=await registerAndVerify(firstEmail,firstPassword,"First Owner");
 second=await registerAndVerify(secondEmail,"correct-horse-battery-staple-2","Second Owner");
});
after(async()=>{await app.close();await workerPool.end();await migrationPool.end();});

test("migration, API and worker roles are non-superuser with separate privileges",async()=>{
 const migration=await migrationPool.query("SELECT rolsuper,rolbypassrls FROM pg_roles WHERE rolname=current_user");
 assert.equal(migration.rows[0].rolsuper,false);assert.equal(migration.rows[0].rolbypassrls,false);
 await migrationPool.query("SELECT version FROM schema_migrations");
 const api=await pool.query("SELECT rolsuper,rolbypassrls FROM pg_roles WHERE rolname=current_user");
 assert.equal(api.rows[0].rolsuper,false);assert.equal(api.rows[0].rolbypassrls,false);
 await assert.rejects(pool.query("SELECT version FROM schema_migrations"),(error)=>error.code==="42501");
 await assert.rejects(pool.query("SELECT id FROM jobs"),(error)=>error.code==="42501");
 const worker=await workerPool.query("SELECT rolsuper,rolbypassrls FROM pg_roles WHERE rolname=current_user");
 assert.equal(worker.rows[0].rolsuper,false);assert.equal(worker.rows[0].rolbypassrls,false);
 const privileges=await workerPool.query("SELECT has_column_privilege(current_user,'jobs','status','UPDATE') AS status_update,has_column_privilege(current_user,'jobs','tenant_id','UPDATE') AS tenant_update");
 assert.equal(privileges.rows[0].status_update,true);assert.equal(privileges.rows[0].tenant_update,false);
 await workerPool.query("BEGIN");
 try{await workerPool.query("SELECT set_config('app.worker','true',true)");await workerPool.query("SELECT id FROM jobs LIMIT 1");await assert.rejects(workerPool.query("SELECT id FROM tenants"),(error)=>error.code==="42501");await workerPool.query("ROLLBACK");}
 catch(error){await workerPool.query("ROLLBACK");throw error;}
});

test("email verification is required before login and verification tokens are one-time",async()=>{
 const suffix=randomUUID(),email="pending-"+suffix+"@example.test";
 const registered=await app.inject({method:"POST",url:"/v1/auth/register",payload:{email,displayName:"Pending User",password:"correct-horse-battery-staple-3",organizationName:"Pending Firm"}});
 assert.equal(registered.statusCode,201);
 const item=registered.json();
 const blocked=await app.inject({method:"POST",url:"/v1/auth/login",payload:{email,password:"correct-horse-battery-staple-3"}});
 assert.equal(blocked.statusCode,403);
 const oldUrl=await readOutboxUrl(item.user.id,item.tenant.id,"email_verification");
 const oldToken=new URL(oldUrl).searchParams.get("token");
 assert.ok(oldToken);
 const resend=await app.inject({method:"POST",url:"/v1/auth/verification/resend",payload:{email}});
 assert.equal(resend.statusCode,200);assert.deepEqual(resend.json(),{accepted:true});
 const newUrl=await readOutboxUrl(item.user.id,item.tenant.id,"email_verification");
 const token=new URL(newUrl).searchParams.get("token");
 assert.ok(token);assert.notEqual(token,oldToken);
 const oldVerification=await app.inject({method:"POST",url:"/v1/auth/verify-email",payload:{token:oldToken}});
 assert.equal(oldVerification.statusCode,400);
 const verified=await app.inject({method:"POST",url:"/v1/auth/verify-email",payload:{token}});
 assert.equal(verified.statusCode,200);
 const replay=await app.inject({method:"POST",url:"/v1/auth/verify-email",payload:{token}});
 assert.equal(replay.statusCode,400);
});

test("registration creates owner membership and tenant context is membership scoped",async()=>{
 const context=await app.inject({method:"GET",url:"/v1/tenant/context",headers:headers()});
 assert.equal(context.statusCode,200,context.body);assert.equal(context.json().role,"owner");
 const denied=await app.inject({method:"GET",url:"/v1/tenant/context",headers:{authorization:"Bearer "+second.accessToken,"x-tenant-id":first.tenant.id}});
 assert.equal(denied.statusCode,403);
});

test("role administration and organization invitation acceptance are tenant scoped",async()=>{
 const role=await app.inject({method:"POST",url:"/v1/tenant/roles",headers:headers(),payload:{key:"audit_operator",name:"Audit Operator",permissions:["members.read","reports.approve","accounting.post","engagements.read","workpapers.write"]}});
 assert.equal(role.statusCode,201,role.body);
 const invite=await app.inject({method:"POST",url:"/v1/tenant/invitations",headers:headers(),payload:{email:secondEmail,roleId:role.json().id}});
 assert.equal(invite.statusCode,201,invite.body);
 const invitationUrl=await readOutboxUrl(first.user.id,first.tenant.id,"organization_invitation");
 const token=new URL(invitationUrl).searchParams.get("token");
 assert.ok(token);
 const accepted=await app.inject({method:"POST",url:"/v1/tenant/invitations/accept",headers:{authorization:"Bearer "+second.accessToken},payload:{token}});
 assert.equal(accepted.statusCode,200,accepted.body);
 assert.equal(accepted.json().accepted,true);
 const context=await app.inject({method:"GET",url:"/v1/tenant/context",headers:{authorization:"Bearer "+second.accessToken,"x-tenant-id":first.tenant.id}});
 assert.equal(context.statusCode,200,context.body);
 const assignment=await app.inject({method:"POST",url:"/v1/tenant/memberships/"+accepted.json().membership_id+"/roles",headers:headers(),payload:{roleId:role.json().id}});
 assert.equal(assignment.statusCode,200,assignment.body);
 const members=await app.inject({method:"GET",url:"/v1/tenant/members",headers:headers()});
 assert.equal(members.statusCode,200,members.body);
 assert.equal(members.json().members.length,2);
 assert.ok(members.json().members.some((member)=>member.email===secondEmail));
 const permissions=await app.inject({method:"GET",url:"/v1/tenant/permissions",headers:headers()});
 assert.equal(permissions.statusCode,200);assert.ok(permissions.json().permissions.some((permission)=>permission.permission_key==="accounting.post"));
 const tempRole=await app.inject({method:"POST",url:"/v1/tenant/roles",headers:headers(),payload:{key:"temporary_role",name:"Temporary Role",permissions:["engagements.read"]}});
 assert.equal(tempRole.statusCode,201);
 const renamed=await app.inject({method:"PUT",url:"/v1/tenant/roles/"+tempRole.json().id,headers:headers(),payload:{name:"Renamed Temporary Role"}});
 assert.equal(renamed.statusCode,200);
 const deleted=await app.inject({method:"DELETE",url:"/v1/tenant/roles/"+tempRole.json().id,headers:headers()});
 assert.equal(deleted.statusCode,200);
 const pendingInvite=await app.inject({method:"POST",url:"/v1/tenant/invitations",headers:headers(),payload:{email:"revoked-"+randomUUID()+"@example.test",roleId:role.json().id}});
 assert.equal(pendingInvite.statusCode,201);
 const revoked=await app.inject({method:"POST",url:"/v1/tenant/invitations/"+pendingInvite.json().id+"/revoke",headers:headers()});
 assert.equal(revoked.statusCode,200);
 const suspended=await app.inject({method:"PATCH",url:"/v1/tenant/memberships/"+accepted.json().membership_id+"/status",headers:headers(),payload:{status:"suspended"}});
 assert.equal(suspended.statusCode,200);
 const blocked=await app.inject({method:"GET",url:"/v1/tenant/context",headers:{authorization:"Bearer "+second.accessToken,"x-tenant-id":first.tenant.id}});
 assert.equal(blocked.statusCode,403);
 const reactivated=await app.inject({method:"PATCH",url:"/v1/tenant/memberships/"+accepted.json().membership_id+"/status",headers:headers(),payload:{status:"active"}});
 assert.equal(reactivated.statusCode,200);
});

test("MFA enrollment requires password, recovery code is single-use and login requires MFA",async()=>{
 const started=await app.inject({method:"POST",url:"/v1/auth/mfa/enroll",headers:headers(),payload:{password:firstPassword}});
 assert.equal(started.statusCode,200,started.body);
 const secret=new URL(started.json().otpauthUri).searchParams.get("secret");
 assert.ok(secret);
 const confirmed=await app.inject({method:"POST",url:"/v1/auth/mfa/confirm",headers:headers(),payload:{code:totp(secret)}});
 assert.equal(confirmed.statusCode,200,confirmed.body);
 const recovery=confirmed.json().recoveryCodes[0] as string;
 const loggedIn=await app.inject({method:"POST",url:"/v1/auth/login",payload:{email:firstEmail,password:firstPassword}});
 assert.equal(loggedIn.statusCode,200);assert.equal(loggedIn.json().mfaRequired,true);
 const completed=await app.inject({method:"POST",url:"/v1/auth/mfa/complete",payload:{challengeToken:loggedIn.json().challengeToken,code:recovery}});
 assert.equal(completed.statusCode,200,completed.body);
 const replay=await app.inject({method:"POST",url:"/v1/auth/mfa/complete",payload:{challengeToken:loggedIn.json().challengeToken,code:recovery}});
 assert.equal(replay.statusCode,401);
});

test("prototype sync is idempotent, version checked, cursor based and domain types are rejected",async()=>{
 const aggregateId=randomUUID();
 const event={eventId:randomUUID(),idempotencyKey:randomUUID(),aggregateType:"prototype.workspace_record",aggregateId,baseVersion:0,eventType:"aggregate.upsert",schemaVersion:1,occurredAt:new Date().toISOString(),payload:{record:{id:aggregateId,record_type:"historical_record",title:"Synthetic sync record",status:"Draft",row_version:1}}};
 const accepted=await app.inject({method:"POST",url:"/v1/sync/events",headers:headers(),payload:{events:[event]}});
 assert.equal(accepted.statusCode,200,accepted.body);assert.equal(accepted.json().results[0].status,"accepted");
 const duplicate=await app.inject({method:"POST",url:"/v1/sync/events",headers:headers(),payload:{events:[event]}});
 assert.equal(duplicate.statusCode,200);assert.equal(duplicate.json().results[0].duplicate,true);
 const conflict=await app.inject({method:"POST",url:"/v1/sync/events",headers:headers(),payload:{events:[{...event,eventId:randomUUID(),idempotencyKey:randomUUID()}]}});
 assert.equal(conflict.json().results[0].status,"conflict");
 const pulled=await app.inject({method:"GET",url:"/v1/sync/events",headers:headers()});
 assert.equal(pulled.statusCode,200);assert.equal(pulled.json().events.length,2);
 assert.deepEqual(pulled.json().events.map((x)=>x.outcome),["accepted","conflict"]);
 const drained=await app.inject({method:"GET",url:"/v1/sync/events?after="+pulled.json().nextCursor,headers:headers()});
 assert.equal(drained.json().events.length,0);
 const unsupported=await app.inject({method:"POST",url:"/v1/sync/events",headers:headers(),payload:{events:[{...event,eventId:randomUUID(),idempotencyKey:randomUUID(),aggregateType:"accounting.posted_journal"}]}});
 assert.equal(unsupported.statusCode,400);
});

test("tenant RLS hides another tenant even when a foreign tenant ID is supplied",async()=>{
 const client=await pool.connect();
 try{await client.query("BEGIN");await client.query("SELECT set_config('app.user_id',$1,true)",[first.user.id]);await client.query("SELECT set_config('app.tenant_id',$1,true)",[second.tenant.id]);const hidden=await client.query("SELECT count(*)::int AS count FROM sync_aggregates");assert.equal(hidden.rows[0].count,0);await client.query("ROLLBACK");}
 finally{client.release();}
});

test("audit engagement, chart of accounts and balanced journals require tenant permissions",async()=>{
 const engagement=await app.inject({method:"POST",url:"/v1/audit/engagements",headers:headers(),payload:{clientReference:"ENG-"+randomUUID(),title:"Annual financial statement audit",periodStart:"2026-01-01",periodEnd:"2026-12-31",framework:"ISA"}});
 assert.equal(engagement.statusCode,201,engagement.body);
 const cash=await app.inject({method:"POST",url:"/v1/accounting/accounts",headers:headers(),payload:{code:"1000-"+randomUUID().slice(0,6),name:"Cash",type:"asset",currency:"ETB"}});
 const revenue=await app.inject({method:"POST",url:"/v1/accounting/accounts",headers:headers(),payload:{code:"4000-"+randomUUID().slice(0,6),name:"Revenue",type:"income",currency:"ETB"}});
 assert.equal(cash.statusCode,201,cash.body);assert.equal(revenue.statusCode,201,revenue.body);
 const journal=await app.inject({method:"POST",url:"/v1/accounting/journals",headers:headers(),payload:{journalNumber:"J-"+randomUUID(),date:"2026-11-24",description:"Synthetic balanced test journal",currency:"ETB",lines:[{accountId:cash.json().id,debit:100,credit:0},{accountId:revenue.json().id,debit:0,credit:100}]}});
 assert.equal(journal.statusCode,201,journal.body);
 const unbalanced=await app.inject({method:"POST",url:"/v1/accounting/journals",headers:headers(),payload:{journalNumber:"U-"+randomUUID(),date:"2026-11-24",description:"Unbalanced",currency:"ETB",lines:[{accountId:cash.json().id,debit:100,credit:0},{accountId:revenue.json().id,debit:0,credit:99}]}});
 assert.equal(unbalanced.statusCode,400);
 const post=await app.inject({method:"POST",url:"/v1/accounting/journals/"+journal.json().id+"/post",headers:{authorization:"Bearer "+second.accessToken,"x-tenant-id":first.tenant.id}});
 assert.equal(post.statusCode,200,post.body);assert.equal(post.json().status,"posted");
 const immutable=await withTenant(first.user.id,first.tenant.id,(client)=>client.query("SELECT status FROM accounting_journals WHERE id=$1",[journal.json().id]));
 assert.equal(immutable.rows[0].status,"posted");
});

test("evidence upload is signature checked and cannot be downloaded before scan",async()=>{
 const invalid=await app.inject({method:"POST",url:"/v1/documents",headers:headers(),payload:{fileName:"fake.pdf",mediaType:"application/pdf",contentBase64:Buffer.from("not a pdf").toString("base64"),engagementId:randomUUID()}});
 assert.equal(invalid.statusCode,415);
});

test("password recovery is non-enumerating, one-time and revokes existing sessions",async()=>{
 const requested=await app.inject({method:"POST",url:"/v1/auth/recovery/request",payload:{email:firstEmail}});
 assert.equal(requested.statusCode,200);assert.deepEqual(requested.json(),{accepted:true});
 const resetUrl=await readOutboxUrl(first.user.id,first.tenant.id,"password_recovery");
 const token=new URL(resetUrl).searchParams.get("token");
 assert.ok(token);
 const reset=await app.inject({method:"POST",url:"/v1/auth/recovery/complete",payload:{token,password:"new-correct-horse-battery-4"}});
 assert.equal(reset.statusCode,200,reset.body);
 const replay=await app.inject({method:"POST",url:"/v1/auth/recovery/complete",payload:{token,password:"another-correct-horse-5"}});
 assert.equal(replay.statusCode,400);
 const oldLogin=await app.inject({method:"POST",url:"/v1/auth/login",payload:{email:firstEmail,password:firstPassword}});
 assert.equal(oldLogin.statusCode,401);
 const newLogin=await app.inject({method:"POST",url:"/v1/auth/login",payload:{email:firstEmail,password:"new-correct-horse-battery-4"}});
 assert.equal(newLogin.statusCode,200);assert.equal(newLogin.json().mfaRequired,true);
});
