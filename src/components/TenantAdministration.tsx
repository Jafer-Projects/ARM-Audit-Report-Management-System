import { useEffect,useState } from "react";
import { cloudAdmin,type Permission,type TenantInvitation,type TenantMember,type TenantRole } from "../services/cloudAdmin";

export function TenantAdministration(){
 const [permissions,setPermissions]=useState<string[]>([]);
 const [members,setMembers]=useState<TenantMember[]>([]);
 const [roles,setRoles]=useState<TenantRole[]>([]);
 const [catalogue,setCatalogue]=useState<Permission[]>([]);
 const [invitations,setInvitations]=useState<TenantInvitation[]>([]);
 const [selectedRoleId,setSelectedRoleId]=useState("");
 const [selectedRolePermissions,setSelectedRolePermissions]=useState<string[]>([]);
 const [newRolePermissions,setNewRolePermissions]=useState<string[]>([]);
 const [roleKey,setRoleKey]=useState("");const [roleName,setRoleName]=useState("");
 const [inviteEmail,setInviteEmail]=useState("");const [inviteRoleId,setInviteRoleId]=useState("");
 const [memberId,setMemberId]=useState("");const [memberRoleId,setMemberRoleId]=useState("");
 const [busy,setBusy]=useState(false);const [error,setError]=useState("");const [message,setMessage]=useState("");
 const can=(key:string)=>permissions.includes(key);
 async function load(){
  setBusy(true);setError("");
  try{
   const context=await cloudAdmin.context();const effective=context.permissions??[];setPermissions(effective);
   const [m,r,i,c]=await Promise.all([
    effective.includes("members.read")?cloudAdmin.members():Promise.resolve([]),
    effective.includes("roles.read")?cloudAdmin.roles():Promise.resolve([]),
    effective.includes("members.read")?cloudAdmin.invitations():Promise.resolve([]),
    effective.includes("roles.manage")?cloudAdmin.permissions():Promise.resolve([])
   ]);
   setMembers(m);setRoles(r);setInvitations(i);setCatalogue(c);
   setSelectedRoleId(current=>r.some(role=>role.id===current)?current:r.find(role=>!role.is_system)?.id??r[0]?.id??"");
   setInviteRoleId(current=>r.some(role=>role.id===current&&!role.is_system)?current:r.find(role=>!role.is_system)?.id??"");
   setMemberId(current=>m.some(member=>member.membership_id===current)?current:m[0]?.membership_id??"");
   setMemberRoleId(current=>r.some(role=>role.id===current&&!role.is_system)?current:r.find(role=>!role.is_system)?.id??"");
  }catch(value){setError(value instanceof Error?value.message:String(value));}
  finally{setBusy(false);}
 }
 useEffect(()=>{void load();},[]);
 useEffect(()=>{const role=roles.find(item=>item.id===selectedRoleId);setSelectedRolePermissions(role?.permissions??[]);},[roles,selectedRoleId]);
 async function run(action:()=>Promise<unknown>,success:string){setBusy(true);setError("");setMessage("");try{await action();setMessage(success);await load();}catch(value){setError(value instanceof Error?value.message:String(value));}finally{setBusy(false);}}
 return <section className="cloud-mfa-section tenant-admin-panel">
  <div className="section-head"><div><span className="overline">TENANT ADMINISTRATION</span><h3>Members, roles and permissions</h3><p>Every operation is re-authorized by the server and written to tenant audit history.</p></div><button type="button" className="text-button" disabled={busy} onClick={()=>void load()}>Refresh</button></div>
  {error&&<p className="error-banner" role="alert">{error}</p>}{message&&<p className="success-message" role="status">{message}</p>}
  {can("members.read")&&<div className="admin-subsection"><h4>Organization members</h4><div className="table-wrap"><table><thead><tr><th>Member</th><th>Status</th><th>Assigned roles</th><th>Actions</th></tr></thead><tbody>{members.map(member=><tr key={member.membership_id}><td><strong>{member.display_name}</strong><small>{member.email}</small></td><td>{member.status}</td><td>{member.assigned_roles.join(", ")||member.legacy_role}{member.assigned_roles.map(name=>{const role=roles.find(item=>item.display_name===name);return role&&can("roles.manage")?<button type="button" key={role.id} disabled={busy} onClick={()=>void run(()=>cloudAdmin.removeRole(member.membership_id,role.id),"Role removed from member.")}>Remove {name}</button>:null;})}</td><td>{can("roles.manage")&&<button type="button" disabled={busy} onClick={()=>{setMemberId(member.membership_id);}}>Select</button>}{can("tenant.manage")&&<select aria-label={"Status for "+member.email} value={member.status} disabled={busy||member.legacy_role==="owner"} onChange={event=>void run(()=>cloudAdmin.changeMemberStatus(member.membership_id,event.target.value as "active"|"suspended"|"revoked"),"Member status updated.")}><option value="active">Active</option><option value="suspended">Suspended</option><option value="revoked">Revoked</option></select>}</td></tr>)}</tbody></table></div>
   {can("roles.manage")&&<div className="form-grid"><label>Selected member<select value={memberId} onChange={event=>setMemberId(event.target.value)}>{members.map(member=><option key={member.membership_id} value={member.membership_id}>{member.display_name} — {member.email}</option>)}</select></label><label>Assign role<select value={memberRoleId} onChange={event=>setMemberRoleId(event.target.value)}>{roles.filter(role=>!role.is_system&&role.role_key!=="owner").map(role=><option key={role.id} value={role.id}>{role.display_name}</option>)}</select></label><button type="button" disabled={busy||!memberId||!memberRoleId} onClick={()=>void run(()=>cloudAdmin.assignRole(memberId,memberRoleId),"Role assigned to member.")}>Assign role</button></div>}</div>}
  {can("members.invite")&&<div className="admin-subsection"><h4>Invite a member</h4><div className="form-grid"><label>Email<input type="email" value={inviteEmail} onChange={event=>setInviteEmail(event.target.value)} required/></label><label>Role<select value={inviteRoleId} onChange={event=>setInviteRoleId(event.target.value)}>{roles.filter(role=>role.role_key!=="owner").map(role=><option key={role.id} value={role.id}>{role.display_name}</option>)}</select></label><button type="button" disabled={busy||!inviteEmail||!inviteRoleId} onClick={()=>void run(async()=>{await cloudAdmin.invite(inviteEmail,inviteRoleId);setInviteEmail("");},"Invitation created and queued for delivery.")}>Send invitation</button></div>{invitations.map(item=><div className="admin-list-row" key={item.id}><span>{item.email_normalized} · {item.role_name} · {item.status}</span>{item.status==="pending"&&<button type="button" disabled={busy} onClick={()=>void run(()=>cloudAdmin.revokeInvitation(item.id),"Invitation revoked.")}>Revoke</button>}</div>)}</div>}
  {can("roles.manage")&&<div className="admin-subsection"><h4>Create a custom role</h4><div className="form-grid"><label>Role key<input value={roleKey} onChange={event=>setRoleKey(event.target.value.toLowerCase())} placeholder="senior_auditor"/></label><label>Display name<input value={roleName} onChange={event=>setRoleName(event.target.value)} placeholder="Senior auditor"/></label></div><div className="permission-grid">{catalogue.map(item=><label key={item.permission_key}><input type="checkbox" checked={newRolePermissions.includes(item.permission_key)} onChange={event=>setNewRolePermissions(current=>event.target.checked?[...current,item.permission_key]:current.filter(value=>value!==item.permission_key))}/><span><strong>{item.permission_key}</strong><small>{item.description}</small></span></label>)}</div><button type="button" disabled={busy||roleKey.length<2||roleName.trim().length<2} onClick={()=>void run(async()=>{await cloudAdmin.createRole({key:roleKey,name:roleName,permissions:newRolePermissions});setRoleKey("");setRoleName("");setNewRolePermissions([]);},"Custom role created.")}>Create role</button></div>}
  {can("roles.read")&&<div className="admin-subsection"><h4>Role definitions</h4><label>Role<select value={selectedRoleId} onChange={event=>setSelectedRoleId(event.target.value)}>{roles.map(role=><option key={role.id} value={role.id}>{role.display_name}{role.is_system?" (system)":""}</option>)}</select></label>{roles.filter(role=>role.id===selectedRoleId).map(role=><div key={role.id}><p>{role.role_key} · {role.is_system?"System role":"Custom role"}</p>{can("roles.manage")&&!role.is_system&&<><label>Permission grants</label><div className="permission-grid">{catalogue.map(item=><label key={item.permission_key}><input type="checkbox" checked={selectedRolePermissions.includes(item.permission_key)} onChange={event=>setSelectedRolePermissions(current=>event.target.checked?[...current,item.permission_key]:current.filter(value=>value!==item.permission_key))}/><span><strong>{item.permission_key}</strong><small>{item.description}</small></span></label>)}</div><button type="button" disabled={busy} onClick={()=>void run(()=>cloudAdmin.replaceRolePermissions(role.id,selectedRolePermissions),"Role permissions replaced.")}>Save permissions</button><button type="button" disabled={busy} onClick={()=>void run(()=>cloudAdmin.renameRole(role.id,roleName||role.display_name),"Role renamed.")}>Rename role</button><button type="button" disabled={busy} onClick={()=>void run(()=>cloudAdmin.deleteRole(role.id),"Role deleted.")}>Delete role</button></>}</div>)}</div>}
 </section>;
}
