import { useEffect, useMemo, useState, type FormEvent } from "react";
import { desktop } from "./services/desktop";
import type { Client, ClientChange, EventItem, HistoricalImportBatch, HistoricalImportRow, ImportCostResult, LocalNote, Persona, TaskReviewNote, WorkspaceInfo, WorkspaceRecord, WorkspaceRecordVersion } from "./types";

type Section = "home" | "clients" | "catalogue" | "history" | "work" | "imports" | "activity";
const nav: { id: Section; label: string; eyebrow: string }[] = [
  { id: "home", label: "Overview", eyebrow: "Workspace" },
  { id: "clients", label: "Clients", eyebrow: "Increment 1" },
  { id: "catalogue", label: "Products & invoices", eyebrow: "Increment 2" },
  { id: "history", label: "Historical & VAT", eyebrow: "Increment 3" },
  { id: "work", label: "Staff & tasks", eyebrow: "Increment 4" },
  { id: "imports", label: "Import costing", eyebrow: "Increment 5" },
  { id: "activity", label: "Activity history", eyebrow: "Records" },
];
const recordTypes: Record<Exclude<Section, "home" | "clients" | "activity">, { type: string; label: string; statuses: string[]; hint: string }> = {
  catalogue: { type: "product", label: "Product / invoice record", statuses: ["Draft", "Needs correction", "Ready for review", "Reviewed"], hint: "Create categories, products and invoice records. Amounts are stored as entered text; no tax or accounting calculation is performed." },
  history: { type: "historical_record", label: "Historical record / workpaper", statuses: ["Draft", "Needs source", "Ready for review", "Reviewed"], hint: "Capture historical records and source references. VAT calculations remain disabled until an approved tax rule pack and independent test fixtures exist." },
  work: { type: "task", label: "Staff task", statuses: ["Draft", "Assigned", "In progress", "Blocked", "Submitted", "Returned", "Completed"], hint: "Record staff and task workflow demonstrations. Demo personas are not production authorization." },
  imports: { type: "import_case", label: "Import-cost worksheet", statuses: ["Discovery", "Inputs captured", "Needs review", "Reviewed", "Deferred"], hint: "Record the import case, source-backed goods and cost components, FX basis and allocation method. Prepare a draft management-cost allocation for review; this is not customs value or a statutory calculation." },
};
const subtypeOptions: Record<"catalogue" | "history" | "work" | "imports", { type: string; label: string }[]> = {
  catalogue: [{ type: "category", label: "Categories" }, { type: "product", label: "Products" }, { type: "invoice", label: "Invoices" }],
  history: [{ type: "period", label: "Periods" }, { type: "historical_record", label: "Historical records" }],
  work: [{ type: "staff", label: "Staff" }, { type: "task", label: "Tasks" }],
  imports: [{ type: "import_case", label: "Discovery workspace" }, { type: "import_cost_input", label: "Cost inputs" }],
};
type RecordField = {
  key: string;
  label: string;
  kind?: "select" | "date" | "textarea";
  options?: string[];
  placeholder?: string;
  wide?: boolean;
  readOnly?: boolean;
};

const statusOptions: Record<string, string[]> = {
  category: ["Draft","Active","Archived"],
  product: ["Draft","Active","Archived"],
  invoice: ["Draft","Needs correction","Ready for review","Reviewed","Archived"],
  period: ["Draft","Needs source","Ready for review","Reviewed"],
  historical_record: ["Draft","Needs source","Ready for review","Reviewed"],
  staff: ["Draft","Active","Inactive"],
  task: ["Draft","Assigned","In progress","Blocked","Submitted","Returned","Completed","Cancelled"],
  import_case: ["Discovery","Inputs captured","Needs review","Reviewed","Deferred"],
  import_cost_input: ["Draft","Needs source","Ready for review","Reviewed"],
};

const recordFields: Record<string, RecordField[]> = {
  category: [
    { key: "description", label: "Description", kind: "textarea", wide: true },
    { key: "parent_category_reference", label: "Parent category reference (optional)", placeholder: "Only if a practitioner confirms category hierarchy" },
  ],
  product: [
    { key: "category_reference", label: "Category reference (optional)" },
    { key: "description", label: "Description", kind: "textarea", wide: true },
    { key: "item_code", label: "Item code / SKU (optional; not assumed unique)" },
    { key: "unit_description", label: "Unit / quantity description (optional)" },
  ],
  invoice: [
    { key: "transaction_class", label: "Transaction class", kind: "select", options: ["Sales","Purchase"] },
    { key: "invoice_number", label: "Invoice number (source text)" },
    { key: "invoice_date", label: "Invoice date", kind: "date" },
    { key: "financial_period_reference", label: "Financial period/year reference (do not infer tax period)" },
    { key: "counterparty_name", label: "Customer / supplier name" },
    { key: "counterparty_tin", label: "Customer / supplier TIN (opaque optional text)" },
    { key: "vat_amount", label: "VAT amount as shown on source (text)" },
    { key: "currency_code", label: "Currency code as shown on source (text)" },
    { key: "quantity_text", label: "Quantity as shown (text; no unit assumptions)" },
    { key: "line_items_text", label: "Line item descriptions as shown on source", kind: "textarea", wide: true },
    { key: "line_note", label: "Line note / remark", kind: "textarea", wide: true },
    { key: "remark", label: "Invoice remark", kind: "textarea", wide: true },
    { key: "source_type", label: "Source type", kind: "select", options: ["Paper","PDF","Spreadsheet","Other"] },
    { key: "source_reference", label: "Source document reference" },
    { key: "source_filename", label: "Source filename (metadata only)" },
    { key: "source_received_date", label: "Source received date", kind: "date" },
    { key: "source_association_note", label: "Source association note", kind: "textarea", wide: true },
    { key: "reviewer_note", label: "Reviewer decision / correction note", kind: "textarea", wide: true },
  ],
  period: [
    { key: "period_label", label: "Period label" },
    { key: "period_type_as_stated", label: "Period type as stated (no statutory inference)" },
    { key: "start_date", label: "Start date", kind: "date" },
    { key: "end_date", label: "End date", kind: "date" },
    { key: "period_source_reference", label: "Source / authority for period definition" },
    { key: "calendar_policy_reference", label: "Approved calendar / period policy reference" },
    { key: "period_note", label: "Period note", kind: "textarea", wide: true },
  ],
  historical_record: [
    { key: "historical_class", label: "Record class", kind: "select", options: ["Purchase","Expense","Tax history","VAT carry-forward (source wording)","Import-related record","Other"] },
    { key: "transaction_date", label: "Transaction date", kind: "date" },
    { key: "description", label: "Description" },
    { key: "currency", label: "Currency as shown on source" },
    { key: "supplier", label: "Supplier / payee" },
    { key: "supplier_tin", label: "Supplier TIN (opaque optional text)" },
    { key: "vat_amount", label: "VAT amount as stated (text)" },
    { key: "invoice_reference", label: "Invoice reference" },
    { key: "period_reference", label: "Source period label (do not infer tax period)" },
    { key: "tax_type_label", label: "Tax type / label exactly as supplied" },
    { key: "tax_period_as_stated", label: "Tax period exactly as stated on source" },
    { key: "balance_label_as_stated", label: "Opening / closing / carry-forward label as stated (if supplied)" },
    { key: "entry_method", label: "Entry method", kind: "select", options: ["Manual","Imported"] },
    { key: "source_type", label: "Source type", kind: "select", options: ["Paper","PDF","Spreadsheet","Bank export","Other"] },
    { key: "source_reference", label: "Source document reference" },
    { key: "source_filename", label: "Source filename (import provenance)", readOnly: true },
    { key: "source_row", label: "Source row number (import provenance)", readOnly: true },
    { key: "import_batch_id", label: "Import batch ID (import provenance)", readOnly: true },
    { key: "exception_note", label: "Missing data / exception note", kind: "textarea", wide: true },
  ],
  staff: [
    { key: "staff_code", label: "Staff reference (optional)" },
    { key: "role_title", label: "Role / professional title" },
    { key: "email", label: "Work email (optional)" },
    { key: "staff_note", label: "Staff note", kind: "textarea", wide: true },
  ],
  task: [
    { key: "description", label: "Task description / expected output", kind: "textarea", wide: true },
    { key: "assignee", label: "Assigned preparer" },
    { key: "reviewer", label: "Assigned reviewer" },
    { key: "priority", label: "Priority", kind: "select", options: ["Low","Normal","High","Urgent"] },
    { key: "due_date", label: "Due date", kind: "date" },
    { key: "related_record_reference", label: "Related record reference" },
    { key: "completion_summary", label: "Completion summary", kind: "textarea", wide: true },
    { key: "blocker_explanation", label: "Blocker explanation", kind: "textarea", wide: true },
    { key: "reassignment_reason", label: "Assignee / reviewer change reason", kind: "textarea", wide: true },
    { key: "cancellation_reason", label: "Cancellation reason", kind: "textarea", wide: true },
  ],
  import_case: [
    { key: "discovery_stage", label: "Discovery stage", kind: "select", options: ["Workflow discovery","Source inventory","Cost component discovery","FX policy discovery","Allocation method discovery","Review and approval discovery","Unresolved questions","Deferred"] },
    { key: "workshop_date", label: "Workshop date", kind: "date" },
    { key: "participants", label: "Participants and roles", kind: "textarea", wide: true },
    { key: "event_trigger", label: "1. What event starts the calculation?", kind: "textarea", wide: true },
    { key: "unit_of_work", label: "2. Is the unit a shipment, declaration, invoice, item, container or other object?", kind: "textarea", wide: true },
    { key: "source_documents", label: "3. Which source documents are normally available?", kind: "textarea", wide: true },
    { key: "cost_components", label: "4. Which cost components are used, optional or case-specific?", kind: "textarea", wide: true },
    { key: "currency_breakdown", label: "5. Which amounts are ETB and which are foreign currency?", kind: "textarea", wide: true },
    { key: "fx_policy", label: "6. FX rate, source, date and authorizer", kind: "textarea", wide: true },
    { key: "allocation_method", label: "7. How are shared costs allocated across products/items?", kind: "textarea", wide: true },
    { key: "damaged_missing_handling", label: "8. Handling of damaged, missing, returned or partially received goods", kind: "textarea", wide: true },
    { key: "tax_duty_policy", label: "9. Taxes/duties included, excluded or recoverable and who confirms them", kind: "textarea", wide: true },
    { key: "required_output_reviewer", label: "10. Required output and reviewer", kind: "textarea", wide: true },
    { key: "missing_source_process", label: "11. What happens when source documents or FX rates are missing?", kind: "textarea", wide: true },
    { key: "result_purpose", label: "12. Is the result for inventory valuation, management pricing, tax, audit evidence or another purpose?", kind: "textarea", wide: true },
    { key: "shipment_identifier", label: "Shipment/import event identifier (as described, not assumed)" },
    { key: "supplier", label: "Supplier/source party" },
    { key: "relevant_dates", label: "Relevant dates (source wording)" },
    { key: "currency", label: "Currencies involved (as described)" },
    { key: "incoterms_rule", label: "Incoterms 2020 rule (as stated in the sales contract)" },
    { key: "named_place", label: "Incoterms named port/place (as stated in the contract)" },
    { key: "workflow_diagram_reference", label: "Approved workflow diagram reference" },
    { key: "field_dictionary_reference", label: "Field dictionary reference" },
    { key: "synthetic_example_reference", label: "Synthetic sample case and independently checked expected result reference" },
    { key: "component_policy_reference", label: "Cost component inclusion/exclusion policy reference" },
    { key: "fx_policy_reference", label: "FX source/date/authorization policy reference" },
    { key: "allocation_policy_reference", label: "Allocation method and worked example reference" },
    { key: "exception_matrix_reference", label: "Exception handling and review matrix reference" },
    { key: "unresolved_questions", label: "Unresolved legal/accounting questions and qualified owners", kind: "textarea", wide: true },
  ],
  import_cost_input: [
    { key: "entry_kind", label: "Input type", kind: "select", options: ["Goods line","Cost component"] },
    { key: "source_reference", label: "Source document / reference" },
    { key: "source_date", label: "Source date", kind: "date" },
    { key: "item_reference", label: "Goods item reference (Goods line)" },
    { key: "description", label: "Goods description" },
    { key: "quantity_micros", label: "Quantity in millionths (1 unit = 1000000)" },
    { key: "unit_price_minor", label: "Unit price in source-currency minor units" },
    { key: "weight_micros", label: "Weight allocation units (millionths, optional)" },
    { key: "volume_micros", label: "Volume allocation units (millionths, optional)" },
    { key: "component_reference", label: "Cost component reference" },
    { key: "category", label: "Cost category", kind: "select", options: ["Freight","Insurance","Packing","Loading / handling","Port / terminal","Customs brokerage","Customs duty","Inland transport","Bank / transfer charges","Non-recoverable tax or levy","Other direct cost"] },
    { key: "amount_minor", label: "Component amount in source-currency minor units" },
    { key: "currency_code", label: "Source currency code (three letters)" },
    { key: "fx_numerator", label: "FX numerator (base minor units)" },
    { key: "fx_denominator", label: "FX denominator (source minor units)" },
    { key: "fx_rate_source", label: "FX rate source / reference" },
    { key: "fx_rate_date", label: "FX rate date", kind: "date" },
    { key: "fx_authorized_by", label: "FX rate authorized / approved by" },
    { key: "allocation_basis", label: "Allocation basis", kind: "select", options: ["Direct","Quantity","Weight","Volume","Value"] },
    { key: "direct_item_reference", label: "Direct target item reference (if Direct)" },
    { key: "treatment", label: "Cost treatment", kind: "select", options: ["Capitalize","Exclude","Recoverable tax","Customs-value-only","Needs review"] },
    { key: "cost_policy_note", label: "Inclusion / exclusion and policy note", kind: "textarea", wide: true },
  ],
};

