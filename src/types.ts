export type Persona = "administrator" | "manager" | "auditor" | "reviewer";
export type WorkspaceInfo = {
  firm_name: string; actor_name: string; persona: Persona; clients_count: number; sync_state: string;
};
export type Client = {
  id: string; record_kind: string; legal_name: string; display_name: string; tin_raw: string;
  sector: string; contact_name: string; contact_email: string; contact_phone: string;
  address_text: string; internal_reference: string; notes: string; status: string;
  row_version: number; updated_at: string;
};
export type WorkspaceRecord = {
  id: string; client_id: string | null; record_type: string; title: string; status: string;
  reference: string; amount_text: string; notes: string; payload_json: string; row_version: number; updated_at: string;
};
export type EventItem = {
  id: string; event_type: string; target_type: string; target_id: string;
  occurred_at_utc: string; actor_name: string; payload_json: string;
};
export type ClientInput = Omit<Client, "id" | "row_version" | "status" | "updated_at"> & { id?: string; row_version?: number; duplicate_acknowledged?: boolean };
export type DuplicateCandidate = Pick<Client, "id" | "record_kind" | "legal_name" | "tin_raw" | "status">;
export type RecordInput = {
  id?: string; row_version?: number; client_id?: string | null; record_type: string;
  title: string; status: string; reference?: string; amount_text?: string; notes?: string; payload_json?: string; duplicate_acknowledged?: boolean; duplicate_acknowledgement_reason?: string;
};

export type LocalNote = { id: string; client_id: string | null; title: string; body: string; status: string; updated_at: string };

export type ClientChange = { id: string; action: string; changed_at: string; actor_name: string; before_json: string; after_json: string };

export type InvoiceDuplicateCandidate = { id: string; title: string; status: string; invoice_number: string; invoice_date: string };
