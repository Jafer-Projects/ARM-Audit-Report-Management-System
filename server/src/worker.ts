import { randomUUID } from "node:crypto";
import { pool, inTransaction } from "./db.js";
import { config } from "./config.js";

const workerId = `arm-worker-${randomUUID()}`;
const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

type Job = { id: string; tenant_id: string | null; job_type: string; payload: Record<string, unknown>; attempts: number };

async function claimJob(): Promise<Job | null> {
  return inTransaction(async (client) => {
    await client.query("SELECT set_config('app.worker', 'true', true)");
    const result = await client.query(
      `SELECT id, tenant_id, job_type, payload, attempts
       FROM jobs
       WHERE (status = 'queued' AND available_at <= now())\n          OR (status = 'running' AND locked_at < now() - interval '5 minutes')
       ORDER BY available_at, created_at
       FOR UPDATE SKIP LOCKED LIMIT 1`
    );
    const job = result.rows[0] as Job | undefined;
    if (!job) return null;
    await client.query(
      "UPDATE jobs SET status = 'running', attempts = attempts + 1, locked_at = now(), locked_by = $2 WHERE id = $1",
      [job.id, workerId]
    );
    return { ...job, attempts: job.attempts + 1 };
  });
}

async function processJob(job: Job): Promise<void> {
  if (job.job_type === "system.health_check") return;
  throw new Error(`No handler registered for job type: ${job.job_type}`);
}

async function runOnce(): Promise<boolean> {
  const job = await claimJob();
  if (!job) return false;
  const heartbeat = setInterval(() => {
    void inTransaction(async (client) => {
      await client.query("SELECT set_config('app.worker', 'true', true)");
      await client.query("UPDATE jobs SET locked_at = now() WHERE id = $1 AND locked_by = $2 AND status = 'running'", [job.id, workerId]);
    }).catch((error) => console.error(JSON.stringify({ level: "error", workerId, jobId: job.id, event: "heartbeat_failed", error: error instanceof Error ? error.message : "unknown" })));
  }, 60_000);
  try {
    await processJob(job);
    await inTransaction(async (client) => {
      await client.query("SELECT set_config('app.worker', 'true', true)");
      await client.query(
        "UPDATE jobs SET status = 'completed', completed_at = now(), locked_at = NULL, locked_by = NULL, last_error = NULL WHERE id = $1 AND locked_by = $2",
        [job.id, workerId]
      );
    });
  } catch (error) {
    const message = error instanceof Error ? error.message.slice(0, 1000) : "Unknown worker error";
    const terminal = job.attempts >= 8;
    await inTransaction(async (client) => {
      await client.query("SELECT set_config('app.worker', 'true', true)");
      await client.query(
        `UPDATE jobs
         SET status = $2, available_at = now() + ($3::text || ' seconds')::interval,
             locked_at = NULL, locked_by = NULL, last_error = $4
         WHERE id = $1 AND locked_by = $5`,
        [job.id, terminal ? "failed" : "queued", String(Math.min(3600, 2 ** job.attempts * 5)), message, workerId]
      );
    });
    console.error(JSON.stringify({ level: "error", workerId, jobId: job.id, attempts: job.attempts, terminal, error: message }));
  } finally {
    clearInterval(heartbeat);
  }
  return true;
}

process.on("SIGTERM", () => { void pool.end().then(() => process.exit(0)); });
process.on("SIGINT", () => { void pool.end().then(() => process.exit(0)); });

console.log(JSON.stringify({ level: "info", workerId, event: "started", nodeEnv: config.NODE_ENV }));
while (true) {
  try {
    const worked = await runOnce();
    if (!worked) await sleep(1500);
  } catch (error) {
    console.error(JSON.stringify({ level: "error", workerId, event: "poll_failed", error: error instanceof Error ? error.message : "unknown" }));
    await sleep(3000);
  }
}
