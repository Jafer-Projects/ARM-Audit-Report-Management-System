import { invoke } from "@tauri-apps/api/core";
import type { Client, ClientChange, ClientInput, DuplicateCandidate, EventItem, HistoricalImportBatch, HistoricalImportResult, HistoricalImportRow, InvoiceDuplicateCandidate, LocalNote, RecordInput, WorkspaceInfo, WorkspaceRecord } from "../types";

export const desktop = {
  initialize: () => invoke<WorkspaceInfo>("initialize_workspace"),
  setPersona: (persona: string) => invoke<WorkspaceInfo>("set_demo_persona", { persona }),
  clients: (includeArchived = false) => invoke<Client[]>("list_clients", { includeArchived }),
  findClientDuplicates: (legalName: string, tinRaw: string, excludeId?: string) => invoke<DuplicateCandidate[]>("find_client_duplicates", { legalName, tinRaw, excludeId }),
  saveClient: (input: ClientInput) => invoke<Client>("save_client", { input }),
  archiveClient: (id: string) => invoke<void>("archive_client", { id }),
  clientChanges: (clientId: string) => invoke<ClientChange[]>("list_client_changes", { clientId }),
  activateClient: (id: string) => invoke<void>("activate_client", { id }),
  restoreClient: (id: string) => invoke<void>("restore_client", { id }),
  notes: () => invoke<LocalNote[]>("list_notes"),
  saveNote: (input: { id?: string; client_id?: string | null; title: string; body: string }) => invoke<LocalNote>("save_note", { input }),
  importHistoricalBatch: (input: { client_id: string; source_filename: string; source_content: string; mapping_json: string; rows: HistoricalImportRow[] }) => invoke<HistoricalImportResult>("import_historical_batch", { input }),
  importBatches: (clientId: string) => invoke<HistoricalImportBatch[]>("list_historical_import_batches", { clientId }),
  findInvoiceDuplicates: (clientId: string, invoiceNumber: string, excludeId?: string) => invoke<InvoiceDuplicateCandidate[]>("find_invoice_duplicates", { clientId, invoiceNumber, excludeId }),
  records: (recordType: string, clientId?: string) => invoke<WorkspaceRecord[]>("list_records", { recordType, clientId: clientId ?? null }),
  saveRecord: (input: RecordInput) => invoke<WorkspaceRecord>("save_record", { input }),
  events: () => invoke<EventItem[]>("list_events"),
  reset: () => invoke<void>("reset_demo_data"),
};
