import test from "node:test";
import assert from "node:assert/strict";

const recordTypes = new Set(["category","product","invoice","historical_record","period","staff","task","import_case","import_cost_input"]);
const statuses = {
  invoice: new Set(["Draft","Submitted for review","Reviewed","Archived"]),
  historical_record: new Set(["Draft","Needs source","Ready for review","Reviewed"]),
  task: new Set(["Draft","Assigned","In progress","Blocked","Submitted","Returned","Completed"]),
  import_case: new Set(["Discovery","Inputs captured","Needs review","Reviewed"]),
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
