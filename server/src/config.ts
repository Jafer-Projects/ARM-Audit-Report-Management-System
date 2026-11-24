import { readFileSync } from "node:fs";
import { z } from "zod";

function secret(name:string):string|undefined {
  const file=process.env[`${name}_FILE`];
  if(file) return readFileSync(file,"utf8").trim();
  return process.env[name];
}
const env={
  ...process.env,
  DATABASE_URL:secret("DATABASE_URL"),
  JWT_SECRET:secret("JWT_SECRET"),
  MFA_ENCRYPTION_KEY:secret("MFA_ENCRYPTION_KEY"),
  EMAIL_PROVIDER_API_KEY:secret("EMAIL_PROVIDER_API_KEY"),
  S3_SECRET_ACCESS_KEY:secret("S3_SECRET_ACCESS_KEY")
};
const envSchema=z.object({
 NODE_ENV:z.enum(["development","test","production"]).default("development"),
 HOST:z.string().default("0.0.0.0"),
 PORT:z.coerce.number().int().min(1).max(65535).default(3000),
 DATABASE_URL:z.string().min(1),
 JWT_SECRET:z.string().min(32),
 MFA_ENCRYPTION_KEY:z.string().min(32),
 ACCESS_TOKEN_TTL_SECONDS:z.coerce.number().int().min(60).max(3600).default(900),
 LOG_LEVEL:z.enum(["fatal","error","warn","info","debug","trace","silent"]).default("info"),
 PUBLIC_APP_URL:z.string().url().default("http://localhost:1420"),
 CORS_ORIGINS:z.string().default("tauri://localhost,http://tauri.localhost,https://tauri.localhost,http://localhost:1420"),
 EMAIL_PROVIDER_URL:z.string().url().optional(),
 EMAIL_PROVIDER_API_KEY:z.string().min(1).optional(),
 EMAIL_FROM:z.string().email().default("no-reply@example.invalid"),
 OBJECT_STORAGE_DIR:z.string().default("./var/objects"),
 OBJECT_STORAGE_MODE:z.enum(["local","s3"]).default("local"),
 S3_ENDPOINT:z.string().url().optional(),
 S3_BUCKET:z.string().min(3).optional(),
 S3_REGION:z.string().default("us-east-1"),
 S3_ACCESS_KEY_ID:z.string().min(1).optional(),
 S3_SECRET_ACCESS_KEY:z.string().min(1).optional(),
 CLAMAV_HOST:z.string().default("clamav"),
 CLAMAV_PORT:z.coerce.number().int().min(1).max(65535).default(3310)
});
export const config=envSchema.superRefine((value,ctx)=>{
 if(value.OBJECT_STORAGE_MODE==="s3"){
  for(const key of ["S3_ENDPOINT","S3_BUCKET","S3_ACCESS_KEY_ID","S3_SECRET_ACCESS_KEY"] as const){if(!value[key]) ctx.addIssue({code:"custom",path:[key],message:"Required when OBJECT_STORAGE_MODE=s3"});}
 }
}).parse(env);
