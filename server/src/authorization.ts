import type { PoolClient } from "pg";
import { randomBytes } from "node:crypto";
import { withTenant } from "./db.js";
import { hashOpaqueToken } from "./security.js";

export async function assertPermission(client: PoolClient,userId: string,tenantId: string,permission: string): Promise<void> {
  const member = await client.query(
    "SELECT id,role FROM tenant_memberships WHERE tenant_id=$1 AND user_id=$2 AND status='active'",
    [tenantId,userId]
  );
  if (!member.rowCount) throw Object.assign(new Error("Active tenant membership required"),{statusCode:403});
  if (member.rows[0].role === "owner") return;
  const granted = await client.query(
    `SELECT 1 FROM tenant_membership_roles mr
     JOIN tenant_role_permissions rp ON rp.tenant_id=mr.tenant_id AND rp.role_id=mr.role_id
     WHERE mr.tenant_id=$1 AND mr.membership_id=$2 AND rp.permission_key=$3`,
    [tenantId,member.rows[0].id,permission]
  );
  if (!granted.rowCount) throw Object.assign(new Error("Permission denied"),{statusCode:403});
}

export async function listRoles(userId: string,tenantId: string) {
  return withTenant(userId,tenantId,async (client) => {
    await assertPermission(client,userId,tenantId,"roles.read");
    const roles = await client.query(
      `SELECT r.id,r.role_key,r.display_name,r.is_system,
       COALESCE(array_agg(rp.permission_key) FILTER(WHERE rp.permission_key IS NOT NULL),'{}') AS permissions
       FROM tenant_roles r LEFT JOIN tenant_role_permissions rp ON rp.tenant_id=r.tenant_id AND rp.role_id=r.id
       WHERE r.tenant_id=$1 GROUP BY r.id ORDER BY r.is_system DESC,r.display_name`,
      [tenantId]
    );
    return roles.rows;
  });
}

export async function createRole(userId: string,tenantId: string,input: {key:string;name:string;permissions:string[]}) {
  return withTenant(userId,tenantId,async (client) => {
    await assertPermission(client,userId,tenantId,"roles.manage");
    if (input.key === "owner" || input.key === "platform_support") throw Object.assign(new Error("Reserved role key"),{statusCode:400});
    const known = await client.query("SELECT permission_key FROM permissions WHERE permission_key=ANY($1::text[])",[input.permissions]);
    if (known.rowCount !== new Set(input.permissions).size) throw Object.assign(new Error("Unknown or duplicate permission"),{statusCode:400});
    const role = await client.query("INSERT INTO tenant_roles(tenant_id,role_key,display_name) VALUES($1,$2,$3) RETURNING id,role_key,display_name",[tenantId,input.key,input.name]);
    for (const permission of input.permissions) {
      await client.query("INSERT INTO tenant_role_permissions(tenant_id,role_id,permission_key) VALUES($1,$2,$3)",[tenantId,role.rows[0].id,permission]);
    }
    await client.query("INSERT INTO audit_events(tenant_id,actor_id,action,target_type,target_id,metadata) VALUES($1,$2,'authorization.role.created','tenant_role',$3,$4::jsonb)",[tenantId,userId,role.rows[0].id,JSON.stringify({key:input.key,permissions:input.permissions})]);
    return {...role.rows[0],permissions:input.permissions};
  });
}

export async function replaceRolePermissions(userId: string,tenantId: string,roleId: string,permissions: string[]) {
  return withTenant(userId,tenantId,async (client) => {
    await assertPermission(client,userId,tenantId,"roles.manage");
    const role = await client.query("SELECT is_system,role_key FROM tenant_roles WHERE tenant_id=$1 AND id=$2 FOR UPDATE",[tenantId,roleId]);
    if (!role.rowCount || role.rows[0].is_system || role.rows[0].role_key==="owner") throw Object.assign(new Error("System roles cannot be modified"),{statusCode:403});
    const known = await client.query("SELECT permission_key FROM permissions WHERE permission_key=ANY($1::text[])",[permissions]);
    if (known.rowCount !== new Set(permissions).size) throw Object.assign(new Error("Unknown or duplicate permission"),{statusCode:400});
    await client.query("DELETE FROM tenant_role_permissions WHERE tenant_id=$1 AND role_id=$2",[tenantId,roleId]);
    for (const permission of permissions) await client.query("INSERT INTO tenant_role_permissions(tenant_id,role_id,permission_key) VALUES($1,$2,$3)",[tenantId,roleId,permission]);
    await client.query("INSERT INTO audit_events(tenant_id,actor_id,action,target_type,target_id,metadata) VALUES($1,$2,'authorization.role.permissions_replaced','tenant_role',$3,$4::jsonb)",[tenantId,userId,roleId,JSON.stringify({permissions})]);
    return {roleId,permissions};
  });
}

