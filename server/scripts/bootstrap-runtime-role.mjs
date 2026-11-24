import pg from "pg";

const adminUrl = process.env.ADMIN_DATABASE_URL ?? process.env.MIGRATION_DATABASE_URL;
const password = process.env.ARM_RUNTIME_PASSWORD;
if (!adminUrl || !password || password.length < 24) {
  throw new Error("ADMIN_DATABASE_URL and ARM_RUNTIME_PASSWORD (at least 24 characters) are required");
}
const pool = new pg.Pool({ connectionString: adminUrl, max: 1 });
try {
  const escaped = await pool.query("SELECT quote_literal($1) AS value", [password]);
  const literal = escaped.rows[0].value;
  await pool.query(`DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'arm_runtime') THEN
      CREATE ROLE arm_runtime LOGIN PASSWORD ${literal} NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOREPLICATION NOBYPASSRLS;
    END IF;
  END $$`);
  await pool.query(`ALTER ROLE arm_runtime WITH LOGIN PASSWORD ${literal} NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOREPLICATION NOBYPASSRLS`);
  await pool.query("GRANT CONNECT ON DATABASE " + (await pool.query("SELECT current_database() AS name")).rows[0].name.replace(/[^a-zA-Z0-9_]/g, "") + " TO arm_runtime");
  await pool.query("GRANT USAGE ON SCHEMA public TO arm_runtime");
  await pool.query("GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO arm_runtime");
  await pool.query("GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO arm_runtime");
  await pool.query("ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO arm_runtime");
  await pool.query("ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT USAGE, SELECT ON SEQUENCES TO arm_runtime");
  process.stdout.write("arm_runtime role is configured as a non-owner, non-superuser, NOBYPASSRLS role\n");
} finally {
  await pool.end();
}
