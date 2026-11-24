import { createCipheriv, createDecipheriv, createHash, createHmac, randomBytes, randomUUID, timingSafeEqual } from "node:crypto";
import type { PoolClient } from "pg";
import { config } from "./config.js";
import { inTransaction, pool } from "./db.js";
import { hashPassword, verifyPassword } from "./crypto.js";

const digest = (value: string) => createHash("sha256").update(value).digest("hex");
const key = () => createHash("sha256").update(config.MFA_ENCRYPTION_KEY).digest();
const token = () => randomBytes(32).toString("base64url");
const base32Alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

function base32Encode(bytes: Buffer): string {
  let bits = 0, value = 0, output = "";
  for (const byte of bytes) {
    value = (value << 8) | byte; bits += 8;
    while (bits >= 5) { output += base32Alphabet[(value >>> (bits - 5)) & 31]; bits -= 5; }
  }
  if (bits) output += base32Alphabet[(value << (5 - bits)) & 31];
  return output;
}
function base32Decode(value: string): Buffer {
  let bits = 0, buffer = 0;
  const output: number[] = [];
  for (const char of value.replace(/=+$/g, "").toUpperCase()) {
    const index = base32Alphabet.indexOf(char);
    if (index < 0) throw new Error("Invalid TOTP secret");
    buffer = (buffer << 5) | index; bits += 5;
    if (bits >= 8) { output.push((buffer >>> (bits - 8)) & 255); bits -= 8; }
  }
  return Buffer.from(output);
}
function encrypt(value: string): string {
  const iv = randomBytes(12);
  const cipher = createCipheriv("aes-256-gcm", key(), iv);
  const encrypted = Buffer.concat([cipher.update(value, "utf8"), cipher.final()]);
  return [iv, cipher.getAuthTag(), encrypted].map((part) => part.toString("base64url")).join(".");
}
function decrypt(value: string): string {
  const [ivText, tagText, encryptedText] = value.split(".");
  if (!ivText || !tagText || !encryptedText) throw new Error("Invalid encrypted MFA secret");
  const decipher = createDecipheriv("aes-256-gcm", key(), Buffer.from(ivText, "base64url"));
  decipher.setAuthTag(Buffer.from(tagText, "base64url"));
  return Buffer.concat([decipher.update(Buffer.from(encryptedText, "base64url")), decipher.final()]).toString("utf8");
}
function totp(secret: string, time = Date.now()): string {
  const counter = Math.floor(time / 30_000);
  const message = Buffer.alloc(8); message.writeBigUInt64BE(BigInt(counter));
  const hmac = createHmac("sha1", base32Decode(secret)).update(message).digest();
  const offset = hmac[hmac.length - 1] & 15;
  const binary = ((hmac[offset] & 127) << 24) | ((hmac[offset + 1] & 255) << 16) | ((hmac[offset + 2] & 255) << 8) | (hmac[offset + 3] & 255);
  return String(binary % 1_000_000).padStart(6, "0");
}
function matchingTotpCounter(secret:string,code:string):number|null {
  if(!/^\d{6}$/.test(code)) return null;
  const supplied=Buffer.from(code),current=Math.floor(Date.now()/30_000);
  for(const offset of [-1,0,1]) {
    const counter=current+offset;
    const expected=Buffer.from(totp(secret,counter*30_000));
    if(timingSafeEqual(supplied,expected)) return counter;
  }
  return null;
}
function verifyTotp(secret:string,code:string):boolean { return matchingTotpCounter(secret,code)!==null; }
async function queueEmail(client: PoolClient, userId: string, tenantId: string, recipient: string, template: string, payload: Record<string, unknown>) {
  await client.query(
    "INSERT INTO email_outbox(user_id,tenant_id,recipient,template_key,payload) VALUES($1,$2,$3,$4,$5::jsonb)",
    [userId, tenantId, recipient, template, JSON.stringify(payload)]
  );
}

export async function verifyEmailAddress(rawToken: string) {
  if (rawToken.length < 32 || rawToken.length > 128) throw Object.assign(new Error("Invalid verification link"), { statusCode: 400 });
  return inTransaction(async (client) => {
    const found = await client.query("SELECT token_id,user_id,tenant_id,email FROM public.arm_lookup_verification($1)", [digest(rawToken)]);
    const row = found.rows[0];
    if (!row) throw Object.assign(new Error("Verification link is invalid or expired"), { statusCode: 400 });
    await client.query("SELECT set_config('app.user_id',$1,true)", [row.user_id]);
    await client.query("SELECT set_config('app.tenant_id',$1,true)", [row.tenant_id]);
    await client.query("UPDATE email_verification_tokens SET consumed_at=now() WHERE id=$1 AND consumed_at IS NULL", [row.token_id]);
    await client.query("UPDATE users SET status='active',updated_at=now() WHERE id=$1 AND status='pending_verification'", [row.user_id]);
    return { verified: true };
  });
}

