import { inTransaction } from "./db.js";
import { hashOpaqueToken } from "./security.js";

export async function acceptOrganizationInvitation(userId:string,rawToken:string) {
  if(rawToken.length<32||rawToken.length>128) throw Object.assign(new Error("Invalid invitation token"),{statusCode:400});
  return inTransaction(async(client)=>{
    await client.query("SELECT set_config('app.user_id',$1,true)",[userId]);
    try {
      const accepted=await client.query("SELECT tenant_id,membership_id FROM public.arm_accept_invitation($1,$2)",[hashOpaqueToken(rawToken),userId]);
      return {accepted:true,...accepted.rows[0]};
    } catch(error) {
      if((error as {code?:string}).code==="P0002") throw Object.assign(new Error("Invitation is invalid, expired or revoked"),{statusCode:400});
      if((error as {code?:string}).code==="42501") throw Object.assign(new Error("Sign in with the verified email address that received this invitation"),{statusCode:403});
      throw error;
    }
  });
}