const csvMappingFields = [
  { key: "transaction_date", label: "Transaction date", required: false },
  { key: "description", label: "Description", required: true },
  { key: "amount_text", label: "Source amount (text)", required: true },
  { key: "currency", label: "Currency", required: false },
  { key: "supplier", label: "Supplier / payee", required: false },
  { key: "supplier_tin", label: "Supplier TIN (opaque)", required: false },
  { key: "vat_amount", label: "VAT amount as stated (text)", required: false },
  { key: "invoice_reference", label: "Invoice reference", required: false },
];
const personaLabels: Record<Persona, string> = {
  administrator: "Demo Firm Administrator", manager: "Demo Engagement Manager",
  auditor: "Demo Auditor", reviewer: "Demo Read-only Reviewer",
};

function parseCsv(text: string): string[][] {
  const source = text.replace(/^\uFEFF/, "");
  const rows: string[][] = [];
  let row: string[] = [];
  let field = "";
  let quoted = false;
  for (let i = 0; i < source.length; i += 1) {
    const char = source[i];
    if (quoted) {
      if (char === '"' && source[i + 1] === '"') { field += '"'; i += 1; }
      else if (char === '"') quoted = false;
      else field += char;
    } else if (char === '"' && field.length === 0) quoted = true;
    else if (char === ",") { row.push(field); field = ""; }
    else if (char === "\n" || char === "\r") {
      row.push(field); field = "";
      rows.push(row);
      row = [];
      if (char === "\r" && source[i + 1] === "\n") i += 1;
    } else field += char;
  }
  if (quoted) throw new Error("The CSV contains an unclosed quoted field.");
  if (field.length > 0 || row.length > 0) {
    row.push(field);
    if (row.some(cell => cell.trim() !== "")) rows.push(row);
  }
  if (rows.some(cells => cells.length > 50)) throw new Error("CSV rows may contain no more than 50 columns.");
  return rows;
}

function suggestedHeader(headers: string[], aliases: string[]): string {
  const normalized = headers.map(header => header.trim().toLowerCase());
  const index = normalized.findIndex(header => aliases.includes(header));
  return index >= 0 ? headers[index] : "";
}

function hasPossibleDuplicate(record: WorkspaceRecord): boolean {
  try { return JSON.parse(record.payload_json).possible_duplicate_reference === true; }
  catch { return false; }
}

function parseRecordPayload(record: WorkspaceRecord): Record<string, string> {
  try { return JSON.parse(record.payload_json) as Record<string, string>; }
  catch { return {}; }
}

function taskDueState(value: string): "Overdue" | "Due soon" | "Scheduled" | "No due date" | "Invalid date" {
  if (!value) return "No due date";
  const parts = value.split("-").map(Number);
  if (parts.length !== 3 || parts.some(part => !Number.isFinite(part))) return "Invalid date";
  if (parts[1] < 1 || parts[1] > 12 || parts[2] < 1 || parts[2] > 31) return "Invalid date";
  const due = new Date(parts[0], parts[1] - 1, parts[2], 23, 59, 59);
  if (Number.isNaN(due.getTime()) || due.getFullYear() !== parts[0] || due.getMonth() !== parts[1] - 1 || due.getDate() !== parts[2]) return "Invalid date";
  const today = new Date(); today.setHours(0, 0, 0, 0);
  const days = Math.ceil((due.getTime() - today.getTime()) / 86_400_000);
  if (days < 0) return "Overdue";
  if (days <= 7) return "Due soon";
  return "Scheduled";
}

function historicalClass(record: WorkspaceRecord): string {
  return parseRecordPayload(record).historical_class ?? "";
}

function importBatchId(record: WorkspaceRecord): string {
  try { return JSON.parse(record.payload_json).import_batch_id ?? ""; }
  catch { return ""; }
}

function formatSnapshot(value: string): string {
  try { return JSON.stringify(JSON.parse(value), null, 2); }
  catch { return value; }
}

