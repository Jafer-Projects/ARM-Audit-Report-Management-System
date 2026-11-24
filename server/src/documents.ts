import { randomUUID } from "node:crypto";
import { createConnection } from "node:net";
import { withTenant } from "./db.js";
import { assertPermission } from "./authorization.js";
import { config } from "./config.js";
import { loadObject,storeObject,deleteObject,sha256 as sha } from "./objectStorage.js";

function detectMime(data:Buffer):string|null {
  if(data.subarray(0,5).toString("ascii")==="%PDF-") return "application/pdf";
  if(data.length>=8 && data.subarray(0,8).equals(Buffer.from([137,80,78,71,13,10,26,10]))) return "image/png";
  if(data.length>=3 && data[0]===0xff && data[1]===0xd8 && data[2]===0xff) return "image/jpeg";
  return null;
}

export async function uploadDocument(userId:string,tenantId:string,input:{fileName:string;mediaType:string;contentBase64:string;engagementId?:string;workpaperId?:string;reportId?:string}) {
  if(!/^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(input.contentBase64)) throw Object.assign(new Error("Invalid base64 document body"),{statusCode:400});
  if(!input.engagementId&&!input.workpaperId&&!input.reportId) throw Object.assign(new Error("Attach evidence to an engagement, workpaper or report"),{statusCode:400});
  const bytes=Buffer.from(input.contentBase64,"base64");
  if(bytes.length===0||bytes.length>52_428_800) throw Object.assign(new Error("Document must be between 1 byte and 50 MiB"),{statusCode:400});
  const detected=detectMime(bytes);
  if(!detected||detected!==input.mediaType) throw Object.assign(new Error("File signature does not match an allowed PDF, JPEG or PNG media type"),{statusCode:415});
  const fileName=input.fileName.replace(/[\\/\\x00-\\x1f]/g,"_").trim().slice(0,180);
  if(!fileName) throw Object.assign(new Error("A safe filename is required"),{statusCode:400});
  const id=randomUUID(),objectKey=`${tenantId}/${id}`;
  await storeObject(objectKey,bytes);
  try {
    const result=await withTenant(userId,tenantId,async(client)=>{
      await assertPermission(client,userId,tenantId,"documents.upload");
      if(input.engagementId) {
        const found=await client.query("SELECT 1 FROM audit_engagements WHERE tenant_id=$1 AND id=$2",[tenantId,input.engagementId]);
        if(!found.rowCount) throw Object.assign(new Error("Engagement not found"),{statusCode:404});
      }
      if(input.workpaperId) {
        const found=await client.query("SELECT 1 FROM audit_workpapers WHERE tenant_id=$1 AND id=$2",[tenantId,input.workpaperId]);
        if(!found.rowCount) throw Object.assign(new Error("Workpaper not found"),{statusCode:404});
      }
      if(input.reportId) {
        const found=await client.query("SELECT 1 FROM audit_reports WHERE tenant_id=$1 AND id=$2",[tenantId,input.reportId]);
        if(!found.rowCount) throw Object.assign(new Error("Report not found"),{statusCode:404});
      }
      const inserted=await client.query(
        "INSERT INTO document_objects(id,tenant_id,engagement_id,workpaper_id,report_id,object_key,original_filename,media_type,size_bytes,sha256,uploaded_by) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11) RETURNING id,scan_status,created_at",
        [id,tenantId,input.engagementId??null,input.workpaperId??null,input.reportId??null,objectKey,fileName,detected,bytes.length,sha(bytes),userId]
      );
      await client.query("INSERT INTO audit_domain_history(tenant_id,actor_id,entity_type,entity_id,action,after_data) VALUES($1,$2,'document',$3,'uploaded',$4::jsonb)",[tenantId,userId,id,JSON.stringify({fileName,mediaType:detected,sizeBytes:bytes.length,sha256:sha(bytes)})]);
      await client.query("INSERT INTO audit_events(tenant_id,actor_id,action,target_type,target_id,metadata) VALUES($1,$2,'document.uploaded','document',$3,$4::jsonb)",[tenantId,userId,id,JSON.stringify({fileName,mediaType:detected,sizeBytes:bytes.length,sha256:sha(bytes)})]);
      return inserted.rows[0];
    });
    return result;
  } catch(error) {
    await deleteObject(objectKey).catch(()=>undefined);
    throw error;
  }
}
export async function readCleanDocument(userId:string,tenantId:string,id:string) {
  return withTenant(userId,tenantId,async(client)=>{
    await assertPermission(client,userId,tenantId,"documents.read");
    const found=await client.query("SELECT id,object_key,original_filename,media_type,size_bytes,sha256,scan_status FROM document_objects WHERE tenant_id=$1 AND id=$2",[tenantId,id]);
    const row=found.rows[0];
    if(!row) throw Object.assign(new Error("Document not found"),{statusCode:404});
    if(row.scan_status!=="clean") throw Object.assign(new Error("Document is unavailable until malware scanning passes"),{statusCode:423});
    const bytes=await loadObject(row.object_key);
    if(sha(bytes)!==row.sha256) throw Object.assign(new Error("Stored document integrity check failed"),{statusCode:500});
    return {...row,contentBase64:bytes.toString("base64")};
  });
}

