import { readdir, readFile } from "node:fs/promises";
import pg from "pg";

const migrationUrl = process.env.MIGRATION_DATABASE_URL ?? process.env.DATABASE_URL;
if (!migrationUrl) throw new Error("MIGRATION_DATABASE_URL or DATABASE_URL is required");
const migrationPool = new pg.Pool({ connectionString: migrationUrl, max: 2 });
const directory = new URL("../migrations/", import.meta.url);
const files = (await readdir(directory)).filter((name) => name.endsWith(".sql")).sort();
const client = await migrationPool.connect();
let migrationLockAcquired = false;
try {
  await client.query("SELECT pg_advisory_lock(hashtext('arm-schema-migrations'))");
  migrationLockAcquired = true;
  await client.query("CREATE TABLE IF NOT EXISTS schema_migrations (version text PRIMARY KEY, applied_at timestamptz NOT NULL DEFAULT now())");
  for (const file of files) {
    const exists = await client.query("SELECT 1 FROM schema_migrations WHERE version = $1", [file]);
    if (exists.rowCount) continue;
    const sql = await readFile(new URL(file, directory), "utf8");
    await client.query("BEGIN");
    try {
      await client.query(sql);
      await client.query("INSERT INTO schema_migrations(version) VALUES ($1)", [file]);
      await client.query("COMMIT");
      process.stdout.write(`applied ${file}\n`);
    } catch (error) {
      await client.query("ROLLBACK");
      throw error;
    }
  }
} finally {
  if (migrationLockAcquired) await client.query("SELECT pg_advisory_unlock(hashtext('arm-schema-migrations'))");
  client.release();
  await migrationPool.end();
}
