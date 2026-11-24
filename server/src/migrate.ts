import { readdir, readFile } from "node:fs/promises";
import { join } from "node:path";
import { pool } from "./db.js";

const directory = new URL("../migrations/", import.meta.url);
const files = (await readdir(directory)).filter((name) => name.endsWith(".sql")).sort();
const client = await pool.connect();
try {
  await client.query("BEGIN");
  await client.query("CREATE TABLE IF NOT EXISTS schema_migrations (version text PRIMARY KEY, applied_at timestamptz NOT NULL DEFAULT now())");
  await client.query("COMMIT");
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
  client.release();
  await pool.end();
}
