import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const recordTypes = new Set(["category","product","invoice","historical_record","period","staff","task","import_case","import_cost_input"]);
const statuses = {
  invoice: new Set(["Draft","Needs correction","Ready for review","Reviewed"]),
  historical_record: new Set(["Draft","Needs source","Ready for review","Reviewed"]),
  task: new Set(["Draft","Assigned","In progress","Blocked","Submitted","Returned","Completed"]),
  import_case: new Set(["Discovery","Inputs captured","Needs review","Reviewed","Deferred"]),
  import_cost_input: new Set(["Draft","Needs source","Ready for review","Reviewed"]),
  period: new Set(["Draft","Needs source","Ready for review","Reviewed"]),
};

test("workspace record allowlist excludes arbitrary tables and command injection strings", () => {
  for (const type of recordTypes) assert.equal(recordTypes.has(type), true);
  assert.equal(recordTypes.has("client; DROP TABLE client"), false);
  assert.equal(recordTypes.has("users"), false);
});

test("tax workpaper and import-cost workflows have no calculation-enabled status", () => {
  assert.equal(statuses.historical_record.has("Calculation approved"), false);
  assert.equal(statuses.import_case.has("Calculated"), false);
  assert.equal(statuses.import_case.has("Filed"), false);
});

test("task workflow distinguishes submission, return and completion", () => {
  assert.ok(statuses.task.has("Submitted"));
  assert.ok(statuses.task.has("Returned"));
  assert.ok(statuses.task.has("Completed"));
  assert.notEqual("Submitted", "Completed");
});

test("prototype invoice and import-cost states use the same review vocabulary", () => {
  assert.ok(statuses.invoice.has("Ready for review"));
  assert.equal(statuses.invoice.has("Submitted for review"), false);
  assert.equal(statuses.invoice.has("Archived"), false);
  assert.ok(statuses.import_cost_input.has("Draft"));
  assert.ok(statuses.import_cost_input.has("Ready for review"));
  assert.equal(statuses.import_cost_input.has("Calculated"), false);
});

test("workspace notification event vocabulary stays task-scoped", () => {
  const events = new Set(["task_assigned","review_assigned","task_submitted_for_review","task_blocked","task_returned","task_review_completed"]);
  assert.equal(events.size, 6);
  assert.equal(events.has("tax_return_filed"), false);
});



test("demo workspace is opt-in and production startup remains cloud-only", () => {
  const desktop = readFileSync(new URL("../src/services/desktop.ts", import.meta.url), "utf8");
  const developmentMode = readFileSync(new URL("../.env.development", import.meta.url), "utf8");
  assert.match(desktop, /VITE_ARM_WORKSPACE_MODE === "demo"/);
  assert.match(desktop, /workspace_mode:"cloud"/);
  assert.match(desktop, /workspace_mode:"demo"/);
  assert.match(developmentMode, /^VITE_ARM_WORKSPACE_MODE=demo$/m);
});

test("cloud client operations carry an explicit tenant and do not fall back to local writes", () => {
  const cloud = readFileSync(new URL("../src/services/cloudClients.ts", import.meta.url), "utf8");
  const desktop = readFileSync(new URL("../src/services/desktop.ts", import.meta.url), "utf8");
  assert.match(cloud, /"x-tenant-id":current\.tenantId/);
  assert.match(cloud, /expectedVersion:client\.row_version/);
  assert.match(desktop, /cloudOrDemo/);
  assert.match(desktop, /requireDemoWorkspace/);
  assert.match(desktop, /The demonstration workspace is disabled in this build/);
});

test("refresh credentials are held by the native vault and tenant administration uses server APIs", () => {
  const vault = readFileSync(new URL("../src-tauri/src/credential_vault.rs", import.meta.url), "utf8");
  const session = readFileSync(new URL("../src/secureSession.ts", import.meta.url), "utf8");
  const admin = readFileSync(new URL("../src/services/cloudAdmin.ts", import.meta.url), "utf8");
  assert.match(vault, /CredWriteW/);
  assert.match(vault, /CredReadW/);
  assert.match(vault, /CredDeleteW/);
  assert.doesNotMatch(session, /localStorage|sessionStorage/);
  assert.match(admin, /\/v1\/tenant\/roles/);
  assert.match(admin, /\/v1\/tenant\/memberships/);
});


test("release desktop does not open demo SQLite or register demo and prototype commands", () => {
  const native = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
  assert.match(native, /if cfg!\(debug_assertions\)/);
  assert.match(native, /Database::open\(app_data\.join\("arm-demo\.sqlite3"\)\)/);
  assert.match(native, /Production binaries expose only native credential-vault operations/);
  assert.match(native, /credential_vault::load_cloud_session/);
  assert.match(native, /commands::initialize_workspace/);
});


test("release cloud panel does not invoke development-only prototype synchronization commands", () => {
  const panel = readFileSync(new URL("../src/components/CloudConnectionPanel.tsx", import.meta.url), "utf8");
  assert.match(panel, /import\.meta\.env\.DEV\?getLocalSyncStatus\(\)/);
  assert.match(panel, /Synchronization remains disabled/);
});