export default function App() {
  const [workspace, setWorkspace] = useState<WorkspaceInfo | null>(null);
  const [section, setSection] = useState<Section>("home");
  const [clients, setClients] = useState<Client[]>([]);
  const [selectedClientId, setSelectedClientId] = useState("");
  const [records, setRecords] = useState<WorkspaceRecord[]>([]);
  const [staffRecords, setStaffRecords] = useState<WorkspaceRecord[]>([]);
  const [events, setEvents] = useState<EventItem[]>([]);
  const [notes, setNotes] = useState<LocalNote[]>([]);
  const [historyClient, setHistoryClient] = useState<Client | null>(null);
  const [clientChanges, setClientChanges] = useState<ClientChange[]>([]);
  const [importBatches, setImportBatches] = useState<HistoricalImportBatch[]>([]);
  const [taskReviewNotes, setTaskReviewNotes] = useState<TaskReviewNote[]>([]);
  const [recordVersions, setRecordVersions] = useState<WorkspaceRecordVersion[]>([]);
  const [reviewNoteCategory, setReviewNoteCategory] = useState("Missing evidence");
  const [reviewNoteDescription, setReviewNoteDescription] = useState("");
  const [reviewNoteReference, setReviewNoteReference] = useState("");
  const [reviewNoteRequired, setReviewNoteRequired] = useState("");
  const [reviewResponses, setReviewResponses] = useState<Record<string, string>>({});
  const [showImportModal, setShowImportModal] = useState(false);
  const [csvFileName, setCsvFileName] = useState("");
  const [csvContent, setCsvContent] = useState("");
  const [csvHeaders, setCsvHeaders] = useState<string[]>([]);
  const [csvRows, setCsvRows] = useState<string[][]>([]);
  const [csvMapping, setCsvMapping] = useState<Record<string, string>>({});
  const [noteEditor, setNoteEditor] = useState<LocalNote | null | undefined>(undefined);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [search, setSearch] = useState("");
  const [includeArchived, setIncludeArchived] = useState(false);
  const [clientEditor, setClientEditor] = useState<Client | null | undefined>(undefined);
  const [recordEditor, setRecordEditor] = useState<WorkspaceRecord | null>(null);
  const [recordType, setRecordType] = useState("product");
  const [invoiceQueueOnly, setInvoiceQueueOnly] = useState(false);
  const [workDashboard, setWorkDashboard] = useState(false);
  const [workloadView, setWorkloadView] = useState(false);
  const [taskAssigneeFilter, setTaskAssigneeFilter] = useState("All");
  const [taskPriorityFilter, setTaskPriorityFilter] = useState("All");
  const [taskStatusFilter, setTaskStatusFilter] = useState("All");
  const [taskDueFilter, setTaskDueFilter] = useState("All");
  const [importCostCaseReference, setImportCostCaseReference] = useState("");
  const [importCostBaseCurrency, setImportCostBaseCurrency] = useState("ETB");
  const [importCostPreview, setImportCostPreview] = useState<ImportCostResult | null>(null);

  async function refresh(nextSection = section) {
    try {
      setError("");
      const info = await desktop.initialize();
      setWorkspace(info);
      setNotes(await desktop.notes());
      if (nextSection === "clients") setClients(await desktop.clients(includeArchived));
      if (nextSection === "activity") setEvents(await desktop.events());
      if (["catalogue", "history", "work", "imports"].includes(nextSection)) setRecords(await desktop.records(recordType, recordType === "staff" ? undefined : selectedClientId || undefined));
    } catch (e) { setError(String(e)); }
  }
  useEffect(() => { void refresh("home"); }, []);
  useEffect(() => { if (section === "clients" || section === "activity") void refresh(section); }, [section, includeArchived]);
  useEffect(() => {
    const config = recordTypes[section as keyof typeof recordTypes];
    if (config) setRecordType(config.type);
  }, [section]);
  useEffect(() => { setWorkDashboard(section === "work"); setWorkloadView(false); if (section === "work") setRecordType("task"); }, [section]);

  useEffect(() => {
    if (["catalogue", "history", "work", "imports"].includes(section)) {
      void desktop.clients(false).then(items => { setClients(items); setSelectedClientId(current => items.some(item => item.id === current) ? current : items[0]?.id || ""); }).catch((e) => setError(String(e)));
      if (section === "work" && ["administrator","manager"].includes(workspace?.persona ?? "")) void desktop.records("staff").then(setStaffRecords).catch((e) => setError(String(e))); else setStaffRecords([]);
      if (recordType === "staff" || selectedClientId) {
        void desktop.records(recordType, recordType === "staff" ? undefined : selectedClientId).then(setRecords).catch((e) => setError(String(e)));
        if (section === "history" && recordType === "historical_record" && selectedClientId) void desktop.importBatches(selectedClientId).then(setImportBatches).catch((e) => setError(String(e)));
      } else { setRecords([]); }
    }
  }, [recordType, section, selectedClientId, workspace?.persona]);

  const recordPayload = useMemo<Record<string, string>>(() => {
    if (!recordEditor) return {};
    return parseRecordPayload(recordEditor);
  }, [recordEditor]);
  const taskContentLocked = Boolean(recordEditor?.record_type === "task" && ["Submitted","Completed","Cancelled"].includes(recordEditor.status));
  const taskStatusLocked = Boolean(recordEditor?.record_type === "task" && (["Completed","Cancelled"].includes(recordEditor.status) || (recordEditor.status === "Submitted" && workspace?.persona === "auditor")));
  const taskCancellationReasonEditable = Boolean(recordEditor?.record_type === "task" && recordEditor.status === "Submitted" && ["manager","administrator"].includes(workspace?.persona ?? ""));
  const taskReviewerReassignEditable = Boolean(recordEditor?.record_type === "task" && recordEditor.status === "Submitted" && ["manager","administrator"].includes(workspace?.persona ?? ""));
  const invoiceContentLocked = Boolean(recordEditor?.record_type === "invoice" && (["Ready for review","Reviewed"].includes(recordEditor.status) || (recordEditor.status === "Needs correction" && workspace?.persona !== "auditor")));
  const invoiceStatusLocked = Boolean(recordEditor?.record_type === "invoice" && (recordEditor.status === "Reviewed" || (recordEditor.status === "Ready for review" && workspace?.persona === "auditor") || (recordEditor.status === "Needs correction" && workspace?.persona !== "auditor")));
  const invoiceReviewNoteEditable = Boolean(recordEditor?.record_type === "invoice" && recordEditor.status === "Ready for review" && ["manager","administrator"].includes(workspace?.persona ?? ""));
  const calculationResultLocked = Boolean(recordEditor?.record_type === "import_cost_input" && recordPayload.entry_kind === "calculation_result");
  const recordContentLocked = taskContentLocked || invoiceContentLocked || calculationResultLocked;


  const visibleRecords = useMemo(() => recordType === "invoice" && invoiceQueueOnly ? records.filter(record => record.status === "Ready for review") : records, [records, recordType, invoiceQueueOnly]);
  const taskAssignees = useMemo(() => ["All", ...Array.from(new Set(records.filter(record => record.record_type === "task").map(record => parseRecordPayload(record).assignee).filter((name): name is string => Boolean(name))))], [records]);
  const visibleTasks = useMemo(() => records.filter(record => record.record_type === "task")
    .filter(record => taskAssigneeFilter === "All" || parseRecordPayload(record).assignee === taskAssigneeFilter)
    .filter(record => taskPriorityFilter === "All" || parseRecordPayload(record).priority === taskPriorityFilter)
    .filter(record => taskStatusFilter === "All" || record.status === taskStatusFilter)
    .filter(record => taskDueFilter === "All" || taskDueState(parseRecordPayload(record).due_date) === taskDueFilter), [records, taskAssigneeFilter, taskPriorityFilter, taskStatusFilter, taskDueFilter]);

  const filteredClients = useMemo(() => clients.filter((c) =>
    [c.legal_name, c.display_name, c.tin_raw].join(" ").toLowerCase().includes(search.toLowerCase())), [clients, search]);
  const canMutate = workspace?.persona !== "reviewer";
  const canEditClientProfiles = canMutate;
  const canManageClientLifecycle = canMutate && ["administrator","manager"].includes(workspace?.persona ?? "");

  async function switchPersona(persona: Persona) {
    try { setBusy(true); setWorkspace(await desktop.setPersona(persona)); if (section === "work" && !["administrator","manager"].includes(persona)) { setRecordType("task"); setWorkDashboard(true); } await refresh(); }
    catch (e) { setError(String(e)); } finally { setBusy(false); }
  }
  async function saveClient(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    const input = Object.fromEntries(data.entries()) as Record<string, string>;
    try {
      setBusy(true); setError("");
      const duplicateCandidates = await desktop.findClientDuplicates(input.legal_name, input.tin_raw ?? "", clientEditor?.id);
      let duplicateAcknowledged = false;
      if (duplicateCandidates.length > 0) {
        const names = duplicateCandidates.map(candidate => `• ${candidate.legal_name} (${candidate.record_kind}, TIN: ${candidate.tin_raw || "not provided"}, ${candidate.status})`).join("\n");
        duplicateAcknowledged = window.confirm(`Possible existing client records were found:\n\n${names}\n\nNo records will be merged. Continue saving this as a separate client and record that you acknowledged the possible duplicate?`);
        if (!duplicateAcknowledged) return;
      }
      await desktop.saveClient({
        ...input, id: clientEditor?.id, row_version: clientEditor?.row_version, duplicate_acknowledged: duplicateAcknowledged,
        record_kind: input.record_kind, legal_name: input.legal_name,
        display_name: input.display_name ?? "", tin_raw: input.tin_raw ?? "", sector: input.sector ?? "",
        contact_name: input.contact_name ?? "", contact_email: input.contact_email ?? "",
        contact_phone: input.contact_phone ?? "", address_text: input.address_text ?? "",
        internal_reference: input.internal_reference ?? "", notes: input.notes ?? "",
      });
      setClientEditor(undefined); setClients(await desktop.clients(includeArchived));
    } catch (e) { setError(String(e)); } finally { setBusy(false); }
  }
  async function openRecord(record: WorkspaceRecord) {
    setRecordEditor(record);
    setTaskReviewNotes([]); setRecordVersions([]);
    try { setRecordVersions(await desktop.recordVersions(record.id)); } catch (e) { setError(String(e)); }
    if (record.record_type === "task") {
      try { setTaskReviewNotes(await desktop.taskReviewNotes(record.id)); }
      catch (e) { setError(String(e)); }
    }
  }
  async function addReviewNote() {
    if (!recordEditor?.id) return;
    try {
      setBusy(true); setError("");
      await desktop.addTaskReviewNote({ taskId: recordEditor.id, category: reviewNoteCategory, description: reviewNoteDescription, affectedRecordReference: reviewNoteReference, requiredResponse: reviewNoteRequired });
      setTaskReviewNotes(await desktop.taskReviewNotes(recordEditor.id));
      setReviewNoteDescription(""); setReviewNoteReference(""); setReviewNoteRequired("");
    } catch (e) { setError(String(e)); } finally { setBusy(false); }
  }
  async function respondReviewNote(note: TaskReviewNote) {
    const response = reviewResponses[note.id] ?? "";
    try {
      setBusy(true); setError("");
      await desktop.respondTaskReviewNote(note.id, response);
      setTaskReviewNotes(await desktop.taskReviewNotes(note.task_id));
      setReviewResponses(current => ({ ...current, [note.id]: "" }));
    } catch (e) { setError(String(e)); } finally { setBusy(false); }
  }
  async function disposeReviewNote(note: TaskReviewNote, decision: "Cleared" | "Reopened") {
    const reason = window.prompt(decision === "Cleared" ? "Record why this review note is cleared:" : "Record why this review note is reopened:", "");
    if (!reason || reason.trim().length < 3) return;
    try {
      setBusy(true); setError("");
      await desktop.dispositionTaskReviewNote(note.id, decision, reason.trim());
      setTaskReviewNotes(await desktop.taskReviewNotes(note.task_id));
    } catch (e) { setError(String(e)); } finally { setBusy(false); }
  }

  async function openClientHistory(client: Client) {
    try { setBusy(true); setError(""); setClientChanges(await desktop.clientChanges(client.id)); setHistoryClient(client); }
    catch (e) { setError(String(e)); } finally { setBusy(false); }
  }
  async function activateClient(client: Client) {
    if (!window.confirm(`Activate ${client.legal_name}? This changes only the local demo profile status.`)) return;
    try { await desktop.activateClient(client.id); setClients(await desktop.clients(includeArchived)); }
    catch (e) { setError(String(e)); }
  }
  async function restoreClient(client: Client) {
    if (!window.confirm(`Restore ${client.legal_name} to the active demo register?`)) return;
    try { await desktop.restoreClient(client.id); setClients(await desktop.clients(includeArchived)); }
    catch (e) { setError(String(e)); }
  }
  async function archiveClient(client: Client) {
    if (!window.confirm(`Archive ${client.legal_name}? This is reversible in the prototype and is not legal deletion.`)) return;
    try { await desktop.archiveClient(client.id); setClients(await desktop.clients(includeArchived)); }
    catch (e) { setError(String(e)); }
  }
  async function saveRecord(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    const v = Object.fromEntries(data.entries()) as Record<string, string>;
    const config = recordTypes[section as keyof typeof recordTypes];
    if (!config) return;
    try {
      setBusy(true); setError("");
      let duplicateAcknowledged = false;
      let duplicateReason = "";
      if (recordType === "invoice" && selectedClientId && v.invoice_number?.trim()) {
        const candidates = await desktop.findInvoiceDuplicates(selectedClientId, v.invoice_number, recordEditor?.id || undefined);
        if (candidates.length > 0) {
          const candidateText = candidates.map(item => `• ${item.invoice_number} · ${item.invoice_date || "date not captured"} · ${item.title} · ${item.status}`).join("\n");
          const proceed = window.confirm(`Possible duplicate invoice number found for this client:\n\n${candidateText}\n\nARM will not merge or overwrite either invoice. Continue as a separate invoice?`);
          if (!proceed) return;
          const reason = window.prompt("Record why this is a separate invoice (3–500 characters):", "");
          if (!reason || reason.trim().length < 3 || reason.trim().length > 500) {
            setError("A 3–500 character reason is required to acknowledge a possible duplicate.");
            return;
          }
          duplicateAcknowledged = true;
          duplicateReason = reason.trim();
        }
      }
      const fields = recordFields[recordType] ?? [];
      const lockedPayload = { ...recordPayload, ...Object.fromEntries(fields.map(field => [field.key, (field.key === "cancellation_reason" && taskCancellationReasonEditable || field.key === "reviewer" && taskReviewerReassignEditable || field.key === "reassignment_reason" && taskReviewerReassignEditable || field.key === "reviewer_note" && invoiceReviewNoteEditable) && v[field.key] !== undefined ? v[field.key] : recordPayload[field.key] ?? ""])) };
      const editablePayload = Object.fromEntries(fields.map(field => {
        const preserve = field.readOnly
          || (recordType === "task" && workspace?.persona === "auditor" && ["assignee","reviewer","priority","due_date","reassignment_reason","cancellation_reason"].includes(field.key))
          || (recordType === "invoice" && field.key === "reviewer_note" && !invoiceReviewNoteEditable);
        return [field.key, preserve ? recordPayload[field.key] ?? "" : v[field.key] ?? ""];
      }));
      await desktop.saveRecord({
        duplicate_acknowledged: duplicateAcknowledged, duplicate_acknowledgement_reason: duplicateReason,
        id: recordEditor?.id || undefined, row_version: recordEditor?.id ? recordEditor.row_version : undefined,
        client_id: recordEditor?.id ? recordEditor.client_id ?? null : v.client_id || null,
        record_type: recordType,
        title: recordContentLocked ? recordEditor?.title ?? "" : v.title ?? "",
        status: v.status || recordEditor?.status || (statusOptions[recordType] ?? config.statuses)[0],
        reference: recordContentLocked ? recordEditor?.reference ?? "" : v.reference || "",
        amount_text: recordContentLocked ? recordEditor?.amount_text ?? "" : v.amount_text || "",
        notes: recordContentLocked ? recordEditor?.notes ?? "" : v.notes || "",
        payload_json: JSON.stringify(recordContentLocked ? lockedPayload : { ...recordPayload, ...editablePayload }),
      });
      setRecordEditor(null); setRecords(await desktop.records(recordType, recordType === "staff" ? undefined : selectedClientId || undefined)); if (recordType === "staff") setStaffRecords(await desktop.records("staff"));
    } catch (e) { setError(String(e)); } finally { setBusy(false); }
  }
  async function saveNote(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    const values = Object.fromEntries(data.entries()) as Record<string, string>;
    try {
      setBusy(true); setError("");
      await desktop.saveNote({ id: noteEditor?.id || undefined, client_id: values.client_id || null, title: values.title, body: values.body || "" });
      setNoteEditor(undefined); setNotes(await desktop.notes());
    } catch (e) { setError(String(e)); } finally { setBusy(false); }
  }
  async function handleCsvFile(file?: File) {
    if (!file) return;
    try {
      setError("");
      if (!file.name.toLowerCase().endsWith(".csv") || file.size > 5 * 1024 * 1024) {
        throw new Error("Choose a CSV file no larger than 5 MB.");
      }
      const content = await file.text();
      const parsed = parseCsv(content);
      if (parsed.length < 2 || parsed.length > 1001) throw new Error("CSV needs a header and 1–1,000 data rows.");
      const headers = parsed[0].map(header => header.trim());
      if (headers.some(header => !header) || new Set(headers.map(h => h.toLowerCase())).size !== headers.length) {
        throw new Error("CSV headers must be non-empty and unique.");
      }
      setCsvFileName(file.name);
      setCsvContent(content);
      setCsvHeaders(headers);
      setCsvRows(parsed.slice(1));
      setCsvMapping({
        transaction_date: suggestedHeader(headers, ["date","transaction date","invoice date"]),
        description: suggestedHeader(headers, ["description","details","narration","particulars"]),
        amount_text: suggestedHeader(headers, ["amount","gross amount","net amount","total"]),
        currency: suggestedHeader(headers, ["currency","ccy"]),
        supplier: suggestedHeader(headers, ["supplier","vendor","payee"]),
        supplier_tin: suggestedHeader(headers, ["supplier tin","tin","tax id"]),
        vat_amount: suggestedHeader(headers, ["vat","vat amount","tax amount"]),
        invoice_reference: suggestedHeader(headers, ["invoice number","invoice no","reference","document number"]),
      });
    } catch (error) {
      setCsvFileName(""); setCsvContent(""); setCsvHeaders([]); setCsvRows([]);
      setError(String(error));
    }
  }

  async function commitCsvImport() {
    if (!selectedClientId || csvRows.length === 0) return;
    try {
      setBusy(true); setError("");
      const indexOf = (key: string) => csvHeaders.indexOf(csvMapping[key] ?? "");
      const mappedRows: HistoricalImportRow[] = csvRows.map((cells, index) => ({
        source_row: index + 2,
        transaction_date: cells[indexOf("transaction_date")] ?? "",
        description: cells[indexOf("description")] ?? "",
        amount_text: cells[indexOf("amount_text")] ?? "",
        currency: cells[indexOf("currency")] ?? "",
        supplier: cells[indexOf("supplier")] ?? "",
        supplier_tin: cells[indexOf("supplier_tin")] ?? "",
        vat_amount: cells[indexOf("vat_amount")] ?? "",
        invoice_reference: cells[indexOf("invoice_reference")] ?? "",
      }));
      const invalidRows = mappedRows.filter(row => !row.description.trim() || !row.amount_text.trim());
      if (invalidRows.length) {
        setError("Import stopped: " + invalidRows.length + " rows are missing a mapped description or source amount. No rows were written.");
        return;
      }
      await desktop.importHistoricalBatch({
        client_id: selectedClientId, source_filename: csvFileName, source_content: csvContent,
        mapping_json: JSON.stringify(csvMapping), rows: mappedRows,
      });
      setRecords(await desktop.records("historical_record", selectedClientId));
      setImportBatches(await desktop.importBatches(selectedClientId));
      setShowImportModal(false); setCsvFileName(""); setCsvContent(""); setCsvHeaders([]); setCsvRows([]);
    } catch (error) { setError(String(error)); }
    finally { setBusy(false); }
  }


  async function calculateImportCost() {
    if (!selectedClientId || importCostCaseReference.trim().length < 2) {
      setError("Select a client and enter the exact reference of a reviewed import discovery case.");
      return;
    }
    try {
      setBusy(true); setError(""); setImportCostPreview(null);
      const result = await desktop.calculateImportCostPreview(selectedClientId, importCostCaseReference.trim(), importCostBaseCurrency.trim().toUpperCase());
      const snapshot = {
        entry_kind: "calculation_result",
        source_reference: "Generated from saved source-backed goods and cost-component records",
        base_currency: result.base_currency,
        result_json: JSON.stringify(result),
      };
      await desktop.saveRecord({
        client_id: selectedClientId,
        record_type: "import_cost_input",
        title: `Draft landed-cost allocation · ${importCostCaseReference.trim()}`,
        status: "Draft",
        reference: importCostCaseReference.trim(),
        amount_text: String(result.total_landed_cost_minor),
        notes: "Draft management-cost allocation only. Not customs value, tax computation, inventory valuation approval or selling-price advice.",
        payload_json: JSON.stringify(snapshot),
      });
      setImportCostPreview(result);
      setRecords(await desktop.records("import_cost_input", selectedClientId));
    } catch (e) { setError(String(e)); }
    finally { setBusy(false); }
  }

  async function resetDemo() {
    if (!window.confirm("Reset the synthetic demonstration workspace? All demo changes will be removed. This does not affect files outside the ARM demo database.")) return;
    try { await desktop.reset(); await refresh("home"); setSection("home"); }
    catch (e) { setError(String(e)); }
  }

  if (!workspace && !error) return <div className="startup"><div className="brand-mark">A</div><h1>Opening your workspace</h1><p>Preparing the local demonstration database…</p></div>;

  return <div className="app-frame">
    <aside className="sidebar">
      <div className="brand"><div className="brand-mark">A</div><div><strong>ARM</strong><span>Audit Report Management</span></div></div>
      <div className="workspace-switch"><span className="overline">WORKSPACE</span><strong>{workspace?.firm_name ?? "Local workspace"}</strong><span className="local-state"><i /> Saved on this device</span></div>
      <nav aria-label="Main navigation">
        {nav.map((item) => <button key={item.id} className={section === item.id ? "nav-item active" : "nav-item"} onClick={() => { setSection(item.id); setError(""); }}>
          <span className="nav-dot" /><span><small>{item.eyebrow}</small>{item.label}</span>{section === item.id && <b>›</b>}
        </button>)}
      </nav>
      <div className="sidebar-bottom"><div className="demo-chip">DEMO ENVIRONMENT</div><p>Synthetic data only. Not connected to production.</p>{workspace?.persona === "administrator" && <button className="text-button" onClick={resetDemo}>Reset demo workspace</button>}</div>
    </aside>
    <main className="main">
      <header className="topbar"><div><span className="overline">ARM WORKSPACE / {nav.find(n => n.id === section)?.label.toUpperCase()}</span><h1>{section === "home" ? "Good work starts with a clear next step." : nav.find(n => n.id === section)?.label}</h1></div>
        <label className="persona-picker"><span>DEMO ROLE</span><select value={workspace?.persona ?? "administrator"} onChange={(e) => void switchPersona(e.target.value as Persona)} disabled={busy}>{Object.entries(personaLabels).map(([key,label]) => <option key={key} value={key}>{label}</option>)}</select></label>
      </header>
      <div className="notice"><span>i</span><div><strong>Prototype mode — synthetic data only</strong><small>{workspace?.sync_state ?? "Sync is not enabled in this prototype"} · No production data or cloud sync</small></div></div>
      {error && <div className="error-banner" role="alert">{error}<button onClick={() => setError("")}>Dismiss</button></div>}

      {section === "home" && <section className="home-grid">
        <article className="welcome-card"><span className="overline">YOUR WORKSPACE</span><h2>One place to keep client work organized.</h2><p>Start with a client profile, then capture source-backed records and move work through review. Every change stays on this device.</p><button className="primary" onClick={() => setSection("clients")}>Continue with clients <span>→</span></button><div className="welcome-foot"><span>{workspace?.clients_count ?? 0} active demo clients</span><span>Local database ready</span></div></article>
        <article className="summary-card"><span className="overline">INITIAL WORKFLOWS</span><div className="summary-row"><b>01</b><div><strong>Client records</strong><small>Register and maintain company or individual profiles</small></div><button onClick={() => setSection("clients")}>Open →</button></div><div className="summary-row"><b>02</b><div><strong>Products & invoices</strong><small>Maintain source-backed catalogue and invoice records</small></div><button onClick={() => setSection("catalogue")}>Open →</button></div><div className="summary-row"><b>03</b><div><strong>Historical & VAT work</strong><small>Capture historical information; calculations remain gated</small></div><button onClick={() => setSection("history")}>Open →</button></div><div className="summary-row"><b>04</b><div><strong>Staff & tasks</strong><small>Demonstrate assignment and review workflow</small></div><button onClick={() => setSection("work")}>Open →</button></div><div className="summary-row"><b>05</b><div><strong>Import costing</strong><small>Capture assumptions without unapproved calculations</small></div><button onClick={() => setSection("imports")}>Open →</button></div></article>
      </section>}

      {section === "home" && <section className="content-panel notes-panel"><div className="section-head"><div><span className="overline">INCREMENT 0 · LOCAL DRAFTS</span><h2>Quick notes</h2><p>Private demonstration notes saved on this device. These are not tasks, workpapers or audit findings.</p></div><button className="primary" disabled={!canMutate} onClick={() => setNoteEditor(null)}>+ New note</button></div><div className="record-grid">{notes.map(note => <article className="record-card" key={note.id}><div className="record-card-top"><span className="status">{note.status}</span><small>{new Date(note.updated_at).toLocaleDateString()}</small></div><h3>{note.title}</h3><p>{note.body || "No note details"}</p><button className="text-button" onClick={() => setNoteEditor(note)}>Open note →</button></article>)}{notes.length === 0 && <div className="empty-card"><strong>No local notes</strong><p>Create a short draft note and reopen ARM to confirm it persists.</p></div>}</div></section>}

      {section === "clients" && <section className="content-panel">
        <div className="section-head"><div><span className="overline">CLIENT WORKSPACE</span><h2>Client register</h2><p>Keep a clear profile for each company or individual.</p></div><button className="primary" disabled={!canEditClientProfiles} onClick={() => setClientEditor(null)}>+ Add client</button></div>
        <div className="toolbar"><input aria-label="Search clients" placeholder="Search name or TIN…" value={search} onChange={(e) => setSearch(e.target.value)} /><label className="check"><input type="checkbox" checked={includeArchived} onChange={(e) => setIncludeArchived(e.target.checked)} /> Include archived</label><span>{filteredClients.length} records</span></div>
        <div className="table-wrap"><table><thead><tr><th>CLIENT</th><th>TYPE</th><th>TIN</th><th>STATUS</th><th>UPDATED</th><th /></tr></thead><tbody>
          {filteredClients.map(c => <tr key={c.id}><td><strong>{c.display_name || c.legal_name}</strong><small>{c.legal_name}</small></td><td>{c.record_kind}</td><td>{c.tin_raw || "Not provided"}</td><td><span className={c.status === "Archived" ? "status muted" : "status"}>{c.status}</span></td><td>{new Date(c.updated_at).toLocaleDateString()}</td><td className="actions"><button onClick={() => setClientEditor(c)}>Open</button><button onClick={() => void openClientHistory(c)}>History</button>{c.status === "Draft" && <button disabled={!canManageClientLifecycle} onClick={() => void activateClient(c)}>Activate</button>}{c.status === "Archived" ? <button disabled={!canManageClientLifecycle} onClick={() => void restoreClient(c)}>Restore</button> : <button disabled={!canManageClientLifecycle} onClick={() => void archiveClient(c)}>Archive</button>}</td></tr>)}
          {filteredClients.length === 0 && <tr><td colSpan={6} className="empty">No clients match this search. Add a client to begin.</td></tr>}
        </tbody></table></div>
      </section>}

      {(["catalogue","history","work","imports"] as Section[]).includes(section) && (() => {
        const config = recordTypes[section as keyof typeof recordTypes];
        if (!config) return null;
        return <section className="content-panel">
          <div className="section-head"><div><span className="overline">{config.label.toUpperCase()}</span><h2>{nav.find(n => n.id === section)?.label}</h2><p>{config.hint}</p></div><div className="header-actions">{recordType === "historical_record" && <><button className="secondary" disabled={!canMutate || !selectedClientId} onClick={() => setShowImportModal(true)}>Import CSV</button><button className="secondary" disabled={!canMutate || !selectedClientId} onClick={() => setRecordEditor({id:"",client_id:selectedClientId,record_type:"historical_record",title:"Draft VAT workpaper",status:"Draft",reference:"",amount_text:"",notes:"",payload_json:JSON.stringify({historical_class:"Draft VAT workpaper grouping only"}),row_version:0,updated_at:""})}>Draft VAT workpaper</button><button className="secondary" disabled={!canMutate || !selectedClientId} onClick={() => setRecordEditor({id:"",client_id:selectedClientId,record_type:"historical_record",title:"VAT exception / unresolved item",status:"Draft",reference:"",amount_text:"",notes:"",payload_json:JSON.stringify({historical_class:"VAT exception / unresolved item",exception_type:"Missing source"}),row_version:0,updated_at:""})}>Log VAT exception</button></>}<button className="primary" disabled={!canMutate || (recordType !== "staff" && !selectedClientId) || (recordType === "task" && workspace?.persona === "auditor")} onClick={() => setRecordEditor({ id:"",client_id:recordType === "staff" ? null : selectedClientId,record_type:recordType,title:"",status:(statusOptions[recordType] ?? config.statuses)[0],reference:"",amount_text:"",notes:"",payload_json:"{}",row_version:0,updated_at:"" })}>+ Add record</button></div></div>
          <div className="subtype-tabs" role="group" aria-label="Record type">{subtypeOptions[section as keyof typeof subtypeOptions].filter(option => !(section === "work" && option.type === "staff" && !["administrator","manager"].includes(workspace?.persona ?? ""))).map(option => <button key={option.type} className={recordType === option.type ? "subtype-tab active" : "subtype-tab"} onClick={() => { setRecordType(option.type); if (section === "work") setWorkDashboard(false); }}>{option.type === "import_cost_input" ? "Cost inputs" : option.label}</button>)}</div>
          {recordType !== "staff" && <label className="client-context">CLIENT WORKSPACE<select value={selectedClientId} onChange={e => setSelectedClientId(e.target.value)}><option value="">Select a client…</option>{clients.map(c => <option key={c.id} value={c.id}>{c.display_name || c.legal_name}</option>)}</select></label>}
          {section === "work" && <div className="work-view-tabs"><button className={workDashboard ? "subtype-tab active" : "subtype-tab"} onClick={() => { setWorkDashboard(true); setRecordType("task"); }}>Overview</button><button className={!workDashboard && recordType === "task" ? "subtype-tab active" : "subtype-tab"} onClick={() => { setWorkDashboard(false); setRecordType("task"); }}>Task list</button></div>}
          {section === "work" && workDashboard && <div className="work-dashboard"><div className="work-dashboard-head"><div><span className="overline">INCREMENT 4 · WORK OVERVIEW</span><h3>{workspace?.persona === "auditor" ? "My work" : workspace?.persona === "reviewer" ? "Assigned review queue" : "Team work overview"}</h3><p>Counts are derived from task records for the selected client. No staff performance scores are calculated.</p></div><button className="secondary" onClick={() => setWorkloadView(value => !value)}>{workloadView ? "Back to task list" : "Workload by assignee"}</button></div>
            {workspace?.persona === "reviewer" && <p className="gate-note">Read-only reviewer view. Review decisions and task notes must be recorded by a manager or administrator explicitly assigned as reviewer.</p>}
            <div className="task-stats">
              <button onClick={() => {setTaskStatusFilter("All");setTaskDueFilter("All");}}><strong>{records.length}</strong><span>All tasks</span></button>
              <button onClick={() => {setTaskDueFilter("Due soon");setTaskStatusFilter("All");}}><strong>{records.filter(task => taskDueState(parseRecordPayload(task).due_date ?? "") === "Due soon").length}</strong><span>Due soon</span></button>
              <button onClick={() => {setTaskDueFilter("Overdue");setTaskStatusFilter("All");}}><strong>{records.filter(task => taskDueState(parseRecordPayload(task).due_date ?? "") === "Overdue").length}</strong><span>Overdue</span></button>
              <button onClick={() => {setTaskStatusFilter("Blocked");setTaskDueFilter("All");}}><strong>{records.filter(task => task.status === "Blocked").length}</strong><span>Blocked</span></button>
              <button onClick={() => {setTaskStatusFilter("Submitted");setTaskDueFilter("All");}}><strong>{records.filter(task => task.status === "Submitted").length}</strong><span>Waiting for review</span></button>
              <button onClick={() => {setTaskStatusFilter("Completed");setTaskDueFilter("All");}}><strong>{records.filter(task => task.status === "Completed").length}</strong><span>Completed</span></button>
            </div>
            <div className="task-filters"><label>Assignee<select value={taskAssigneeFilter} onChange={e => setTaskAssigneeFilter(e.target.value)}>{taskAssignees.map(name => <option key={name}>{name}</option>)}</select></label><label>Priority<select value={taskPriorityFilter} onChange={e => setTaskPriorityFilter(e.target.value)}>{["All","Urgent","High","Normal","Low"].map(value => <option key={value}>{value}</option>)}</select></label><label>Status<select value={taskStatusFilter} onChange={e => setTaskStatusFilter(e.target.value)}>{["All","Draft","Assigned","In progress","Blocked","Submitted","Returned","Completed","Cancelled"].map(value => <option key={value}>{value}</option>)}</select></label><label>Due date<select value={taskDueFilter} onChange={e => setTaskDueFilter(e.target.value)}>{["All","Overdue","Due soon","Scheduled","No due date"].map(value => <option key={value}>{value}</option>)}</select></label><button className="text-button" onClick={() => {setTaskAssigneeFilter("All");setTaskPriorityFilter("All");setTaskStatusFilter("All");setTaskDueFilter("All");}}>Clear filters</button></div>
            {workloadView ? <div className="table-wrap"><table><thead><tr><th>ASSIGNEE</th><th>ASSIGNED</th><th>IN PROGRESS</th><th>BLOCKED</th><th>WAITING REVIEW</th><th>COMPLETED</th></tr></thead><tbody>{taskAssignees.filter(name => name !== "All").map(name => { const assigned = records.filter(task => parseRecordPayload(task).assignee === name); return <tr key={name}><td><strong>{name}</strong></td><td>{assigned.filter(task => task.status === "Assigned").length}</td><td>{assigned.filter(task => task.status === "In progress").length}</td><td>{assigned.filter(task => task.status === "Blocked").length}</td><td>{assigned.filter(task => task.status === "Submitted").length}</td><td>{assigned.filter(task => task.status === "Completed").length}</td></tr>; })}{taskAssignees.length <= 1 && <tr><td colSpan={6} className="empty">No assigned tasks to summarize.</td></tr>}</tbody></table></div> : <div className="task-dashboard-list">{visibleTasks.map(task => { const meta = parseRecordPayload(task); const dueState = taskDueState(meta.due_date ?? ""); return <article className="task-dashboard-card" key={task.id}><div className="record-card-top"><span className="status">{task.status}</span><span className={dueState === "Overdue" ? "due-state overdue" : dueState === "Due soon" ? "due-state soon" : "due-state"}>{dueState}</span></div><h4>{task.title}</h4><p>{meta.description || task.notes || "No task description recorded."}</p><div className="task-card-meta"><span>Assignee: {meta.assignee || "Unassigned"}</span><span>Reviewer: {meta.reviewer || "Not assigned"}</span><span>Priority: {meta.priority || "Not set"}</span><span>Due: {meta.due_date || "No due date"}</span></div><button className="text-button" onClick={() => void openRecord(task)}>{workspace?.persona === "auditor" ? "Continue task →" : "Open task →"}</button></article>; })}{visibleTasks.length === 0 && <div className="empty-card"><strong>{workspace?.persona === "auditor" ? "No assigned work is waiting for you." : "No tasks match these filters."}</strong><p>Create or assign work from the task list when required.</p></div>}</div>}
          </div>}

          {recordType === "invoice" && (workspace?.persona === "manager" || workspace?.persona === "administrator") && <div className="queue-filter"><button className={invoiceQueueOnly ? "subtype-tab active" : "subtype-tab"} onClick={() => setInvoiceQueueOnly(value => !value)}>{invoiceQueueOnly ? "Show all invoices" : "Review queue"}</button><span>{records.filter(record => record.status === "Submitted for review").length} awaiting review</span></div>}
          {section === "imports" && <p className="gate-note">Prototype worksheet: enter source-backed goods lines and cost components, then prepare a draft allocation. The calculation keeps commercial purchase cost, customs-value references and management landed cost distinct. It does not determine customs value, statutory duties, recoverable tax or accounting capitalization policy.</p>}
          {section === "history" && <p className="gate-note">Historical amounts and VAT values are preserved as source text. Draft VAT workpapers only group references and questions; no tax rate, calculation, statutory return or filing output is generated until the approved tax matrix and independent expected results are available.</p>}
          {section === "imports" && <div className="import-cost-tools"><label>Reviewed import case reference<input value={importCostCaseReference} onChange={e => setImportCostCaseReference(e.target.value)} placeholder="Use the exact case reference" /></label><label>Base currency<input value={importCostBaseCurrency} onChange={e => setImportCostBaseCurrency(e.target.value.toUpperCase().slice(0,3))} maxLength={3} /></label><button className="primary" disabled={!canMutate || !["administrator","manager"].includes(workspace?.persona ?? "") || busy || !selectedClientId || !importCostCaseReference.trim()} onClick={() => void calculateImportCost()}>Prepare draft allocation</button><small>First mark the discovery case Reviewed and add goods lines and cost components with status Ready for review. All values use integer minor units; FX is an explicit numerator/denominator.</small></div>}
          {!(section === "work" && workDashboard) && <div className="record-grid">{visibleRecords.map(r => <article className="record-card" key={r.id}><div className="record-card-top"><span className="status">{r.status}</span><small>{new Date(r.updated_at).toLocaleDateString()}</small></div>{r.record_type === "historical_record" && historicalClass(r) && <span className="duplicate-flag">{historicalClass(r)}</span>}{r.record_type === "historical_record" && hasPossibleDuplicate(r) && <span className="duplicate-flag">Possible duplicate source reference · review required</span>}{r.record_type === "import_cost_input" && parseRecordPayload(r).entry_kind === "calculation_result" && <span className="duplicate-flag">Draft calculation snapshot · review required</span>}<h3>{r.title}</h3><p>{r.notes || "No additional notes"}</p><div className="record-meta"><span>{r.reference || "No source reference"}</span>{r.amount_text && <strong>{r.amount_text}</strong>}</div><div className="record-actions"><button className="text-button" onClick={() => void openRecord(r)}>Open record →</button>{r.record_type === "import_cost_input" && parseRecordPayload(r).entry_kind === "calculation_result" && <button className="text-button" onClick={() => { try { setImportCostPreview(JSON.parse(parseRecordPayload(r).result_json) as ImportCostResult); } catch { setError("The saved calculation snapshot could not be read."); } }}>View allocation →</button>}{historicalClass(r) === "Draft VAT workpaper grouping only" && <button className="text-button" onClick={() => setPreviewRecord(r)}>Preview / Print →</button>}</div></article>)}
          {visibleRecords.length === 0 && <div className="empty-card"><strong>No records yet</strong><p>Create a record to start this workflow. Data is stored locally in the demo database.</p></div>}</div>}

          {section === "imports" && importCostPreview && <section className="import-cost-preview"><div className="section-head"><div><span className="overline">DRAFT · NOT CUSTOMS VALUE</span><h3>Import-cost allocation preview</h3><p>Base currency: {importCostPreview.base_currency} · values shown in minor units, not formatted currency units.</p></div><button className="text-button" onClick={() => setImportCostPreview(null)}>Close preview</button></div><div className="task-stats"><div><strong>{importCostPreview.total_goods_value_minor.toLocaleString()}</strong><span>Goods value · minor units</span></div><div><strong>{importCostPreview.total_capitalized_cost_minor.toLocaleString()}</strong><span>Capitalized components</span></div><div><strong>{importCostPreview.total_landed_cost_minor.toLocaleString()}</strong><span>Draft landed cost</span></div><div><strong>{importCostPreview.excluded_or_pending_minor.toLocaleString()}</strong><span>Excluded / pending</span></div></div><div className="table-wrap"><table><thead><tr><th>ITEM</th><th>QUANTITY (MICROS)</th><th>GOODS VALUE</th><th>ALLOCATED COST</th><th>LANDED COST</th></tr></thead><tbody>{importCostPreview.lines.map(line => <tr key={line.item_reference}><td><strong>{line.item_reference}</strong><small>{line.description}</small><small>Source: {line.source_reference} · FX: {line.fx_rate_source} ({line.fx_rate_date}) · {line.fx_authorized_by}</small></td><td>{line.quantity_micros.toLocaleString()}</td><td>{line.goods_value_minor.toLocaleString()}</td><td>{line.allocated_cost_minor.toLocaleString()}</td><td><strong>{line.landed_cost_minor.toLocaleString()}</strong></td></tr>)}</tbody></table></div><div className="cost-component-list">{importCostPreview.components.map(component => <article key={component.component_reference}><strong>{component.category} · {component.component_reference}</strong><span>{component.source_amount_minor.toLocaleString()} {component.source_currency} → {component.base_amount_minor.toLocaleString()} {importCostPreview.base_currency} minor units</span><small>Source: {component.source_reference} · FX: {component.fx_rate_source} ({component.fx_rate_date}) · authorized by {component.fx_authorized_by}</small><small>{component.treatment} · {component.allocation_basis} · {component.allocations.map(a => `${a.item_reference}: ${a.amount_minor.toLocaleString()}`).join(" · ") || "No landed-cost allocation"}</small></article>)}</div><ul>{importCostPreview.limitations.map(item => <li key={item}>{item}</li>)}</ul><p className="print-warning">Draft management allocation only. It is not customs value, customs duty, tax computation, IFRS inventory valuation, approved cost capitalization, or selling-price advice.</p></section>}
          {recordType === "historical_record" && importBatches.length > 0 && <section className="import-batches"><div className="preview-heading"><strong>Imported source batches</strong><span>Original CSV retained in local database</span></div>{importBatches.map(batch => <article className="batch-row" key={batch.id}><div><strong>{batch.source_filename}</strong><small>{batch.imported_row_count} rows · {new Date(batch.imported_at).toLocaleString()}</small><small>{batch.source_row_count === batch.imported_row_count ? "Row-count reconciled" : "Row-count mismatch · investigate" } · {records.filter(record => importBatchId(record) === batch.id && hasPossibleDuplicate(record)).length} possible duplicate references</small></div><div><small>SHA-256</small><code>{batch.source_checksum_sha256}</code><small>Monetary totals are not calculated in this prototype.</small></div></article>)}</section>}
        </section>;
      })()}

      {section === "activity" && <section className="content-panel"><div className="section-head"><div><span className="overline">LOCAL AUDIT HISTORY</span><h2>Recent activity</h2><p>Append-only local events for the synthetic workspace.</p></div><button className="secondary" onClick={() => void refresh("activity")}>Refresh</button></div>
        <div className="event-list">{events.map(e => <div className="event-row" key={e.id}><span className="event-mark" /><div><strong>{e.event_type.replaceAll("_"," ")}</strong><small>{e.actor_name} · {e.target_type} · {e.target_id}</small><details className="event-details"><summary>Event data</summary><pre>{formatSnapshot(e.payload_json)}</pre></details></div><time>{new Date(e.occurred_at_utc).toLocaleString()}</time></div>)}{events.length === 0 && <p className="empty">No activity recorded yet.</p>}</div>
      </section>}

      <footer className="main-footer"><span>ARM · Local demonstration workspace</span><span>Changes are saved on this device only</span></footer>
    </main>

    {previewRecord && <div className="modal-backdrop print-backdrop" role="presentation" onMouseDown={(e) => { if (e.target === e.currentTarget) setPreviewRecord(null); }}><section className="modal print-modal"><article className="print-sheet"><span className="overline">ARM · DRAFT WORKPAPER</span><h1>VAT workpaper — grouping preview</h1><div className="print-warning">DRAFT ONLY · NOT A TAX COMPUTATION · NOT A STATUTORY RETURN · NOT FOR FILING</div><p><strong>Client:</strong> {clients.find(c => c.id === previewRecord.client_id)?.display_name || clients.find(c => c.id === previewRecord.client_id)?.legal_name || "Client workspace"}</p><p><strong>Workpaper:</strong> {previewRecord.title}</p><p><strong>Period:</strong> {parseRecordPayload(previewRecord).workpaper_period_label || "Not supplied"}</p><p><strong>Status:</strong> {previewRecord.status}</p><h3>Included source references</h3><pre>{parseRecordPayload(previewRecord).included_source_references || "No source references recorded."}</pre><h3>Scope and basis note</h3><pre>{parseRecordPayload(previewRecord).scope_and_basis_note || previewRecord.notes || "No scope note recorded."}</pre><h3>Practitioner questions</h3><pre>{parseRecordPayload(previewRecord).review_questions || "No questions recorded."}</pre><h3>Record metadata</h3><pre>{formatSnapshot(previewRecord.payload_json)}</pre><p className="print-warning">No VAT totals, rates, adjustments, tax conclusions or filing data are generated by this preview. A qualified practitioner must determine and approve applicable treatment separately.</p></article><div className="modal-actions print-actions"><button className="secondary" onClick={() => setPreviewRecord(null)}>Close</button><button className="primary" onClick={() => window.print()}>Print / Save as PDF</button></div></section></div>}

    {showImportModal && <div className="modal-backdrop" role="presentation" onMouseDown={(e) => { if (e.target === e.currentTarget) setShowImportModal(false); }}><section className="modal import-modal"><div className="modal-head"><div><span className="overline">INCREMENT 3 · HISTORICAL IMPORT</span><h2>CSV mapping preview</h2><p>Map source columns before writing any rows to this client workspace.</p></div><button type="button" className="close" onClick={() => setShowImportModal(false)}>×</button></div><div className="import-warning">Synthetic data only. The CSV content is stored in the local SQLite database, which is not yet encrypted. No tax or VAT calculations are performed.</div><label className="wide">Choose CSV source<input type="file" accept=".csv,text/csv" onChange={e => void handleCsvFile(e.target.files?.[0])} /></label>{csvFileName && <div className="import-file-summary"><strong>{csvFileName}</strong><span>{csvRows.length} data rows · {csvContent.length.toLocaleString()} characters</span></div>}{csvHeaders.length > 0 && <><div className="mapping-grid">{csvMappingFields.map(field => <label key={field.key}>{field.label}{field.required && " *"}<select value={csvMapping[field.key] ?? ""} onChange={e => setCsvMapping(current => ({ ...current, [field.key]: e.target.value }))}><option value="">Not mapped</option>{csvHeaders.map(header => <option key={header} value={header}>{header}</option>)}</select></label>)}</div><div className="preview-heading"><strong>Preview</strong><span>First 5 rows · source values are preserved as text</span></div><div className="csv-preview"><table><thead><tr>{csvHeaders.map(header => <th key={header}>{header}</th>)}</tr></thead><tbody>{csvRows.slice(0,5).map((row,index)=><tr key={index}>{csvHeaders.map((header,col)=><td key={header}>{row[col] ?? ""}</td>)}</tr>)}</tbody></table></div><p className="import-footnote">Required mappings: Description and Source amount. All rows are validated before a single SQLite transaction commits. A failed row prevents the entire batch from being written.</p></>}<div className="modal-actions"><button type="button" className="secondary" onClick={() => setShowImportModal(false)}>Cancel</button><button className="primary" disabled={busy || !csvFileName || !csvMapping.description || !csvMapping.amount_text || csvRows.length === 0} onClick={() => void commitCsvImport()}>Import {csvRows.length} rows</button></div></section></div>}

    {historyClient && <div className="modal-backdrop" role="presentation" onMouseDown={(e) => { if (e.target === e.currentTarget) setHistoryClient(null); }}><section className="modal history-modal"><div className="modal-head"><div><span className="overline">CLIENT CHANGE HISTORY</span><h2>{historyClient.display_name || historyClient.legal_name}</h2><p>Before-and-after profile snapshots, actor and time. This history is local to the synthetic workspace.</p></div><button type="button" className="close" onClick={() => setHistoryClient(null)}>×</button></div><div className="change-list">{clientChanges.map(change => <article className="change-item" key={change.id}><div className="change-heading"><strong>{change.action === "created" ? "Profile created" : "Profile updated"}</strong><span>{change.actor_name} · {new Date(change.changed_at).toLocaleString()}</span></div><div className="change-columns"><div><small>BEFORE</small><pre>{formatSnapshot(change.before_json)}</pre></div><div><small>AFTER</small><pre>{formatSnapshot(change.after_json)}</pre></div></div></article>)}{clientChanges.length === 0 && <p className="empty">No profile changes recorded yet.</p>}</div></section></div>}

    {noteEditor !== undefined && <div className="modal-backdrop" role="presentation" onMouseDown={(e) => { if (e.target === e.currentTarget) setNoteEditor(undefined); }}><form className="modal note-modal" onSubmit={saveNote}><div className="modal-head"><div><span className="overline">INCREMENT 0 · LOCAL DRAFT</span><h2>{noteEditor ? "Edit local note" : "New local note"}</h2><p>Not a task, workpaper, finding or professional approval.</p></div><button type="button" className="close" onClick={() => setNoteEditor(undefined)}>×</button></div><div className="form-grid"><label className="wide">Title *<input name="title" required minLength={1} maxLength={120} defaultValue={noteEditor?.title ?? ""} /></label><label>Related client (optional)<select name="client_id" defaultValue={noteEditor?.client_id ?? ""}><option value="">No client</option>{clients.map(c => <option key={c.id} value={c.id}>{c.display_name || c.legal_name}</option>)}</select></label><label className="wide">Note<textarea name="body" maxLength={2000} defaultValue={noteEditor?.body ?? ""} /></label></div><div className="modal-actions"><button type="button" className="secondary" onClick={() => setNoteEditor(undefined)}>Cancel</button><button className="primary" disabled={!canMutate || busy}>Save local draft</button></div></form></div>}

    {clientEditor !== undefined && <div className="modal-backdrop" role="presentation" onMouseDown={(e) => { if (e.target === e.currentTarget) setClientEditor(undefined); }}><form className="modal" onSubmit={saveClient}>
      <div className="modal-head"><div><span className="overline">CLIENT PROFILE</span><h2>{clientEditor ? (canEditClientProfiles ? "Edit client" : "View client") : "Add a client"}</h2><p>Only a record kind and legal name are required to save a draft.</p></div><button type="button" className="close" onClick={() => setClientEditor(undefined)}>×</button></div>
      <fieldset className="form-fieldset" disabled={!canEditClientProfiles}><div className="form-grid"><label>Record type<select name="record_kind" defaultValue={clientEditor?.record_kind ?? "Company"}><option>Company</option><option>Individual</option><option>Other organization</option></select></label><label>Legal or full name *<input name="legal_name" required minLength={2} maxLength={200} defaultValue={clientEditor?.legal_name ?? ""} /></label><label>Display name<input name="display_name" maxLength={200} defaultValue={clientEditor?.display_name ?? ""} /></label><label>TIN (not verified)<input name="tin_raw" maxLength={40} defaultValue={clientEditor?.tin_raw ?? ""} /></label><label>Sector / activity<input name="sector" defaultValue={clientEditor?.sector ?? ""} /></label><label>Primary contact<input name="contact_name" defaultValue={clientEditor?.contact_name ?? ""} /></label><label>Contact email<input type="email" name="contact_email" defaultValue={clientEditor?.contact_email ?? ""} /></label><label>Contact phone<input name="contact_phone" defaultValue={clientEditor?.contact_phone ?? ""} /></label><label className="wide">Address<input name="address_text" defaultValue={clientEditor?.address_text ?? ""} /></label><label>Internal reference<input name="internal_reference" defaultValue={clientEditor?.internal_reference ?? ""} /></label><label className="wide">Notes<textarea name="notes" maxLength={2000} defaultValue={clientEditor?.notes ?? ""} /></label></div></fieldset>
      <div className="modal-actions"><button type="button" className="secondary" onClick={() => setClientEditor(undefined)}>Close</button><button className="primary" disabled={!canEditClientProfiles || busy}>Save draft</button></div>
    </form></div>}

    {recordEditor && <div className="modal-backdrop" role="presentation" onMouseDown={(e) => { if (e.target === e.currentTarget) setRecordEditor(null); }}><form className="modal" onSubmit={saveRecord}>
      <div className="modal-head"><div><span className="overline">WORKFLOW RECORD</span><h2>{recordEditor.id ? "Edit record" : "Add record"}</h2><p>Store source information and review state; do not infer professional conclusions.</p></div><button type="button" className="close" onClick={() => setRecordEditor(null)}>×</button></div>
      <fieldset className="form-fieldset" disabled={!canMutate}><div className="form-grid"><label className="wide">Title / primary name *<input name="title" required maxLength={200} defaultValue={recordEditor.title} disabled={recordContentLocked} /></label><label>Client<select name="client_id" defaultValue={recordEditor.client_id ?? ""} disabled={Boolean(recordEditor.id)}><option value="">No client selected</option>{clients.map(c => <option value={c.id} key={c.id}>{c.display_name || c.legal_name}</option>)}</select></label><label>Status<select name="status" defaultValue={recordEditor.status} disabled={taskStatusLocked || invoiceStatusLocked || calculationResultLocked}>{(recordType === "task" && workspace?.persona === "auditor" ? ["In progress","Blocked","Submitted"] : recordType === "invoice" && workspace?.persona === "auditor" ? (recordEditor.status === "Needs correction" ? ["Needs correction","Draft","Ready for review"] : ["Draft","Ready for review"]) : statusOptions[recordType] ?? recordTypes[section as keyof typeof recordTypes]?.statuses ?? []).map(s => <option key={s}>{s}</option>)}</select></label><label>{recordType === "import_case" || recordType === "import_cost_input" ? "Import case reference" : "Source / reference"}<input name="reference" defaultValue={recordEditor.reference} disabled={recordContentLocked} /></label><label>{recordType === "invoice" || recordType === "historical_record" ? "Amount before VAT / source amount (text)" : "Amount / value (text only)"}<input name="amount_text" defaultValue={recordEditor.amount_text} placeholder="Preserve source notation; no calculation" disabled={recordContentLocked} /></label>{(recordFields[recordType] ?? []).map(field => <label key={field.key} className={field.wide ? "wide" : ""}>{field.label}{recordType === "task" && ["assignee","reviewer"].includes(field.key) ? <select name={field.key} defaultValue={recordPayload[field.key] ?? ""} disabled={(taskContentLocked && !(field.key === "reviewer" && taskReviewerReassignEditable)) || workspace?.persona === "auditor"}><option value="">Select active staff…</option>{staffRecords.filter(staff => staff.status === "Active").map(staff => <option key={staff.id} value={staff.title}>{staff.title}</option>)}</select> : field.kind === "select" ? <select name={field.key} defaultValue={recordPayload[field.key] ?? field.options?.[0] ?? ""} disabled={Boolean(field.readOnly) || recordContentLocked || (recordType === "invoice" && field.key === "reviewer_note" && !invoiceReviewNoteEditable) || (recordType === "task" && workspace?.persona === "auditor" && ["priority","due_date","reassignment_reason","cancellation_reason"].includes(field.key))}>{(field.options ?? []).map(option => <option key={option}>{option}</option>)}</select> : field.kind === "textarea" ? <textarea name={field.key} defaultValue={recordPayload[field.key] ?? ""} placeholder={field.placeholder} disabled={Boolean(field.readOnly) || (recordContentLocked && !(field.key === "cancellation_reason" && taskCancellationReasonEditable) && !(field.key === "reassignment_reason" && taskReviewerReassignEditable) && !(field.key === "reviewer_note" && invoiceReviewNoteEditable)) || (recordType === "invoice" && field.key === "reviewer_note" && !invoiceReviewNoteEditable) || (recordType === "task" && workspace?.persona === "auditor" && ["priority","due_date","reassignment_reason","cancellation_reason"].includes(field.key))} /> : <input name={field.key} type={field.kind === "date" ? "date" : "text"} defaultValue={recordPayload[field.key] ?? ""} placeholder={field.placeholder} disabled={recordContentLocked || (recordType === "task" && workspace?.persona === "auditor" && ["priority","due_date","reassignment_reason","cancellation_reason"].includes(field.key))} />}</label>)}{recordType === "import_cost_input" && <p className="wide evidence-prototype-note">For Goods lines, enter quantity in millionths and unit price in source-currency minor units. For cost components, choose a treatment and explicit allocation basis. FX numerator/denominator converts source minor units to base-currency minor units. Keep customs-value-only and recoverable-tax amounts out of capitalized landed cost. Source references are required before review.</p>}{recordType === "invoice" && <p className="wide evidence-prototype-note">Source evidence is metadata/reference-only in this prototype. No files are uploaded, stored or previewed. Use synthetic documents and references only.</p>}<label className="wide">Notes and assumptions<textarea name="notes" defaultValue={recordEditor.notes} disabled={recordContentLocked} /></label>{taskContentLocked && <p className="wide gate-note">{recordEditor.status === "Submitted" ? "Submitted task content is locked. The preparer may respond to review notes but cannot edit the submission until the assigned reviewer returns it." : "This task is terminal and read-only."}</p>}</div></fieldset>
      {recordEditor.id && <details className="version-history"><summary>Version history · {recordVersions.length} snapshots</summary>{recordVersions.map(version => <article className="version-item" key={version.id}><div><strong>Version {version.row_version}</strong><small>{version.actor_name} · {new Date(version.changed_at).toLocaleString()}</small></div><pre>{formatSnapshot(version.snapshot_json)}</pre></article>)}</details>}
      {recordEditor.record_type === "task" && recordEditor.id && <section className="review-note-panel"><div className="section-head"><div><span className="overline">SEPARATE TASK REVIEW</span><h3>Review notes</h3><p>Task completion is not professional approval. Notes are retained and cannot be deleted.</p></div><span className="status">{taskReviewNotes.length} notes</span></div>{taskReviewNotes.map(note => <article className="review-note-card" key={note.id}><div className="review-note-top"><strong>{note.category}</strong><span className="status">{note.status}</span></div><p>{note.description}</p><small>Reviewer: {note.reviewer_name} · {new Date(note.created_at).toLocaleString()}</small>{note.affected_record_reference && <p><strong>Affected reference:</strong> {note.affected_record_reference}</p>}<p><strong>Required response:</strong> {note.required_response}</p>{note.preparer_response && <div className="review-response"><small>PREPARER RESPONSE · {note.response_actor_name}</small><p>{note.preparer_response}</p></div>}{note.disposition && <div className="review-response"><small>{note.status.toUpperCase()} · {note.disposition_actor_name}</small><p>{note.disposition}</p></div>}{workspace?.persona === "auditor" && recordPayload.assignee === workspace.actor_name && ["Open","Reopened"].includes(note.status) && <div className="review-response-form"><textarea value={reviewResponses[note.id] ?? ""} onChange={e => setReviewResponses(current => ({ ...current, [note.id]: e.target.value }))} placeholder="Describe the correction or response…" /><button type="button" className="secondary" disabled={busy || (reviewResponses[note.id] ?? "").trim().length < 3} onClick={() => void respondReviewNote(note)}>Submit response</button></div>}{(workspace?.persona === "manager" || workspace?.persona === "administrator") && recordPayload.reviewer === workspace.actor_name && note.status === "Responded" && <div className="review-actions"><button type="button" className="secondary" disabled={busy} onClick={() => void disposeReviewNote(note,"Reopened")}>Reopen</button><button type="button" className="primary" disabled={busy} onClick={() => void disposeReviewNote(note,"Cleared")}>Clear note</button></div>}</article>)}{taskReviewNotes.length === 0 && <p className="empty">No review notes. A reviewer may add a note after the preparer submits this task.</p>}{(workspace?.persona === "manager" || workspace?.persona === "administrator") && recordEditor.status === "Submitted" && recordPayload.reviewer === workspace.actor_name && <div className="review-note-compose"><h4>Add review note</h4><label>Category<select value={reviewNoteCategory} onChange={e => setReviewNoteCategory(e.target.value)}>{["Missing evidence","Incorrect data","Unclear explanation","Calculation query","Other"].map(option => <option key={option}>{option}</option>)}</select></label><label>Description<textarea value={reviewNoteDescription} onChange={e => setReviewNoteDescription(e.target.value)} /></label><label>Affected record / evidence reference<input value={reviewNoteReference} onChange={e => setReviewNoteReference(e.target.value)} /></label><label>Required response<textarea value={reviewNoteRequired} onChange={e => setReviewNoteRequired(e.target.value)} /></label><button type="button" className="secondary" disabled={busy || reviewNoteDescription.trim().length < 5 || reviewNoteRequired.trim().length < 5} onClick={() => void addReviewNote()}>Open review note</button></div>}</section>}
      <div className="modal-actions"><button type="button" className="secondary" onClick={() => setRecordEditor(null)}>Cancel</button><button className="primary" disabled={!canMutate || busy || taskStatusLocked || invoiceStatusLocked || calculationResultLocked}>Save local record</button></div>
    </form></div>}
  </div>;
}
