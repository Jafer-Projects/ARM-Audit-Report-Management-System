import { invoke } from "@tauri-apps/api/core";
import type { Client, ClientInput, EventItem, RecordInput, WorkspaceInfo, WorkspaceRecord } from "../types";

export const desktop = {
  initialize: () => invoke<WorkspaceInfo>("initialize_workspace"),
  setPersona: (persona: string) => invoke<WorkspaceInfo>("set_demo_persona", { persona }),
  clients: (includeArchived = false) => invoke<Client[]>("list_clients", { includeArchived }),
  saveClient: (input: ClientInput) => invoke<Client>("save_client", { input }),
  archiveClient: (id: string) => invoke<void>("archive_client", { id }),
  records: (recordType: string) => invoke<WorkspaceRecord[]>("list_records", { recordType }),
  saveRecord: (input: RecordInput) => invoke<WorkspaceRecord>("save_record", { input }),
  events: () => invoke<EventItem[]>("list_events"),
  reset: () => invoke<void>("reset_demo_data"),
};
