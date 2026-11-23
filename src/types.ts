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

export type HistoricalImportRow = { source_row: number; transaction_date: string; description: string; amount_text: string; currency: string; supplier: string; supplier_tin: string; vat_amount: string; invoice_reference: string };
export type HistoricalImportResult = { batch_id: string; source_checksum_sha256: string; source_row_count: number; imported_row_count: number };
export type HistoricalImportBatch = { id: string; source_filename: string; source_checksum_sha256: string; source_row_count: number; imported_row_count: number; imported_at: string };

export type TaskReviewNote = { id: string; task_id: string; category: string; description: string; affected_record_reference: string; required_response: string; status: string; preparer_response: string; reviewer_name: string; response_actor_name: string; disposition: string; disposition_actor_name: string; created_at: string; updated_at: string };

export type WorkspaceRecordVersion = { id: string; row_version: number; actor_name: string; changed_at: string; snapshot_json: string };

export type ImportCostLineResult = {
  item_reference: string; description: string; source_reference: string; fx_rate_source: string; fx_rate_date: string; fx_authorized_by: string; quantity_micros: number;
  goods_value_minor: number; allocated_cost_minor: number; landed_cost_minor: number;
};
export type ImportCostComponentAllocation = { item_reference: string; amount_minor: number };
export type ImportCostComponentResult = {
  component_reference: string; category: string; source_reference: string; fx_rate_source: string; fx_rate_date: string; fx_authorized_by: string; source_currency: string; source_amount_minor: number;
  base_amount_minor: number; treatment: string; allocation_basis: string; allocations: ImportCostComponentAllocation[];
};
export type ImportCostResult = {
  base_currency: string; total_goods_value_minor: number; total_capitalized_cost_minor: number;
  total_landed_cost_minor: number; excluded_or_pending_minor: number;
  lines: ImportCostLineResult[]; components: ImportCostComponentResult[]; limitations: string[];
};

export type SourceDocumentMeta = {
  id: string; record_id: string; file_name: string; media_type: string;
  size_bytes: number; sha256: string; actor_name: string; created_at: string;
};
export type SourceDocumentContent = { metadata: SourceDocumentMeta; content: number[] };

export type InvoiceLineDraft = {
  item_reference: string; category_reference: string; description: string;
  quantity_text: string; unit_text: string; unit_price_text: string;
  line_amount_text: string; tax_label: string; tax_amount_text: string;
};
