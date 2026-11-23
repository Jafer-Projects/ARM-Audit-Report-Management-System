import { useEffect, useMemo, useState, type FormEvent } from "react";
import { desktop } from "./services/desktop";
import type { Client, EventItem, Persona, WorkspaceInfo, WorkspaceRecord } from "./types";

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
const personaLabels: Record<Persona, string> = {
  administrator: "Demo Firm Administrator", manager: "Demo Engagement Manager",
  auditor: "Demo Auditor", reviewer: "Demo Read-only Reviewer",
};

export default function App() {
  const [workspace, setWorkspace] = useState<WorkspaceInfo | null>(null);
  const [section, setSection] = useState<Section>("home");
  const [clients, setClients] = useState<Client[]>([]);
  const [records, setRecords] = useState<WorkspaceRecord[]>([]);
  const [events, setEvents] = useState<EventItem[]>([]);
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
      if (nextSection === "clients") setClients(await desktop.clients(includeArchived));
      if (nextSection === "activity") setEvents(await desktop.events());
      const config = Object.values(recordTypes).find((item) => item.type === recordType);
      if (config && ["catalogue", "history", "work", "imports"].includes(nextSection)) setRecords(await desktop.records(config.type));
    } catch (e) { setError(String(e)); }
  }
  useEffect(() => { void refresh("home"); }, []);
  useEffect(() => { if (section === "clients" || section === "activity") void refresh(section); }, [section, includeArchived]);
  useEffect(() => {
    const config = recordTypes[section as keyof typeof recordTypes];
    if (config) { setRecordType(config.type); void desktop.records(config.type).then(setRecords).catch((e) => setError(String(e))); }
  }, [section]);

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
      await desktop.saveClient({
        ...input, id: clientEditor?.id, row_version: clientEditor?.row_version,
        record_kind: input.record_kind, legal_name: input.legal_name,
        display_name: input.display_name ?? "", tin_raw: input.tin_raw ?? "", sector: input.sector ?? "",
        contact_name: input.contact_name ?? "", contact_email: input.contact_email ?? "",
        contact_phone: input.contact_phone ?? "", address_text: input.address_text ?? "",
        internal_reference: input.internal_reference ?? "", notes: input.notes ?? "",
      });
      setClientEditor(undefined); setClients(await desktop.clients(includeArchived));
    } catch (e) { setError(String(e)); } finally { setBusy(false); }
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
    const config = Object.values(recordTypes).find((item) => item.type === recordType);
    if (!config) return;
    try {
      setBusy(true); setError("");
      await desktop.saveRecord({
        id: recordEditor?.id || undefined, row_version: recordEditor?.id ? recordEditor.row_version : undefined,
        client_id: v.client_id || null, record_type: recordType, title: v.title,
        status: v.status || config.statuses[0], reference: v.reference || "",
        amount_text: v.amount_text || "", notes: v.notes || "",
      });
      setRecordEditor(null); setRecords(await desktop.records(recordType));
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

      {section === "clients" && <section className="content-panel">
        <div className="section-head"><div><span className="overline">CLIENT WORKSPACE</span><h2>Client register</h2><p>Keep a clear profile for each company or individual.</p></div><button className="primary" disabled={!canMutate} onClick={() => setClientEditor(null)}>+ Add client</button></div>
        <div className="toolbar"><input aria-label="Search clients" placeholder="Search name or TIN…" value={search} onChange={(e) => setSearch(e.target.value)} /><label className="check"><input type="checkbox" checked={includeArchived} onChange={(e) => setIncludeArchived(e.target.checked)} /> Include archived</label><span>{filteredClients.length} records</span></div>
        <div className="table-wrap"><table><thead><tr><th>CLIENT</th><th>TYPE</th><th>TIN</th><th>STATUS</th><th>UPDATED</th><th /></tr></thead><tbody>
          {filteredClients.map(c => <tr key={c.id}><td><strong>{c.display_name || c.legal_name}</strong><small>{c.legal_name}</small></td><td>{c.record_kind}</td><td>{c.tin_raw || "Not provided"}</td><td><span className={c.status === "Archived" ? "status muted" : "status"}>{c.status}</span></td><td>{new Date(c.updated_at).toLocaleDateString()}</td><td className="actions"><button onClick={() => setClientEditor(c)}>Open</button>{c.status !== "Archived" && <button disabled={!canMutate} onClick={() => void archiveClient(c)}>Archive</button>}</td></tr>)}
          {filteredClients.length === 0 && <tr><td colSpan={6} className="empty">No clients match this search. Add a client to begin.</td></tr>}
        </tbody></table></div>
      </section>}

      {(["catalogue","history","work","imports"] as Section[]).includes(section) && (() => {
        const config = recordTypes[section as keyof typeof recordTypes];
        if (!config) return null;
        return <section className="content-panel">
          <div className="section-head"><div><span className="overline">{config.label.toUpperCase()}</span><h2>{nav.find(n => n.id === section)?.label}</h2><p>{config.hint}</p></div><button className="primary" disabled={!canMutate} onClick={() => setRecordEditor({ id:"",client_id:null,record_type:config.type,title:"",status:config.statuses[0],reference:"",amount_text:"",notes:"",row_version:0,updated_at:"" })}>+ Add record</button></div>
          <div className="record-grid">{records.map(r => <article className="record-card" key={r.id}><div className="record-card-top"><span className="status">{r.status}</span><small>{new Date(r.updated_at).toLocaleDateString()}</small></div><h3>{r.title}</h3><p>{r.notes || "No additional notes"}</p><div className="record-meta"><span>{r.reference || "No source reference"}</span>{r.amount_text && <strong>{r.amount_text}</strong>}</div><button className="text-button" onClick={() => setRecordEditor(r)}>Open record →</button></article>)}
          {records.length === 0 && <div className="empty-card"><strong>No records yet</strong><p>Create a record to start this workflow. Data is stored locally in the demo database.</p></div>}</div>
        </section>;
      })()}

      {section === "activity" && <section className="content-panel"><div className="section-head"><div><span className="overline">LOCAL AUDIT HISTORY</span><h2>Recent activity</h2><p>Append-only local events for the synthetic workspace.</p></div><button className="secondary" onClick={() => void refresh("activity")}>Refresh</button></div>
        <div className="event-list">{events.map(e => <div className="event-row" key={e.id}><span className="event-mark" /><div><strong>{e.event_type.replaceAll("_"," ")}</strong><small>{e.actor_name} · {e.target_type} · {e.target_id}</small></div><time>{new Date(e.occurred_at_utc).toLocaleString()}</time></div>)}{events.length === 0 && <p className="empty">No activity recorded yet.</p>}</div>
      </section>}

      <footer className="main-footer"><span>ARM · Local demonstration workspace</span><span>Changes are saved on this device only</span></footer>
    </main>

    {clientEditor !== undefined && <div className="modal-backdrop" role="presentation" onMouseDown={(e) => { if (e.target === e.currentTarget) setClientEditor(undefined); }}><form className="modal" onSubmit={saveClient}>
      <div className="modal-head"><div><span className="overline">CLIENT PROFILE</span><h2>{clientEditor ? "Edit client" : "Add a client"}</h2><p>Only a record kind and legal name are required to save a draft.</p></div><button type="button" className="close" onClick={() => setClientEditor(undefined)}>×</button></div>
      <div className="form-grid"><label>Record type<select name="record_kind" defaultValue={clientEditor?.record_kind ?? "Company"}><option>Company</option><option>Individual</option><option>Other organization</option></select></label><label>Legal or full name *<input name="legal_name" required minLength={2} maxLength={200} defaultValue={clientEditor?.legal_name ?? ""} /></label><label>Display name<input name="display_name" maxLength={200} defaultValue={clientEditor?.display_name ?? ""} /></label><label>TIN (not verified)<input name="tin_raw" maxLength={40} defaultValue={clientEditor?.tin_raw ?? ""} /></label><label>Sector / activity<input name="sector" defaultValue={clientEditor?.sector ?? ""} /></label><label>Primary contact<input name="contact_name" defaultValue={clientEditor?.contact_name ?? ""} /></label><label>Contact email<input type="email" name="contact_email" defaultValue={clientEditor?.contact_email ?? ""} /></label><label>Contact phone<input name="contact_phone" defaultValue={clientEditor?.contact_phone ?? ""} /></label><label className="wide">Address<input name="address_text" defaultValue={clientEditor?.address_text ?? ""} /></label><label>Internal reference<input name="internal_reference" defaultValue={clientEditor?.internal_reference ?? ""} /></label><label className="wide">Notes<textarea name="notes" maxLength={2000} defaultValue={clientEditor?.notes ?? ""} /></label></div>
      <div className="modal-actions"><button type="button" className="secondary" onClick={() => setClientEditor(undefined)}>Cancel</button><button className="primary" disabled={!canMutate || busy}>Save draft</button></div>
    </form></div>}

    {recordEditor && <div className="modal-backdrop" role="presentation" onMouseDown={(e) => { if (e.target === e.currentTarget) setRecordEditor(null); }}><form className="modal" onSubmit={saveRecord}>
      <div className="modal-head"><div><span className="overline">WORKFLOW RECORD</span><h2>{recordEditor.id ? "Edit record" : "Add record"}</h2><p>Store source information and review state; do not infer professional conclusions.</p></div><button type="button" className="close" onClick={() => setRecordEditor(null)}>×</button></div>
      <div className="form-grid"><label className="wide">Title *<input name="title" required maxLength={200} defaultValue={recordEditor.title} /></label><label>Client<select name="client_id" defaultValue={recordEditor.client_id ?? ""}><option value="">No client selected</option>{clients.map(c => <option value={c.id} key={c.id}>{c.display_name || c.legal_name}</option>)}</select></label><label>Status<select name="status" defaultValue={recordEditor.status}>{recordTypes[section as keyof typeof recordTypes]?.statuses.map(s => <option key={s}>{s}</option>)}</select></label><label>Source / reference<input name="reference" defaultValue={recordEditor.reference} /></label><label>Amount / value (text only)<input name="amount_text" defaultValue={recordEditor.amount_text} placeholder="Preserve source notation; no calculation" /></label><label className="wide">Notes and assumptions<textarea name="notes" defaultValue={recordEditor.notes} /></label></div>
      <div className="modal-actions"><button type="button" className="secondary" onClick={() => setRecordEditor(null)}>Cancel</button><button className="primary" disabled={!canMutate || busy}>Save local record</button></div>
    </form></div>}
  </div>;
}
