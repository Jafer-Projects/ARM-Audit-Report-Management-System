import { useEffect, useMemo, useState, type FormEvent } from "react";
import { desktop } from "./services/desktop";
import type { Client, ClientChange, EventItem, LocalNote, Persona, WorkspaceInfo, WorkspaceRecord } from "./types";

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
  catalogue: { type: "product", label: "Product / invoice record", statuses: ["Draft", "Submitted for review", "Reviewed", "Archived"], hint: "Create categories, products and invoice records. Amounts are stored as entered text; no tax or accounting calculation is performed." },
  history: { type: "historical_record", label: "Historical record / workpaper", statuses: ["Draft", "Needs source", "Ready for review", "Reviewed"], hint: "Capture historical records and source references. VAT calculations remain disabled until an approved tax rule pack and independent test fixtures exist." },
  work: { type: "task", label: "Staff task", statuses: ["Draft", "Assigned", "In progress", "Blocked", "Submitted", "Returned", "Completed"], hint: "Record staff and task workflow demonstrations. Demo personas are not production authorization." },
  imports: { type: "import_case", label: "Import-cost case", statuses: ["Discovery", "Inputs captured", "Needs review", "Reviewed"], hint: "Capture import-cost assumptions and source references only. No customs valuation, landed-cost or selling-price calculation is enabled." },
};
const subtypeOptions: Record<"catalogue" | "history" | "work" | "imports", { type: string; label: string }[]> = {
  catalogue: [{ type: "category", label: "Categories" }, { type: "product", label: "Products" }, { type: "invoice", label: "Invoices" }],
  history: [{ type: "period", label: "Periods" }, { type: "historical_record", label: "Historical records" }],
  work: [{ type: "staff", label: "Staff" }, { type: "task", label: "Tasks" }],
  imports: [{ type: "import_case", label: "Import cases" }, { type: "import_cost_input", label: "Cost inputs" }],
};
const statusOptions: Record<string, string[]> = {
  category: ["Draft","Active","Archived"], product: ["Draft","Active","Archived"],
  invoice: ["Draft","Submitted for review","Returned","Archived"],
  period: ["Draft","Needs source","Ready for review"],
  historical_record: ["Draft","Needs source","Ready for review"],
  staff: ["Draft","Active","Inactive"],
  task: ["Draft","Assigned","In progress","Blocked","Submitted","Returned","Completed","Cancelled"],
  import_case: ["Discovery","Needs practitioner input","Ready for workshop","Deferred"],
  import_cost_input: ["Discovery","Needs practitioner input","Ready for workshop","Deferred"],
};
type RecordField = { key: string; label: string; kind?: "text" | "date" | "select" | "textarea"; options?: string[]; placeholder?: string; wide?: boolean };
const recordFields: Record<string, RecordField[]> = {
  category: [
    { key: "description", label: "Description", kind: "textarea", wide: true },
    { key: "parent_category", label: "Parent category (optional)" },
  ],
  product: [
    { key: "category", label: "Category" },
    { key: "description", label: "Description", kind: "textarea", wide: true },
    { key: "item_code", label: "Internal code / SKU (optional)" },
    { key: "unit", label: "Unit / quantity description (optional)" },
  ],
  invoice: [
    { key: "transaction_class", label: "Transaction class", kind: "select", options: ["Sales","Purchase"] },
    { key: "invoice_number", label: "Invoice number (source text)" },
    { key: "invoice_date", label: "Invoice date", kind: "date" },
    { key: "party_name", label: "Customer / supplier name" },
    { key: "party_tin", label: "Customer / supplier TIN (not verified)" },
    { key: "source_type", label: "Source type", kind: "select", options: ["Paper","PDF","Spreadsheet","Photo / scan","Other"] },
    { key: "source_filename", label: "Source filename / document label" },
    { key: "source_file_size_bytes", label: "Source file size in bytes (metadata only)" },
    { key: "source_date_received", label: "Date source received", kind: "date" },
    { key: "source_document_reference", label: "Document storage reference (not a local path)" },
    { key: "source_association_note", label: "How this source relates to the invoice", kind: "textarea", wide: true },
    { key: "financial_period", label: "Period label (as supplied)" },
    { key: "vat_amount", label: "VAT amount as shown on source (text)" },
    { key: "currency", label: "Currency as shown on source" },
    { key: "line_items", label: "Invoice lines — one per line: description | quantity | amount before VAT | VAT as stated", kind: "textarea", wide: true },
  ],
  period: [
    { key: "period_label", label: "Period label" },
    { key: "start_date", label: "Start date", kind: "date" },
    { key: "end_date", label: "End date", kind: "date" },
    { key: "period_type", label: "Period type", kind: "select", options: ["Not classified","Month","Quarter","Year","Custom"] },
    { key: "authority_source", label: "Period definition source" },
  ],
  historical_record: [
    { key: "historical_class", label: "Record type", kind: "select", options: ["Purchase","Expense","Tax / VAT carry-forward (source supplied)","Other"] },
    { key: "transaction_date", label: "Transaction date", kind: "date" },
    { key: "source_period", label: "Source period label" },
    { key: "tax_type_label", label: "Tax type / label exactly as supplied" },
    { key: "tax_period_label", label: "Tax period exactly as stated" },
    { key: "balance_kind", label: "Balance kind", kind: "select", options: ["Not classified","Opening","Closing","Carry-forward"] },
    { key: "supplier", label: "Supplier / payee" },
    { key: "supplier_tin", label: "TIN (opaque text)" },
    { key: "description", label: "Description / category" },
    { key: "currency", label: "Currency as stated" },
    { key: "balance_amount", label: "Balance / amount as stated (text)" },
    { key: "vat_amount", label: "VAT amount as stated (text)" },
    { key: "invoice_reference", label: "Invoice / reference number" },
    { key: "entry_method", label: "Entry method", kind: "select", options: ["Manual","Imported"] },
    { key: "source_filename", label: "Source filename / reference" },
  ],
  staff: [
    { key: "staff_reference", label: "Demo staff reference" },
    { key: "job_title", label: "Job title" },
    { key: "discipline", label: "Discipline / work category" },
  ],
  task: [
    { key: "description", label: "Description / expected output", kind: "textarea", wide: true },
    { key: "assignee", label: "Assignee (active staff name)" },
    { key: "due_date", label: "Due date", kind: "date" },
    { key: "priority", label: "Priority", kind: "select", options: ["Not set","Low","Normal","High","Urgent"] },
    { key: "task_type", label: "Task type", kind: "select", options: ["Data capture","Review","Follow-up","Other"] },
    { key: "related_record", label: "Related record reference" },
    { key: "completion_summary", label: "Work performed / completion summary", kind: "textarea", wide: true },
  ],
  import_case: [
    { key: "discovery_stage", label: "Discovery stage", kind: "select", options: ["Workflow discovery","Source inventory","Cost component discovery","FX policy discovery","Allocation method discovery","Review and approval discovery"] },
    { key: "shipment_identifier", label: "Shipment / import event identifier (as described)" },
    { key: "supplier", label: "Supplier / source party" },
    { key: "relevant_dates", label: "Relevant dates (source wording)" },
    { key: "currency", label: "Currencies involved (as described)" },
    { key: "intended_purpose", label: "Intended purpose of calculation", kind: "textarea", wide: true },
    { key: "unresolved_questions", label: "Unresolved questions", kind: "textarea", wide: true },
  ],
  import_cost_input: [
    { key: "component_name", label: "Component name (not approved)" },
    { key: "amount_and_currency", label: "Amount and currency as shown" },
    { key: "source_reference", label: "Source document / reference" },
    { key: "source_date", label: "Source date", kind: "date" },
    { key: "fx_rate_and_source", label: "FX rate and source (if supplied)" },
    { key: "allocation_basis", label: "Allocation basis (unapproved)", kind: "textarea", wide: true },
    { key: "inclusion_reason", label: "Inclusion / exclusion reason", kind: "textarea", wide: true },
  ],
};
const personaLabels: Record<Persona, string> = {
  administrator: "Demo Firm Administrator", manager: "Demo Engagement Manager",
  auditor: "Demo Auditor", reviewer: "Demo Read-only Reviewer",
};

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
  const [events, setEvents] = useState<EventItem[]>([]);
  const [notes, setNotes] = useState<LocalNote[]>([]);
  const [historyClient, setHistoryClient] = useState<Client | null>(null);
  const [clientChanges, setClientChanges] = useState<ClientChange[]>([]);
  const [noteEditor, setNoteEditor] = useState<LocalNote | null | undefined>(undefined);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const [search, setSearch] = useState("");
  const [includeArchived, setIncludeArchived] = useState(false);
  const [clientEditor, setClientEditor] = useState<Client | null | undefined>(undefined);
  const [recordEditor, setRecordEditor] = useState<WorkspaceRecord | null>(null);
  const [recordType, setRecordType] = useState("product");

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
  useEffect(() => {
    if (["catalogue", "history", "work", "imports"].includes(section)) {
      void desktop.clients(false).then(items => { setClients(items); setSelectedClientId(current => items.some(item => item.id === current) ? current : items[0]?.id || ""); }).catch((e) => setError(String(e)));
      if (recordType === "staff" || selectedClientId) {
        void desktop.records(recordType, recordType === "staff" ? undefined : selectedClientId).then(setRecords).catch((e) => setError(String(e)));
      } else { setRecords([]); }
    }
  }, [recordType, section, selectedClientId]);

  const recordPayload = useMemo<Record<string, string>>(() => {
    if (!recordEditor?.payload_json) return {};
    try { return JSON.parse(recordEditor.payload_json) as Record<string, string>; }
    catch { return {}; }
  }, [recordEditor]);
  const filteredClients = useMemo(() => clients.filter((c) =>
    [c.legal_name, c.display_name, c.tin_raw].join(" ").toLowerCase().includes(search.toLowerCase())), [clients, search]);
  const canMutate = workspace?.persona !== "reviewer";

  async function switchPersona(persona: Persona) {
    try { setBusy(true); setWorkspace(await desktop.setPersona(persona)); await refresh(); }
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
      await desktop.saveRecord({
        duplicate_acknowledged: duplicateAcknowledged, duplicate_acknowledgement_reason: duplicateReason,
        id: recordEditor?.id || undefined, row_version: recordEditor?.id ? recordEditor.row_version : undefined,
        client_id: v.client_id || null, record_type: recordType, title: v.title,
        status: v.status || (statusOptions[recordType] ?? config.statuses)[0], reference: v.reference || "",
        amount_text: v.amount_text || "", notes: v.notes || "", payload_json: JSON.stringify(Object.fromEntries((recordFields[recordType] ?? []).map(field => [field.key, v[field.key] ?? ""]))),
      });
      setRecordEditor(null); setRecords(await desktop.records(recordType, recordType === "staff" ? undefined : selectedClientId || undefined));
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
      <div className="sidebar-bottom"><div className="demo-chip">DEMO ENVIRONMENT</div><p>Synthetic data only. Not connected to production.</p><button className="text-button" onClick={resetDemo}>Reset demo workspace</button></div>
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
        <div className="section-head"><div><span className="overline">CLIENT WORKSPACE</span><h2>Client register</h2><p>Keep a clear profile for each company or individual.</p></div><button className="primary" disabled={!canMutate} onClick={() => setClientEditor(null)}>+ Add client</button></div>
        <div className="toolbar"><input aria-label="Search clients" placeholder="Search name or TIN…" value={search} onChange={(e) => setSearch(e.target.value)} /><label className="check"><input type="checkbox" checked={includeArchived} onChange={(e) => setIncludeArchived(e.target.checked)} /> Include archived</label><span>{filteredClients.length} records</span></div>
        <div className="table-wrap"><table><thead><tr><th>CLIENT</th><th>TYPE</th><th>TIN</th><th>STATUS</th><th>UPDATED</th><th /></tr></thead><tbody>
          {filteredClients.map(c => <tr key={c.id}><td><strong>{c.display_name || c.legal_name}</strong><small>{c.legal_name}</small></td><td>{c.record_kind}</td><td>{c.tin_raw || "Not provided"}</td><td><span className={c.status === "Archived" ? "status muted" : "status"}>{c.status}</span></td><td>{new Date(c.updated_at).toLocaleDateString()}</td><td className="actions"><button onClick={() => setClientEditor(c)}>Open</button><button onClick={() => void openClientHistory(c)}>History</button>{c.status === "Draft" && <button disabled={!canMutate} onClick={() => void activateClient(c)}>Activate</button>}{c.status === "Archived" ? <button disabled={!canMutate} onClick={() => void restoreClient(c)}>Restore</button> : <button disabled={!canMutate} onClick={() => void archiveClient(c)}>Archive</button>}</td></tr>)}
          {filteredClients.length === 0 && <tr><td colSpan={6} className="empty">No clients match this search. Add a client to begin.</td></tr>}
        </tbody></table></div>
      </section>}

      {(["catalogue","history","work","imports"] as Section[]).includes(section) && (() => {
        const config = recordTypes[section as keyof typeof recordTypes];
        if (!config) return null;
        return <section className="content-panel">
          <div className="section-head"><div><span className="overline">{config.label.toUpperCase()}</span><h2>{nav.find(n => n.id === section)?.label}</h2><p>{config.hint}</p></div><button className="primary" disabled={!canMutate || (recordType !== "staff" && !selectedClientId)} onClick={() => setRecordEditor({ id:"",client_id:recordType === "staff" ? null : selectedClientId,record_type:recordType,title:"",status:(statusOptions[recordType] ?? config.statuses)[0],reference:"",amount_text:"",notes:"",payload_json:"{}",row_version:0,updated_at:"" })}>+ Add record</button></div>
          <div className="subtype-tabs" role="group" aria-label="Record type">{subtypeOptions[section as keyof typeof subtypeOptions].map(option => <button key={option.type} className={recordType === option.type ? "subtype-tab active" : "subtype-tab"} disabled={option.type === "import_cost_input"} onClick={() => setRecordType(option.type)}>{option.type === "import_cost_input" ? "Cost inputs · gated" : option.label}</button>)}</div>
          {recordType !== "staff" && <label className="client-context">CLIENT WORKSPACE<select value={selectedClientId} onChange={e => setSelectedClientId(e.target.value)}><option value="">Select a client…</option>{clients.map(c => <option key={c.id} value={c.id}>{c.display_name || c.legal_name}</option>)}</select></label>}
          {section === "imports" && <p className="gate-note">Cost-input capture and calculation stay locked until the Gate 5-A practitioner discovery brief, approved input dictionary and independent worked example are recorded. Use the Discovery workspace to document questions; no customs or landed-cost assumptions are applied.</p>}
          <div className="record-grid">{records.map(r => <article className="record-card" key={r.id}><div className="record-card-top"><span className="status">{r.status}</span><small>{new Date(r.updated_at).toLocaleDateString()}</small></div><h3>{r.title}</h3><p>{r.notes || "No additional notes"}</p><div className="record-meta"><span>{r.reference || "No source reference"}</span>{r.amount_text && <strong>{r.amount_text}</strong>}</div><button className="text-button" onClick={() => setRecordEditor(r)}>Open record →</button></article>)}
          {records.length === 0 && <div className="empty-card"><strong>No records yet</strong><p>Create a record to start this workflow. Data is stored locally in the demo database.</p></div>}</div>
        </section>;
      })()}

      {section === "activity" && <section className="content-panel"><div className="section-head"><div><span className="overline">LOCAL AUDIT HISTORY</span><h2>Recent activity</h2><p>Append-only local events for the synthetic workspace.</p></div><button className="secondary" onClick={() => void refresh("activity")}>Refresh</button></div>
        <div className="event-list">{events.map(e => <div className="event-row" key={e.id}><span className="event-mark" /><div><strong>{e.event_type.replaceAll("_"," ")}</strong><small>{e.actor_name} · {e.target_type} · {e.target_id}</small><details className="event-details"><summary>Event data</summary><pre>{formatSnapshot(e.payload_json)}</pre></details></div><time>{new Date(e.occurred_at_utc).toLocaleString()}</time></div>)}{events.length === 0 && <p className="empty">No activity recorded yet.</p>}</div>
      </section>}

      <footer className="main-footer"><span>ARM · Local demonstration workspace</span><span>Changes are saved on this device only</span></footer>
    </main>

    {historyClient && <div className="modal-backdrop" role="presentation" onMouseDown={(e) => { if (e.target === e.currentTarget) setHistoryClient(null); }}><section className="modal history-modal"><div className="modal-head"><div><span className="overline">CLIENT CHANGE HISTORY</span><h2>{historyClient.display_name || historyClient.legal_name}</h2><p>Before-and-after profile snapshots, actor and time. This history is local to the synthetic workspace.</p></div><button type="button" className="close" onClick={() => setHistoryClient(null)}>×</button></div><div className="change-list">{clientChanges.map(change => <article className="change-item" key={change.id}><div className="change-heading"><strong>{change.action === "created" ? "Profile created" : "Profile updated"}</strong><span>{change.actor_name} · {new Date(change.changed_at).toLocaleString()}</span></div><div className="change-columns"><div><small>BEFORE</small><pre>{formatSnapshot(change.before_json)}</pre></div><div><small>AFTER</small><pre>{formatSnapshot(change.after_json)}</pre></div></div></article>)}{clientChanges.length === 0 && <p className="empty">No profile changes recorded yet.</p>}</div></section></div>}

    {noteEditor !== undefined && <div className="modal-backdrop" role="presentation" onMouseDown={(e) => { if (e.target === e.currentTarget) setNoteEditor(undefined); }}><form className="modal note-modal" onSubmit={saveNote}><div className="modal-head"><div><span className="overline">INCREMENT 0 · LOCAL DRAFT</span><h2>{noteEditor ? "Edit local note" : "New local note"}</h2><p>Not a task, workpaper, finding or professional approval.</p></div><button type="button" className="close" onClick={() => setNoteEditor(undefined)}>×</button></div><div className="form-grid"><label className="wide">Title *<input name="title" required minLength={1} maxLength={120} defaultValue={noteEditor?.title ?? ""} /></label><label>Related client (optional)<select name="client_id" defaultValue={noteEditor?.client_id ?? ""}><option value="">No client</option>{clients.map(c => <option key={c.id} value={c.id}>{c.display_name || c.legal_name}</option>)}</select></label><label className="wide">Note<textarea name="body" maxLength={2000} defaultValue={noteEditor?.body ?? ""} /></label></div><div className="modal-actions"><button type="button" className="secondary" onClick={() => setNoteEditor(undefined)}>Cancel</button><button className="primary" disabled={!canMutate || busy}>Save local draft</button></div></form></div>}

    {clientEditor !== undefined && <div className="modal-backdrop" role="presentation" onMouseDown={(e) => { if (e.target === e.currentTarget) setClientEditor(undefined); }}><form className="modal" onSubmit={saveClient}>
      <div className="modal-head"><div><span className="overline">CLIENT PROFILE</span><h2>{clientEditor ? "Edit client" : "Add a client"}</h2><p>Only a record kind and legal name are required to save a draft.</p></div><button type="button" className="close" onClick={() => setClientEditor(undefined)}>×</button></div>
      <div className="form-grid"><label>Record type<select name="record_kind" defaultValue={clientEditor?.record_kind ?? "Company"}><option>Company</option><option>Individual</option><option>Other organization</option></select></label><label>Legal or full name *<input name="legal_name" required minLength={2} maxLength={200} defaultValue={clientEditor?.legal_name ?? ""} /></label><label>Display name<input name="display_name" maxLength={200} defaultValue={clientEditor?.display_name ?? ""} /></label><label>TIN (not verified)<input name="tin_raw" maxLength={40} defaultValue={clientEditor?.tin_raw ?? ""} /></label><label>Sector / activity<input name="sector" defaultValue={clientEditor?.sector ?? ""} /></label><label>Primary contact<input name="contact_name" defaultValue={clientEditor?.contact_name ?? ""} /></label><label>Contact email<input type="email" name="contact_email" defaultValue={clientEditor?.contact_email ?? ""} /></label><label>Contact phone<input name="contact_phone" defaultValue={clientEditor?.contact_phone ?? ""} /></label><label className="wide">Address<input name="address_text" defaultValue={clientEditor?.address_text ?? ""} /></label><label>Internal reference<input name="internal_reference" defaultValue={clientEditor?.internal_reference ?? ""} /></label><label className="wide">Notes<textarea name="notes" maxLength={2000} defaultValue={clientEditor?.notes ?? ""} /></label></div>
      <div className="modal-actions"><button type="button" className="secondary" onClick={() => setClientEditor(undefined)}>Cancel</button><button className="primary" disabled={!canMutate || busy}>Save draft</button></div>
    </form></div>}

    {recordEditor && <div className="modal-backdrop" role="presentation" onMouseDown={(e) => { if (e.target === e.currentTarget) setRecordEditor(null); }}><form className="modal" onSubmit={saveRecord}>
      <div className="modal-head"><div><span className="overline">WORKFLOW RECORD</span><h2>{recordEditor.id ? "Edit record" : "Add record"}</h2><p>Store source information and review state; do not infer professional conclusions.</p></div><button type="button" className="close" onClick={() => setRecordEditor(null)}>×</button></div>
      <div className="form-grid"><label className="wide">Title / primary name *<input name="title" required maxLength={200} defaultValue={recordEditor.title} /></label><label>Client<select name="client_id" defaultValue={recordEditor.client_id ?? ""}><option value="">No client selected</option>{clients.map(c => <option value={c.id} key={c.id}>{c.display_name || c.legal_name}</option>)}</select></label><label>Status<select name="status" defaultValue={recordEditor.status}>{(statusOptions[recordType] ?? recordTypes[section as keyof typeof recordTypes]?.statuses ?? []).map(s => <option key={s}>{s}</option>)}</select></label><label>Source / reference<input name="reference" defaultValue={recordEditor.reference} /></label><label>{recordType === "invoice" || recordType === "historical_record" ? "Amount before VAT / source amount (text)" : "Amount / value (text only)"}<input name="amount_text" defaultValue={recordEditor.amount_text} placeholder="Preserve source notation; no calculation" /></label>{(recordFields[recordType] ?? []).map(field => <label key={field.key} className={field.wide ? "wide" : ""}>{field.label}{field.kind === "select" ? <select name={field.key} defaultValue={recordPayload[field.key] ?? field.options?.[0] ?? ""}>{(field.options ?? []).map(option => <option key={option}>{option}</option>)}</select> : field.kind === "textarea" ? <textarea name={field.key} defaultValue={recordPayload[field.key] ?? ""} placeholder={field.placeholder} /> : <input name={field.key} type={field.kind === "date" ? "date" : "text"} defaultValue={recordPayload[field.key] ?? ""} placeholder={field.placeholder} />}</label>)}<label className="wide">Notes and assumptions<textarea name="notes" defaultValue={recordEditor.notes} /></label></div>
      <div className="modal-actions"><button type="button" className="secondary" onClick={() => setRecordEditor(null)}>Cancel</button><button className="primary" disabled={!canMutate || busy}>Save local record</button></div>
    </form></div>}
  </div>;
}
