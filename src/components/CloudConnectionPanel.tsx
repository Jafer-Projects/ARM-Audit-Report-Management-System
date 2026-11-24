import { useState,type FormEvent } from "react";
import { cloudAuth,type CloudSession } from "../cloudAuth";
import { getLocalSyncStatus,synchronizeDesktop } from "../cloudSync";

type Props={onClose:()=>void};
type LocalStatus={configured:boolean;cursor:string;pending:number;conflicts:number};
export function CloudConnectionPanel({onClose}:Props){
 const [api,setApi]=useState("https://api.example.com");
 const [email,setEmail]=useState("");
 const [password,setPassword]=useState("");
 const [displayName,setDisplayName]=useState("");
 const [organizationName,setOrganizationName]=useState("");
 const [verificationToken,setVerificationToken]=useState("");
 const [recoveryToken,setRecoveryToken]=useState("");
 const [mfaCode,setMfaCode]=useState("");
 const [recoveryPassword,setRecoveryPassword]=useState("");
 const [challenge,setChallenge]=useState("");
 const [session,setSession]=useState<CloudSession|null>(null);
 const [tenantId,setTenantId]=useState("");
 const [status,setStatus]=useState<LocalStatus|null>(null);
 const [mfaEnabled,setMfaEnabled]=useState(false);
 const [mfaEnrollment,setMfaEnrollment]=useState<{secret:string;otpauthUri:string}|null>(null);
 const [mfaPassword,setMfaPassword]=useState("");
 const [mfaManagementCode,setMfaManagementCode]=useState("");
 const [recoveryCodes,setRecoveryCodes]=useState<string[]>([]);
 const [message,setMessage]=useState("");
 const [error,setError]=useState("");
 const [busy,setBusy]=useState(false);
 const [showRegister,setShowRegister]=useState(false);
 const [showRecovery,setShowRecovery]=useState(false);

 async function run(action:()=>Promise<void>){
  setBusy(true);setError("");setMessage("");
  try{await action();}catch(value){setError(value instanceof Error?value.message:String(value));}
  finally{setBusy(false);}
 }
 async function login(event:FormEvent){
  event.preventDefault();
  await run(async()=>{
   const result=await cloudAuth.login(api,email,password);
   if("mfaRequired" in result){setChallenge(result.challengeToken);setMessage("Enter the authenticator code or a single-use recovery code.");return;}
   setSession(result);setTenantId(result.memberships?.[0]?.tenant_id??"");
   setMfaEnabled((await cloudAuth.mfaStatus(api,result.accessToken)).enabled);
   setStatus(await getLocalSyncStatus());setMessage("Cloud session established in memory for this application session.");
  });
 }
 async function completeMfa(event:FormEvent){
  event.preventDefault();
  await run(async()=>{
   const result=await cloudAuth.completeMfa(api,challenge,mfaCode);
   setSession(result);setTenantId(result.memberships?.[0]?.tenant_id??"");
   setMfaEnabled((await cloudAuth.mfaStatus(api,result.accessToken)).enabled);
   setChallenge("");setMfaCode("");setStatus(await getLocalSyncStatus());setMessage("Multi-factor authentication completed.");
  });
 }
 async function register(event:FormEvent){
  event.preventDefault();
  await run(async()=>{
   await cloudAuth.register(api,{email,displayName,password,organizationName});
   setShowRegister(false);setMessage("Registration created. Verify the address using the link sent to the supplied email before signing in.");
  });
 }
 async function verify(event:FormEvent){
  event.preventDefault();
  await run(async()=>{await cloudAuth.verifyEmail(api,verificationToken);setVerificationToken("");setMessage("Email verified. You can now sign in.");});
 }
 async function recover(event:FormEvent){
  event.preventDefault();
  await run(async()=>{await cloudAuth.requestRecovery(api,email);setMessage("If the account is eligible, a recovery link has been sent.");});
 }
 async function completeRecovery(event:FormEvent){
  event.preventDefault();
  await run(async()=>{await cloudAuth.completeRecovery(api,recoveryToken,recoveryPassword);setRecoveryToken("");setRecoveryPassword("");setShowRecovery(false);setMessage("Password reset. Existing sessions have been revoked; sign in again.");});
 }
 async function refreshSession(){
  if(!session)return;
  await run(async()=>{
   const next=await cloudAuth.refresh(api,session.refreshToken);
   setSession({...session,...next});
   setMessage("Cloud session refreshed. The refresh token was rotated.");
  });
 }
 async function syncNow(){
  if(!session||!tenantId)throw new Error("Sign in and select an organization first.");
  await run(async()=>{
   const result=await synchronizeDesktop({apiBaseUrl:api,tenantId,accessToken:session.accessToken});
   setStatus(result.status);
   setMessage("Sync cycle completed: "+result.pushed+" event(s) submitted and "+result.pulled+" event(s) received.");
  });
 }
 async function closePanel(){
  if(session){try{await cloudAuth.logout(api,session.accessToken);}catch(value){setError(value instanceof Error?value.message:String(value));return;}}
  onClose();
 }

 async function beginMfa(){
  if(!session||!tenantId) return;
  await run(async()=>{setMfaEnrollment(await cloudAuth.beginMfa(api,session.accessToken,tenantId,mfaPassword));setMfaPassword("");setMessage("Scan the otpauth URI in your authenticator app, then enter the current six-digit code.");});
 }
 async function confirmMfa(){
  if(!session||!tenantId) return;
  await run(async()=>{
   const result=await cloudAuth.confirmMfa(api,session.accessToken,tenantId,mfaManagementCode);
   setMfaEnabled(true);setMfaEnrollment(null);setMfaManagementCode("");setRecoveryCodes(result.recoveryCodes);
   setMessage("MFA enabled. Copy and store these recovery codes securely; they are shown only once.");
  });
 }
 async function disableMfa(){
  if(!session||!tenantId) return;
  await run(async()=>{
   await cloudAuth.disableMfa(api,session.accessToken,tenantId,mfaPassword,mfaManagementCode);
   setMfaEnabled(false);setMfaManagementCode("");setMfaPassword("");setRecoveryCodes([]);
   setMessage("MFA disabled after password and fresh authenticator verification.");
  });
 }
 async function logout(){
  if(!session)return;
  await run(async()=>{await cloudAuth.logout(api,session.accessToken);setSession(null);setTenantId("");setStatus(null);setMessage("Cloud session revoked.");});
 }
 return <div className="cloud-panel-backdrop" role="presentation">
  <section className="cloud-panel" role="dialog" aria-modal="true" aria-labelledby="cloud-panel-title">
   <header className="cloud-panel-header"><div><span className="overline">SECURE CLOUD CONNECTION</span><h2 id="cloud-panel-title">Cloud workspace</h2></div><button type="button" className="text-button" disabled={busy} onClick={()=>void closePanel()}>Close</button></header>
   <p className="cloud-panel-note">Cloud access is separate from the local demo persona. Tokens stay in memory only and are cleared when this panel is closed or the app restarts.</p>
   <label>Cloud API HTTPS address<input value={api} onChange={event=>setApi(event.target.value)} placeholder="https://api.your-domain.example" disabled={busy||!!session||!!challenge}/></label>
   {!session&&!challenge&&<form onSubmit={login} className="cloud-panel-form">
    <h3>Sign in</h3>
    <label>Email<input type="email" autoComplete="username" value={email} onChange={event=>setEmail(event.target.value)} required/></label>
    <label>Password<input type="password" autoComplete="current-password" value={password} onChange={event=>setPassword(event.target.value)} required/></label>
    <button type="submit" disabled={busy}>Sign in to cloud</button>
    <button type="button" className="text-button" onClick={()=>setShowRegister(value=>!value)}>Create an account</button>
    <button type="button" className="text-button" onClick={()=>setShowRecovery(value=>!value)}>Forgot password?</button>
   </form>}
   {showRegister&&!session&&<form onSubmit={register} className="cloud-panel-form">
    <h3>Register a cloud account</h3>
    <label>Full display name<input value={displayName} onChange={event=>setDisplayName(event.target.value)} required/></label>
    <label>Organization name<input value={organizationName} onChange={event=>setOrganizationName(event.target.value)} required/></label>
    <button type="submit" disabled={busy}>Register and send verification</button>
   </form>}
   {!session&&!challenge&&<form onSubmit={verify} className="cloud-panel-form">
    <h3>Verify email</h3><label>Verification token<input value={verificationToken} onChange={event=>setVerificationToken(event.target.value)} required/></label>
    <button type="submit" disabled={busy}>Verify email</button>
    <button type="button" className="text-button" disabled={busy||!email} onClick={()=>void run(async()=>{await cloudAuth.resendVerification(api,email);setMessage("If the account needs verification, a new link has been sent.");})}>Resend verification link</button>
   </form>}
   {showRecovery&&!session&&<form onSubmit={recover} className="cloud-panel-form">
    <h3>Account recovery</h3><label>Email<input type="email" value={email} onChange={event=>setEmail(event.target.value)} required/></label>
    <button type="submit" disabled={busy}>Request recovery link</button>
    <label>Recovery token<input value={recoveryToken} onChange={event=>setRecoveryToken(event.target.value)}/></label>
    <label>New password<input type="password" minLength={12} value={recoveryPassword} onChange={event=>setRecoveryPassword(event.target.value)}/></label>
    <button type="button" disabled={busy||!recoveryToken||!recoveryPassword} onClick={()=>void run(async()=>{await cloudAuth.completeRecovery(api,recoveryToken,recoveryPassword);setShowRecovery(false);setMessage("Password reset; sign in with the new password.");})}>Complete recovery</button>
   </form>}
   {challenge&&!session&&<form onSubmit={completeMfa} className="cloud-panel-form">
    <h3>Multi-factor authentication</h3><label>Authenticator or recovery code<input value={mfaCode} onChange={event=>setMfaCode(event.target.value)} autoComplete="one-time-code" required/></label>
    <button type="submit" disabled={busy}>Verify and continue</button>
   </form>}
   {session&&<div className="cloud-panel-form">
    <h3>Connected account</h3><p>{session.user?.email??email}</p>
    <label>Organization<select value={tenantId} onChange={event=>setTenantId(event.target.value)}>{(session.memberships??[]).map(item=><option key={item.tenant_id} value={item.tenant_id}>{item.tenant_id} — {item.role}</option>)}</select></label>
    <div className="cloud-sync-status">
     <span>Pending local events: <strong>{status?.pending??"—"}</strong></span>
     <span>Conflicts: <strong>{status?.conflicts??"—"}</strong></span>
     <span>Pull cursor: <strong>{status?.cursor??"—"}</strong></span>
    </div>

    <div className="cloud-mfa-section">
     <h3>Multi-factor authentication</h3>
     <p>Status: <strong>{mfaEnabled?"Enabled":"Not enabled"}</strong></p>
     {!mfaEnabled&&!mfaEnrollment&&<><label>Confirm current password<input type="password" value={mfaPassword} onChange={event=>setMfaPassword(event.target.value)}/></label><button type="button" disabled={busy||!mfaPassword||!tenantId} onClick={()=>void beginMfa()}>Set up authenticator MFA</button></>}
     {mfaEnrollment&&<><p>Secret: <code>{mfaEnrollment.secret}</code></p><p className="cloud-panel-note">Authenticator URI: <code>{mfaEnrollment.otpauthUri}</code></p><label>Six-digit authenticator code<input inputMode="numeric" autoComplete="one-time-code" value={mfaManagementCode} onChange={event=>setMfaManagementCode(event.target.value)}/></label><button type="button" disabled={busy||!/^\d{6}$/.test(mfaManagementCode)} onClick={()=>void confirmMfa()}>Confirm and enable MFA</button></>}
     {mfaEnabled&&<><label>Current password<input type="password" value={mfaPassword} onChange={event=>setMfaPassword(event.target.value)}/></label><label>Fresh authenticator code<input inputMode="numeric" autoComplete="one-time-code" value={mfaManagementCode} onChange={event=>setMfaManagementCode(event.target.value)}/></label><button type="button" disabled={busy||!mfaPassword||!/^\d{6}$/.test(mfaManagementCode)} onClick={()=>void disableMfa()}>Disable MFA</button></>}
     {recoveryCodes.length>0&&<div className="cloud-recovery-codes"><strong>One-time recovery codes</strong><ul>{recoveryCodes.map(code=><li key={code}><code>{code}</code></li>)}</ul><button type="button" className="text-button" onClick={()=>setRecoveryCodes([])}>I stored these codes</button></div>}
    </div>
    <button type="button" disabled={busy||!tenantId} onClick={()=>void syncNow()}>Synchronize now</button>
    <button type="button" className="text-button" disabled={busy} onClick={()=>void refreshSession()}>Refresh session</button>
    <button type="button" className="text-button" disabled={busy} onClick={()=>void logout()}>Sign out</button>
   </div>}
   {message&&<p className="cloud-panel-message" role="status">{message}</p>}
   {error&&<p className="cloud-panel-error" role="alert">{error}</p>}
   <footer>Cloud operations require an independently validated deployment. Local demo data is not automatically uploaded or treated as production evidence.</footer>
  </section>
 </div>;
}