export async function requestPasswordRecovery(email: string) {
  const lookup = await pool.query("SELECT user_id,tenant_id,email FROM public.arm_lookup_account($1)", [email]);
  const row = lookup.rows[0];
  if (row) {
    const raw = token();
    await inTransaction(async (client) => {
      await client.query("SELECT set_config('app.user_id',$1,true)", [row.user_id]);
      await client.query("SELECT set_config('app.tenant_id',$1,true)", [row.tenant_id]);
      await client.query("UPDATE password_recovery_tokens SET consumed_at=now() WHERE user_id=$1 AND consumed_at IS NULL",[row.user_id]);
      await client.query("INSERT INTO password_recovery_tokens(user_id,token_hash,expires_at) VALUES($1,$2,now()+interval '30 minutes')", [row.user_id,digest(raw)]);
      await queueEmail(client,row.user_id,row.tenant_id,row.email,"password_recovery",{ resetUrl: `${config.PUBLIC_APP_URL}/reset-password?token=${raw}` });
    });
  }
  return { accepted: true };
}

export async function resetPassword(rawToken: string, password: string) {
  if (rawToken.length < 32 || rawToken.length > 128) throw Object.assign(new Error("Invalid recovery link"), { statusCode: 400 });
  const encoded = await hashPassword(password);
  await inTransaction(async (client) => {
    const found = await client.query("SELECT token_id,user_id FROM public.arm_lookup_recovery($1)", [digest(rawToken)]);
    const row = found.rows[0];
    if (!row) throw Object.assign(new Error("Recovery link is invalid or expired"), { statusCode: 400 });
    await client.query("SELECT set_config('app.user_id',$1,true)", [row.user_id]);
    await client.query("UPDATE password_recovery_tokens SET consumed_at=now() WHERE id=$1 AND consumed_at IS NULL", [row.token_id]);
    await client.query("UPDATE users SET password_hash=$1,updated_at=now() WHERE id=$2", [encoded,row.user_id]);
    await client.query("UPDATE auth_sessions SET revoked_at=now() WHERE user_id=$1 AND revoked_at IS NULL", [row.user_id]);
  });
  return { reset: true, sessionsRevoked: true };
}

export async function beginMfaEnrollment(userId:string,email:string,password:string) {
  const account=await inTransaction(async(client)=>{
    await client.query("SELECT set_config('app.user_id',$1,true)",[userId]);
    const found=await client.query("SELECT password_hash FROM users WHERE id=$1 AND status='active'",[userId]);
    const existing=await client.query("SELECT enabled_at FROM mfa_credentials WHERE user_id=$1",[userId]);
    if(existing.rows[0]?.enabled_at) throw Object.assign(new Error("MFA is already enabled. Verify with MFA before disabling it."),{statusCode:409});
    return found;
  });
  if(!account.rowCount || !(await verifyPassword(password,account.rows[0].password_hash))) throw Object.assign(new Error("Current password is incorrect"),{statusCode:401});
  const secret=base32Encode(randomBytes(20));
  await inTransaction(async(client)=>{
    await client.query("SELECT set_config('app.user_id',$1,true)",[userId]);
    await client.query("INSERT INTO mfa_credentials(user_id,secret_ciphertext,enabled_at,last_totp_counter) VALUES($1,$2,NULL,0) ON CONFLICT(user_id) DO UPDATE SET secret_ciphertext=EXCLUDED.secret_ciphertext,enabled_at=NULL,last_totp_counter=0,updated_at=now()",[userId,encrypt(secret)]);
    await client.query("DELETE FROM mfa_recovery_codes WHERE user_id=$1",[userId]);
  });
  const label=encodeURIComponent(`ARM:${email}`);
  return {secret,otpauthUri:`otpauth://totp/${label}?secret=${secret}&issuer=ARM&algorithm=SHA1&digits=6&period=30`};
}

