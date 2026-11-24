import { buildApp } from "./app.js";
import { config } from "./config.js";

const app = await buildApp();
let shuttingDown = false;

async function shutdown(signal: string): Promise<void> {
  if (shuttingDown) return;
  shuttingDown = true;
  app.log.info({ signal }, "graceful shutdown started");
  try {
    await app.close();
    app.log.info({ signal }, "graceful shutdown completed");
  } catch (error) {
    app.log.error({ err: error, signal }, "graceful shutdown failed");
    process.exitCode = 1;
  }
}

process.on("SIGTERM", () => { void shutdown("SIGTERM"); });
process.on("SIGINT", () => { void shutdown("SIGINT"); });

try {
  await app.listen({ host: config.HOST, port: config.PORT });
} catch (error) {
  app.log.error(error);
  process.exitCode = 1;
  await shutdown("startup_failure");
}
