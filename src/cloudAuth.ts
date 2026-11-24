export type CloudSession={
 accessToken:string;refreshToken:string;expiresIn:number;
 user:{id:string;email:string;displayName:string};
 memberships:Array<{tenantId:string;tenantName:string;role:string;status:string}>;
};
export type CloudLoginResult=CloudSession|{mfaRequired:true;challengeToken:string;expiresIn:number};
function baseUrl(value:string){
 const url=new URL(value);
 if(url.protocol!=="https:"&&!(url.protocol==="http:"&&["localhost","127.0.0.1"].includes(url.hostname))) throw new Error("Cloud identity requires HTTPS except local development.");
 if(url.username||url.password||url.search||url.hash) throw new Error("Cloud API URL cannot include credentials or query parameters.");
 return url.toString().replace(/\/$/,"");
}
async function request<T>(apiBaseUrl:string,path:string,body?:unknown,token?:string,tenantId?:string):Promise<T>{
 const headers:Record<string,string>={"content-type":"application/json"};
 if(token) headers.authorization="Bearer "+token;
 if(tenantId) headers["x-tenant-id"]=tenantId;
 const response=await fetch(baseUrl(apiBaseUrl)+path,{method:body===undefined?"GET":"POST",headers,body:body===undefined?undefined:JSON.stringify(body),cache:"no-store"});
 if(!response.ok){const detail=await response.text();throw new Error("Cloud identity API "+response.status+": "+detail.slice(0,400));}
 return response.json() as Promise<T>;
}
export const cloudAuth={
 register:(api:string,input:{email:string;displayName:string;password:string;organizationName:string})=>request<{user:{id:string};tenant:{id:string};verificationRequired:true}>(api,"/v1/auth/register",input),
 verifyEmail:(api:string,token:string)=>request<{verified:boolean}>(api,"/v1/auth/verify-email",{token}),
 resendVerification:(api:string,email:string)=>request<{accepted:boolean}>(api,"/v1/auth/verification/resend",{email}),
 requestRecovery:(api:string,email:string)=>request<{accepted:boolean}>(api,"/v1/auth/recovery/request",{email}),
 completeRecovery:(api:string,token:string,password:string)=>request<{reset:boolean;sessionsRevoked:boolean}>(api,"/v1/auth/recovery/complete",{token,password}),
 login:(api:string,email:string,password:string)=>request<CloudLoginResult>(api,"/v1/auth/login",{email,password}),
 completeMfa:(api:string,challengeToken:string,code:string)=>request<CloudSession>(api,"/v1/auth/mfa/complete",{challengeToken,code}),
 refresh:(api:string,refreshToken:string)=>request<CloudSession>(api,"/v1/auth/refresh",{refreshToken}),
 logout:(api:string,accessToken:string)=>request<{revoked:boolean}>(api,"/v1/auth/logout",{},accessToken),
 beginMfa:(api:string,accessToken:string,tenantId:string,password:string)=>request<{secret:string;otpauthUri:string}>(api,"/v1/auth/mfa/enroll",{password},accessToken,tenantId),
 confirmMfa:(api:string,accessToken:string,tenantId:string,code:string)=>request<{enabled:boolean;recoveryCodes:string[]}>(api,"/v1/auth/mfa/confirm",{code},accessToken,tenantId),
 disableMfa:(api:string,accessToken:string,tenantId:string,password:string,code:string)=>request<{disabled:boolean}>(api,"/v1/auth/mfa/disable",{password,code},accessToken,tenantId),
 tenantContext:(api:string,accessToken:string,tenantId:string)=>request<Record<string,unknown>>(api,"/v1/tenant/context",undefined,accessToken,tenantId),
 acceptInvitation:(api:string,accessToken:string,token:string)=>request<{accepted:boolean;tenant_id:string;membership_id:string}>(api,"/v1/tenant/invitations/accept",{token},accessToken)
};
