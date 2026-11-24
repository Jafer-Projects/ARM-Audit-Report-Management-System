import pg from "pg";

const adminUrl = process.env.ADMIN_DATABASE_URL ?? process.env.MIGRATION_DATABASE_URL;
const runtimePassword = process.env.ARM_RUNTIME_PASSWORD;
const workerPassword = process.env.ARM_WORKER_PASSWORD;
if (!adminUrl || !runtimePassword || runtimePassword.length < 24 || !workerPassword || workerPassword.length < 24) {
  throw new Error("Admin DB URL and distinct runtime/worker passwords (at least 24 characters each) are required");
}
const pool = new pg.Pool({ connectionString: adminUrl, max: 1 });
try {
  const escaped = await pool.query("SELECT quote_literal($1) AS runtime, quote_literal($2) AS worker, quote_ident(current_database()) AS db", [runtimePassword, workerPassword]);
  const runtimeLiteral = escaped.rows[0].runtime;
  const workerLiteral = escaped.rows[0].worker;
  const databaseIdentifier = escaped.rows[0].db;

  const runtimeExists = await pool.query("SELECT 1 FROM pg_roles WHERE rolname = 'arm_runtime'");
  if (!runtimeExists.rowCount) {
    await pool.query("CREATE ROLE arm_runtime LOGIN PASSWORD " + runtimeLiteral + " NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOREPLICATION NOBYPASSRLS");
  } else {
    await pool.query("ALTER ROLE arm_runtime WITH LOGIN PASSWORD " + runtimeLiteral + " NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOREPLICATION NOBYPASSRLS");
  }
  const workerExists = await pool.query("SELECT 1 FROM pg_roles WHERE rolname = 'arm_worker'");
  if (!workerExists.rowCount) {
    await pool.query("CREATE ROLE arm_worker LOGIN PASSWORD " + workerLiteral + " NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOREPLICATION NOBYPASSRLS");
  } else {
    await pool.query("ALTER ROLE arm_worker WITH LOGIN PASSWORD " + workerLiteral + " NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOREPLICATION NOBYPASSRLS");
  }
  await pool.query("GRANT CONNECT ON DATABASE " + databaseIdentifier + " TO arm_runtime, arm_worker");
  await pool.query("GRANT USAGE ON SCHEMA public TO arm_runtime, arm_worker");
  console.log("Configured non-owner, non-superuser, NOBYPASSRLS runtime and worker roles");
} finally {
  await pool.end();
}
