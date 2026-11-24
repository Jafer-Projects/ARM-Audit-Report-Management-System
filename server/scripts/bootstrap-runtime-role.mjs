import pg from "pg";

const adminUrl = process.env.ADMIN_DATABASE_URL;
const runtimePassword = process.env.ARM_RUNTIME_PASSWORD;
const workerPassword = process.env.ARM_WORKER_PASSWORD;
const migratorPassword = process.env.ARM_MIGRATOR_PASSWORD;
if (!adminUrl || !runtimePassword || runtimePassword.length < 24 ||
    !workerPassword || workerPassword.length < 24 ||
    !migratorPassword || migratorPassword.length < 24) {
  throw new Error("ADMIN_DATABASE_URL and distinct runtime, worker and migrator passwords (at least 24 characters each) are required");
}

const pool = new pg.Pool({ connectionString: adminUrl, max: 1 });
try {
  const escaped = await pool.query(
    "SELECT quote_literal($1) AS runtime, quote_literal($2) AS worker, quote_literal($3) AS migrator, quote_ident(current_database()) AS db",
    [runtimePassword, workerPassword, migratorPassword]
  );
  const { runtime, worker, migrator, db } = escaped.rows[0];

  for (const [role, password] of [["arm_runtime", runtime], ["arm_worker", worker], ["arm_migrator", migrator]]) {
    const exists = await pool.query("SELECT 1 FROM pg_roles WHERE rolname = $1", [role]);
    if (!exists.rowCount) {
      await pool.query("CREATE ROLE " + role + " LOGIN PASSWORD " + password + " NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOREPLICATION NOBYPASSRLS");
    } else {
      await pool.query("ALTER ROLE " + role + " WITH LOGIN PASSWORD " + password + " NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT NOREPLICATION NOBYPASSRLS");
    }
  }
  await pool.query("GRANT CONNECT ON DATABASE " + db + " TO arm_runtime, arm_worker, arm_migrator");
  await pool.query("ALTER SCHEMA public OWNER TO arm_migrator");
  await pool.query("GRANT USAGE, CREATE ON SCHEMA public TO arm_migrator");
  await pool.query("GRANT USAGE ON SCHEMA public TO arm_runtime, arm_worker");
  console.log("Configured isolated non-superuser API, worker and migration roles");
} finally {
  await pool.end();
}
