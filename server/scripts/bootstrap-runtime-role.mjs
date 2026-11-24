import pg from "pg";

const adminUrl = process.env.ADMIN_DATABASE_URL ?? process.env.MIGRATION_DATABASE_URL;
const runtimePassword = process.env.ARM_RUNTIME_PASSWORD;
const workerPassword = process.env.ARM_WORKER_PASSWORD;
if (!adminUrl || !runtimePassword || runtimePassword.length < 24 || !workerPassword || workerPassword.length < 24) {
  throw new Error("Admin DB URL and distinct runtime/worker passwords (at least 24 characters each) are required");
}
const pool = new pg.Pool({ connectionString: adminUrl, max: 1 });
try {
  const escaped = await pool.query("SELECT quote_literal($1) AS runtime, quote_literal($2) AS worker", [runtimePassword, workerPassword]);
  const runtimeLiteral = escaped.rows[0].runtime;
  const workerLiteral = escaped.rows[0].worker;
  await pool.query(\`DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'arm_runtime') THEN
      CREATE ROLE arm_runtime LOGIN PASSWORD \${runtimeLiteral} NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOREPLICATION NOBYPASSRLS;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'arm_worker') THEN
      CREATE ROLE arm_worker LOGIN PASSWORD \${workerLiteral} NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOREPLICATION NOBYPASSRLS;
    END IF;
  END $$\`);
  await pool.query(\`ALTER ROLE arm_runtime WITH LOGIN PASSWORD \${runtimeLiteral} NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOREPLICATION NOBYPASSRLS\`);
  await pool.query(\`ALTER ROLE arm_worker WITH LOGIN PASSWORD \${workerLiteral} NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOREPLICATION NOBYPASSRLS\`);
  const db = (await pool.query("SELECT quote_ident(current_database()) AS name")).rows[0].name;
  await pool.query(\`GRANT CONNECT ON DATABASE \${db} TO arm_runtime, arm_worker\`);
  await pool.query("GRANT USAGE ON SCHEMA public TO arm_runtime, arm_worker");
  await pool.query("GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO arm_runtime");
  await pool.query("GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO arm_runtime");
  await pool.query("ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO arm_runtime");
  await pool.query("ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT USAGE, SELECT ON SEQUENCES TO arm_runtime");
  process.stdout.write("Configured non-owner, non-superuser, NOBYPASSRLS runtime and worker roles\\n");
} finally {
  await pool.end();
}