export async function scanNextDocument(workerPool: {connect:()=>Promise<any>},clamHost:string,clamPort:number):Promise<boolean> {
  const client=await workerPool.connect();
  let document:any;
  try {
    await client.query("BEGIN");
    await client.query("SELECT set_config('app.worker','true',true)");
    const found=await client.query("SELECT id,object_key,sha256 FROM document_objects WHERE scan_status='pending' ORDER BY created_at FOR UPDATE SKIP LOCKED LIMIT 1");
    document=found.rows[0];
    if(!document) { await client.query("COMMIT"); return false; }
    await client.query("UPDATE document_objects SET scan_status='scanning' WHERE id=$1",[document.id]);
    await client.query("COMMIT");
  } catch(error) { await client.query("ROLLBACK"); throw error; }
  finally { client.release(); }
  let status="failed",detail="Scanner unavailable";
  try {
    const bytes=await loadObject(document.object_key);
    if(sha(bytes)!==document.sha256) throw new Error("SHA-256 mismatch before scanning");
    const verdict=await clamScan(bytes,clamHost,clamPort);
    status=verdict==="OK"?"clean":"infected";
    detail=verdict;
  } catch(error) { detail=error instanceof Error?error.message.slice(0,500):"Scanner error"; }
  const update=await workerPool.connect();
  try {
    await update.query("BEGIN");
    await update.query("SELECT set_config('app.worker','true',true)");
    await update.query("UPDATE document_objects SET scan_status=$2,scanned_at=now(),scan_detail=$3 WHERE id=$1 AND scan_status='scanning'",[document.id,status,detail]);
    await update.query("COMMIT");
  } catch(error) { await update.query("ROLLBACK"); throw error; }
  finally { update.release(); }
  return true;
}

function clamScan(bytes:Buffer,host:string,port:number):Promise<string> {
  return new Promise((resolvePromise,reject)=>{
    const socket=createConnection({host,port});
    let response="";
    socket.setTimeout(120_000);
    socket.on("connect",()=>{
      socket.write("zINSTREAM\\0");
      for(let offset=0;offset<bytes.length;offset+=64*1024) {
        const chunk=bytes.subarray(offset,Math.min(bytes.length,offset+64*1024));
        const length=Buffer.alloc(4); length.writeUInt32BE(chunk.length); socket.write(length); socket.write(chunk);
      }
      socket.write(Buffer.alloc(4));
    });
    socket.on("data",(chunk:Buffer)=>{response+=chunk.toString("utf8");});
    socket.on("end",()=>resolvePromise(response.replace(/\\0/g,"").trim()));
    socket.on("timeout",()=>{socket.destroy();reject(new Error("ClamAV scan timed out"));});
    socket.on("error",reject);
  });
}

