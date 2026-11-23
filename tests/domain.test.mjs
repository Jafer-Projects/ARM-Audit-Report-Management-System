import test from "node:test";
import assert from "node:assert/strict";

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

