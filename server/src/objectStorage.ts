import { createHash,createHmac } from "node:crypto";
import { mkdir,readFile,rename,rm,writeFile } from "node:fs/promises";
import { dirname,resolve } from "node:path";
import { config } from "./config.js";

const root=resolve(config.OBJECT_STORAGE_DIR);
const sha=(bytes:Buffer)=>createHash("sha256").update(bytes).digest("hex");
const hmac=(key:string|Buffer,value:string)=>createHmac("sha256",key).update(value).digest();
function encodedPath(value:string){return value.split("/").map((part)=>encodeURIComponent(part).replace(/[!'()*]/g,(c)=>"%"+c.charCodeAt(0).toString(16).toUpperCase())).join("/");}

function s3RequestUrl(key:string){
 if(!config.S3_ENDPOINT||!config.S3_BUCKET) throw new Error("S3 endpoint and bucket are required in S3 mode");
 const endpoint=new URL(config.S3_ENDPOINT);
 if(config.NODE_ENV==="production"&&endpoint.protocol!=="https:") throw new Error("Production object storage must use HTTPS");
 const prefix=endpoint.pathname.replace(/\/$/,"");
 const path=prefix+"/"+encodedPath(config.S3_BUCKET)+"/"+encodedPath(key);
 return new URL(endpoint.origin+path);
}
async function s3Request(method:"PUT"|"GET"|"DELETE",key:string,body?:Buffer):Promise<Buffer|null>{
 const url=s3RequestUrl(key);
 const now=new Date();
 const amzDate=now.toISOString().replace(/[:-]|\.\d{3}/g,"");
 const dateStamp=amzDate.slice(0,8);
 const payloadHash=sha(body??Buffer.alloc(0));
 const canonicalHeaders="host:"+url.host+"\n"+"x-amz-content-sha256:"+payloadHash+"\n"+"x-amz-date:"+amzDate+"\n";
 const signedHeaders="host;x-amz-content-sha256;x-amz-date";
 const canonicalRequest=[method,url.pathname,"",canonicalHeaders,signedHeaders,payloadHash].join("\n");
 const scope=dateStamp+"/"+config.S3_REGION+"/s3/aws4_request";
 const stringToSign="AWS4-HMAC-SHA256\n"+amzDate+"\n"+scope+"\n"+createHash("sha256").update(canonicalRequest).digest("hex");
 const kDate=hmac("AWS4"+config.S3_SECRET_ACCESS_KEY,dateStamp);
 const kRegion=hmac(kDate,config.S3_REGION);
 const kService=hmac(kRegion,"s3");
 const kSigning=hmac(kService,"aws4_request");
 const signature=createHmac("sha256",kSigning).update(stringToSign).digest("hex");
 const authorization="AWS4-HMAC-SHA256 Credential="+config.S3_ACCESS_KEY_ID+"/"+scope+", SignedHeaders="+signedHeaders+", Signature="+signature;
 const response=await fetch(url,{method,headers:{host:url.host,"x-amz-date":amzDate,"x-amz-content-sha256":payloadHash,authorization,...(body?{"content-type":"application/octet-stream"}:{})},body:body?new Uint8Array(body):undefined,signal:AbortSignal.timeout(60_000)});
 if(!response.ok) throw new Error("Private object storage request failed ("+response.status+")");
 if(method==="GET") return Buffer.from(await response.arrayBuffer());
 return null;
}

export async function storeObject(key:string,bytes:Buffer):Promise<void>{
 if(config.OBJECT_STORAGE_MODE==="s3"){await s3Request("PUT",key,bytes);return;}
 const path=localPath(key),temp=path+".tmp";
 await mkdir(dirname(path),{recursive:true,mode:0o700});
 await writeFile(temp,bytes,{flag:"wx",mode:0o600});
 await rename(temp,path);
}
export async function loadObject(key:string):Promise<Buffer>{
 if(config.OBJECT_STORAGE_MODE==="s3") return (await s3Request("GET",key))!;
 return readFile(localPath(key));
}
export async function deleteObject(key:string):Promise<void>{
 if(config.OBJECT_STORAGE_MODE==="s3"){await s3Request("DELETE",key);return;}
 await rm(localPath(key),{force:true});
}
function localPath(key:string){
 const path=resolve(root,key);
 if(!path.startsWith(root+"/")) throw new Error("Invalid object key");
 return path;
}
export { sha as sha256 };
