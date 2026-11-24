import { invoke } from "@tauri-apps/api/core";
import { secureSession } from "../secureSession";
import { cloudClients } from "./cloudClients";
import type { Client, ClientChange, ClientInput, DuplicateCandidate, EventItem, HistoricalImportBatch, HistoricalImportResult, HistoricalImportRow, ImportCostResult, SourceDocumentContent, SourceDocumentMeta, InvoiceDuplicateCandidate, LocalNote, RecordInput, TaskReviewNote, WorkspaceInfo, WorkspaceNotification, WorkspaceRecord, WorkspaceRecordVersion } from "../types";

async function requireDemoWorkspace(){if(await secureSession.load())throw new Error("This workflow is still available only in the isolated demo workspace; it cannot write to a cloud tenant until its domain API and offline policy are implemented.");}
async function demo<T>(command:string,args?:Record<string,unknown>):Promise<T>{await requireDemoWorkspace();return invoke<T>(command,args);}

export const desktop = {
 initialize: async ():Promise<WorkspaceInfo> => await secureSession.load() ? cloudClients.workspace() : invoke<WorkspaceInfo>("initialize_workspace"),
 setPersona: async (persona:string) => {await requireDemoWorkspace();return invoke<WorkspaceInfo>("set_demo_persona",{persona});},
 clients: async (includeArchived=false) => await secureSession.load() ? cloudClients.list(includeArchived) : invoke<Client[]>("list_clients",{includeArchived}),
 findClientDuplicates: async (legalName:string,tinRaw:string,excludeId?:string) => await secureSession.load() ? cloudClients.duplicates(legalName,tinRaw,excludeId) : invoke<DuplicateCandidate[]>("find_client_duplicates",{legalName,tinRaw,excludeId}),
 saveClient: async (input:ClientInput) => await secureSession.load() ? cloudClients.save(input) : invoke<Client>("save_client",{input}),
 archiveClient: async (id:string) => await secureSession.load() ? cloudClients.transition(id,"archive") : demo<void>("archive_client",{id}),
 clientChanges: async (clientId:string) => await secureSession.load() ? cloudClients.history(clientId) : invoke<ClientChange[]>("list_client_changes",{clientId}),
 activateClient: async (id:string) => await secureSession.load() ? cloudClients.transition(id,"activate") : demo<void>("activate_client",{id}),
 restoreClient: async (id:string) => await secureSession.load() ? cloudClients.transition(id,"restore") : demo<void>("restore_client",{id}),
 notes: async () => await secureSession.load() ? [] : invoke<LocalNote[]>("list_notes"),
 saveNote: (input:{id?:string;client_id?:string|null;title:string;body:string}) => demo<LocalNote>("save_note",{input}),
 recordVersions: (recordId:string) => demo<WorkspaceRecordVersion[]>("list_workspace_record_versions",{recordId}),
 taskReviewNotes: (taskId:string) => demo<TaskReviewNote[]>("list_task_review_notes",{taskId}),
 addTaskReviewNote: (input:{taskId:string;category:string;description:string;affectedRecordReference:string;requiredResponse:string}) => demo<TaskReviewNote>("add_task_review_note",input),
 respondTaskReviewNote: (noteId:string,response:string) => demo<void>("respond_task_review_note",{noteId,response}),
 dispositionTaskReviewNote: (noteId:string,decision:string,reason:string) => demo<void>("disposition_task_review_note",{noteId,decision,reason}),
 importHistoricalBatch: (input:{client_id:string;source_filename:string;source_content:string;mapping_json:string;rows:HistoricalImportRow[]}) => demo<HistoricalImportResult>("import_historical_batch",{input}),
 importBatches: (clientId:string) => demo<HistoricalImportBatch[]>("list_historical_import_batches",{clientId}),
 findInvoiceDuplicates: (clientId:string,invoiceNumber:string,excludeId?:string) => demo<InvoiceDuplicateCandidate[]>("find_invoice_duplicates",{clientId,invoiceNumber,excludeId}),
 calculateImportCostPreview: (clientId:string,caseReference:string,baseCurrency:string) => demo<ImportCostResult>("calculate_import_cost_preview",{clientId,caseReference,baseCurrency}),
 attachSourceDocument: (input:{clientId:string;recordId:string;fileName:string;mediaType:string;content:number[]}) => demo<SourceDocumentMeta>("attach_source_document",input),
 listSourceDocuments: (recordId:string) => demo<SourceDocumentMeta[]>("list_source_documents",{recordId}),
 readSourceDocument: (id:string) => demo<SourceDocumentContent>("read_source_document",{id}),
 notifications: async () => await secureSession.load() ? [] : invoke<WorkspaceNotification[]>("list_workspace_notifications"),
 markNotificationRead: (id:string) => demo<void>("mark_workspace_notification_read",{id}),
 records: (recordType:string,clientId?:string) => demo<WorkspaceRecord[]>("list_records",{recordType,clientId:clientId??null}),
 saveRecord: (input:RecordInput) => demo<WorkspaceRecord>("save_record",{input}),
 events: async () => await secureSession.load() ? [] : invoke<EventItem[]>("list_events"),
 reset: () => demo<void>("reset_demo_data"),
};
