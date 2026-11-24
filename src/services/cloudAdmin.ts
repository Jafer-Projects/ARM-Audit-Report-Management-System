import { secureSession } from "../secureSession";

export type TenantMember={membership_id:string;user_id:string;email:string;display_name:string;legacy_role:string;status:string;created_at:string;assigned_roles:string[]};
export type TenantRole={id:string;role_key:string;display_name:string;is_system:boolean;permissions:string[]};
export type Permission={permission_key:string;description:string};
export type TenantInvitation={id:string;email_normalized:string;status:string;expires_at:string;accepted_at:string|null;role_key:string;role_name:string};

async function request<T>(path:string,init:RequestInit={}):Promise<T>{
 const saved=await secureSession.load();
 if(!saved||!saved.selectedTenantId)throw new Error("Select an organization before managing its members or roles.");
 const base=new URL(saved.apiBaseUrl).toString().replace(/\/$/,"");
 const send=(token:string)=>fetch(base+path,{...init,headers:{"authorization":"Bearer "+token,"x-tenant-id":saved.selectedTenantId,"content-type":"application/json",...init.headers},cache:"no-store"});
 let response=await send(saved.session.accessToken);
 if(response.status===401){const updated=await secureSession.refresh();response=await send(updated.session.accessToken);}
 if(!response.ok){const body=await response.text();throw new Error("Tenant administration API returned "+response.status+": "+body.slice(0,400));}
 if(response.status===204)return undefined as T;
 return response.json() as Promise<T>;
}
export const cloudAdmin={
 context:()=>request<{id:string;name:string;slug:string;role:string;permissions:string[]}>("/v1/tenant/context"),
 members:async()=> (await request<{members:TenantMember[]}>("/v1/tenant/members")).members,
 roles:async()=> (await request<{roles:TenantRole[]}>("/v1/tenant/roles")).roles,
 permissions:async()=> (await request<{permissions:Permission[]}>("/v1/tenant/permissions")).permissions,
 invitations:async()=> (await request<{invitations:TenantInvitation[]}>("/v1/tenant/invitations")).invitations,
 createRole:(input:{key:string;name:string;permissions:string[]})=>request<TenantRole>("/v1/tenant/roles",{method:"POST",body:JSON.stringify(input)}),
 renameRole:(roleId:string,name:string)=>request<void>("/v1/tenant/roles/"+encodeURIComponent(roleId),{method:"PUT",body:JSON.stringify({name})}),
 replaceRolePermissions:(roleId:string,permissions:string[])=>request<void>("/v1/tenant/roles/"+encodeURIComponent(roleId)+"/permissions",{method:"PUT",body:JSON.stringify({permissions})}),
 deleteRole:(roleId:string)=>request<void>("/v1/tenant/roles/"+encodeURIComponent(roleId),{method:"DELETE"}),
 assignRole:(membershipId:string,roleId:string)=>request<void>("/v1/tenant/memberships/"+encodeURIComponent(membershipId)+"/roles",{method:"POST",body:JSON.stringify({roleId})}),
 removeRole:(membershipId:string,roleId:string)=>request<void>("/v1/tenant/memberships/"+encodeURIComponent(membershipId)+"/roles/"+encodeURIComponent(roleId),{method:"DELETE"}),
 changeMemberStatus:(membershipId:string,status:"active"|"suspended"|"revoked")=>request<void>("/v1/tenant/memberships/"+encodeURIComponent(membershipId)+"/status",{method:"PATCH",body:JSON.stringify({status})}),
 invite:(email:string,roleId:string)=>request<{id:string;expires_at:string}>("/v1/tenant/invitations",{method:"POST",body:JSON.stringify({email,roleId})}),
 revokeInvitation:(id:string)=>request<void>("/v1/tenant/invitations/"+encodeURIComponent(id)+"/revoke",{method:"POST"})
};
