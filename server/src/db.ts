import pg, { type PoolClient } from "pg";
import { config } from "./config.js";

export const pool = new pg.Pool({
  connectionString: config.DATABASE_URL,
  max: 12,
  idleTimeoutMillis: 30_000,
  connectionTimeoutMillis: 5_000,
  application_name: "arm-cloud-api"
});

export async function inTransaction<T>(work: (client: PoolClient) => Promise<T>): Promise<T> {
  const client = await pool.connect();
  try {
    await client.query("BEGIN");
    const result = await work(client);
    await client.query("COMMIT");
    return result;
  } catch (error) {
    await client.query("ROLLBACK");
    throw error;
  } finally {
    client.release();
  }
}

export async function withTenant<T>(
  userId: string,
  tenantId: string,
  work: (client: PoolClient) => Promise<T>
): Promise<T> {
  return inTransaction(async (client) => {
    await client.query("SELECT set_config('app.user_id', $1, true)", [userId]);
    await client.query("SELECT set_config('app.tenant_id', $1, true)", [tenantId]);
    const membership = await client.query(
      "SELECT 1 FROM tenant_memberships WHERE tenant_id = $1 AND user_id = $2 AND status = 'active'",
      [tenantId, userId]
    );
    if (membership.rowCount !== 1) {
      throw Object.assign(new Error("Tenant membership is not active"), { statusCode: 403 });
    }
    return work(client);
  });
}