export async function confirmMfaEnrollment(userId:string,code:string) {
  const recoveryCodes=await inTransaction(async(client)=>{
    await client.query("SELECT set_config('app.user_id',$1,true)",[userId]);
    const found=await client.query("SELECT secret_ciphertext FROM mfa_credentials WHERE user_id=$1 FOR UPDATE",[userId]);
    if(!found.rowCount) throw Object.assign(new Error("MFA enrollment has not started"),{statusCode:409});
    const secret=decrypt(found.rows[0].secret_ciphertext);
    const counter=matchingTotpCounter(secret,code);
    if(counter===null) throw Object.assign(new Error("Invalid authenticator code"),{statusCode:400});
    await client.query("UPDATE mfa_credentials SET enabled_at=now(),last_totp_counter=$2,updated_at=now() WHERE user_id=$1",[userId,counter]);
    await client.query("DELETE FROM mfa_recovery_codes WHERE user_id=$1",[userId]);
    const codes=Array.from({length:10},()=>randomBytes(8).toString("hex"));
    for(const recovery of codes) await client.query("INSERT INTO mfa_recovery_codes(user_id,code_hash) VALUES($1,$2)",[userId,digest(recovery)]);
    return codes;
  });
  return {enabled:true,recoveryCodes};
}

export async function disableMfa(userId:string,password:string,code:string) {
  const account=await inTransaction(async(client)=>{ await client.query("SELECT set_config('app.user_id',$1,true)",[userId]); return client.query("SELECT password_hash FROM users WHERE id=$1 AND status='active'",[userId]); });
  if(!account.rowCount || !(await verifyPassword(password,account.rows[0].password_hash))) throw Object.assign(new Error("Current password is incorrect"),{statusCode:401});
  await inTransaction(async(client)=>{
    await client.query("SELECT set_config('app.user_id',$1,true)",[userId]);
    const found=await client.query("SELECT secret_ciphertext,enabled_at,last_totp_counter FROM mfa_credentials WHERE user_id=$1 FOR UPDATE",[userId]);
    if(!found.rowCount || !found.rows[0].enabled_at) throw Object.assign(new Error("Authenticator code is invalid"),{statusCode:401});
    const counter=matchingTotpCounter(decrypt(found.rows[0].secret_ciphertext),code);
    if(counter===null) throw Object.assign(new Error("Authenticator code is invalid"),{statusCode:401});
    const advanced=await client.query("UPDATE mfa_credentials SET last_totp_counter=$2,updated_at=now() WHERE user_id=$1 AND last_totp_counter<$2",[userId,counter]);
    if(advanced.rowCount!==1) throw Object.assign(new Error("Authenticator code has already been used"),{statusCode:401});
    await client.query("DELETE FROM mfa_recovery_codes WHERE user_id=$1",[userId]);
    await client.query("DELETE FROM mfa_credentials WHERE user_id=$1",[userId]);
  });
  return {disabled:true};
}

export async function verifyMfaCode(userId:string,code:string):Promise<boolean> {
  return inTransaction(async(client)=>{
    await client.query("SELECT set_config('app.user_id',$1,true)",[userId]);
    const found=await client.query("SELECT secret_ciphertext,enabled_at,last_totp_counter FROM mfa_credentials WHERE user_id=$1 FOR UPDATE",[userId]);
    if(!found.rowCount || !found.rows[0].enabled_at) return false;
    const secret=decrypt(found.rows[0].secret_ciphertext);
    const counter=matchingTotpCounter(secret,code);
    if(counter!==null) {
      const advanced=await client.query("UPDATE mfa_credentials SET last_totp_counter=$2,updated_at=now() WHERE user_id=$1 AND last_totp_counter<$2",[userId,counter]);
      if(advanced.rowCount===1) return true;
    }
    const recovery=await client.query("SELECT id FROM mfa_recovery_codes WHERE user_id=$1 AND code_hash=$2 AND used_at IS NULL FOR UPDATE",[userId,digest(code)]);
    if(!recovery.rowCount) return false;
    await client.query("UPDATE mfa_recovery_codes SET used_at=now() WHERE id=$1",[recovery.rows[0].id]);
    return true;
  });
}

export function newVerificationToken() { return token(); }
export function hashOpaqueToken(value: string) { return digest(value); }
export function verificationExpiry() { return new Date(Date.now() + 24 * 60 * 60 * 1000); }
export async function enqueueVerificationEmail(client: PoolClient, userId: string, tenantId: string, email: string, raw: string) {
  await client.query("UPDATE email_verification_tokens SET consumed_at=now() WHERE user_id=$1 AND consumed_at IS NULL",[userId]);
  await client.query("INSERT INTO email_verification_tokens(user_id,token_hash,expires_at) VALUES($1,$2,now()+interval '24 hours')", [userId,digest(raw)]);
  await queueEmail(client,userId,tenantId,email,"email_verification",{ verificationUrl: `${config.PUBLIC_APP_URL}/verify-email?token=${raw}` });
}