export async function assignMemberRole(userId: string,tenantId: string,membershipId: string,roleId: string) {
  return withTenant(userId,tenantId,async (client) => {
    await assertPermission(client,userId,tenantId,"roles.manage");
    const member = await client.query("SELECT id FROM tenant_memberships WHERE tenant_id=$1 AND id=$2 AND status='active'",[tenantId,membershipId]);
    const role = await client.query("SELECT id,role_key FROM tenant_roles WHERE tenant_id=$1 AND id=$2",[tenantId,roleId]);
    if (!member.rowCount || !role.rowCount || role.rows[0].role_key==="owner") throw Object.assign(new Error("Membership or assignable role not found"),{statusCode:400});
    await client.query("INSERT INTO tenant_membership_roles(tenant_id,membership_id,role_id,assigned_by) VALUES($1,$2,$3,$4) ON CONFLICT DO NOTHING",[tenantId,membershipId,roleId,userId]);
    await client.query("INSERT INTO audit_events(tenant_id,actor_id,action,target_type,target_id,metadata) VALUES($1,$2,'authorization.member.role_assigned','membership',$3,$4::jsonb)",[tenantId,userId,membershipId,JSON.stringify({roleId})]);
    return {membershipId,roleId};
  });
}

export async function inviteOrganizationMember(userId: string,tenantId: string,email: string,roleId: string,publicAppUrl: string) {
  return withTenant(userId,tenantId,async (client) => {
    await assertPermission(client,userId,tenantId,"members.invite");
    const role = await client.query("SELECT id,role_key FROM tenant_roles WHERE tenant_id=$1 AND id=$2",[tenantId,roleId]);
    if (!role.rowCount || role.rows[0].role_key==="owner") throw Object.assign(new Error("Invalid invitation role"),{statusCode:400});
    const raw = randomBytes(32).toString("base64url");
    const invitation = await client.query(
      "INSERT INTO organization_invitations(tenant_id,email_normalized,role_id,token_hash,invited_by,expires_at) VALUES($1,lower(btrim($2)),$3,$4,$5,now()+interval '7 days') RETURNING id,expires_at",
      [tenantId,email,roleId,hashOpaqueToken(raw),userId]
    );
    await client.query(
      "INSERT INTO email_outbox(user_id,tenant_id,recipient,template_key,payload) VALUES($1,$2,$3,'organization_invitation',$4::jsonb)",
      [userId,tenantId,email,JSON.stringify({invitationUrl:`${publicAppUrl}/accept-invitation?token=${raw}`})]
    );
    await client.query("INSERT INTO audit_events(tenant_id,actor_id,action,target_type,target_id,metadata) VALUES($1,$2,'organization.invitation.created','organization_invitation',$3,$4::jsonb)",[tenantId,userId,invitation.rows[0].id,JSON.stringify({email,roleId})]);
    return invitation.rows[0];
  });
}

export async function listMembers(userId:string,tenantId:string) {
 return withTenant(userId,tenantId,async(client)=>{
  await assertPermission(client,userId,tenantId,"members.read");
  const result=await client.query(
   `SELECT m.id AS membership_id,m.user_id,u.email,u.display_name,m.role,m.status,m.created_at,
    COALESCE(array_agg(DISTINCT r.display_name) FILTER(WHERE r.id IS NOT NULL),'{}') AS assigned_roles
    FROM tenant_memberships m JOIN users u ON u.id=m.user_id
    LEFT JOIN tenant_membership_roles mr ON mr.tenant_id=m.tenant_id AND mr.membership_id=m.id
    LEFT JOIN tenant_roles r ON r.tenant_id=mr.tenant_id AND r.id=mr.role_id
    WHERE m.tenant_id=$1 GROUP BY m.id,u.email,u.display_name ORDER BY m.created_at`,
   [tenantId]
  );
  return result.rows;
 });
}
export async function listInvitations(userId:string,tenantId:string) {
 return withTenant(userId,tenantId,async(client)=>{
  await assertPermission(client,userId,tenantId,"members.read");
  const result=await client.query(
   "SELECT i.id,i.email_normalized,i.status,i.expires_at,i.accepted_at,r.role_key,r.display_name AS role_name FROM organization_invitations i JOIN tenant_roles r ON r.tenant_id=i.tenant_id AND r.id=i.role_id WHERE i.tenant_id=$1 ORDER BY i.created_at DESC",
   [tenantId]
  );
  return result.rows;
 });
}
export async function revokeInvitation(userId:string,tenantId:string,invitationId:string) {
 return withTenant(userId,tenantId,async(client)=>{
  await assertPermission(client,userId,tenantId,"members.invite");
  const result=await client.query("UPDATE organization_invitations SET status='revoked' WHERE tenant_id=$1 AND id=$2 AND status='pending' RETURNING id,status",[tenantId,invitationId]);
  if(!result.rowCount) throw Object.assign(new Error("Pending invitation not found"),{statusCode:404});
  await client.query("INSERT INTO audit_events(tenant_id,actor_id,action,target_type,target_id) VALUES($1,$2,'organization.invitation.revoked','organization_invitation',$3)",[tenantId,userId,invitationId]);
  return result.rows[0];
 });
}
export async function listPermissionCatalogue(userId:string,tenantId:string) {
 return withTenant(userId,tenantId,async(client)=>{
  await assertPermission(client,userId,tenantId,"roles.manage");
  const result=await client.query("SELECT permission_key,description FROM permissions ORDER BY permission_key");
  return result.rows;
 });
}
