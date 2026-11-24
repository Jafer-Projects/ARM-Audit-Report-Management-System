import { invoke } from "@tauri-apps/api/core";
import { secureSession } from "../secureSession";
import { cloudClients } from "./cloudClients";
import type { Client, ClientChange, ClientInput, DuplicateCandidate, EventItem, HistoricalImportBatch, HistoricalImportResult, HistoricalImportRow, ImportCostResult, SourceDocumentContent, SourceDocumentMeta, InvoiceDuplicateCandidate, LocalNote, RecordInput, TaskReviewNote, WorkspaceInfo, WorkspaceNotification, WorkspaceRecord, WorkspaceRecordVersion } from "../types";

const demoMode = import.meta.env.VITE_ARM_WORKSPACE_MODE === "demo";
async function requireDemoWorkspace(){if(!demoMode)throw new Error("The demonstration workspace is disabled in this build.");if(await secureSession.load())throw new Error("This workflow is still available only in the isolated demo workspace; it cannot write to a cloud tenant until its domain API and offline policy are implemented.");}
async function cloudOrDemo<T>(cloud:()=>Promise<T>,command:string,args?:Record<string,unknown>):Promise<T>{const saved=await secureSession.load();if(saved)return cloud();await requireDemoWorkspace();return invoke<T>(command,args);}
async function demo<T>(command:string,args?:Record<string,unknown>):Promise<T>{await requireDemoWorkspace();return invoke<T>(command,args);}

export const desktop = {
 initialize: async ():Promise<WorkspaceInfo> => {const saved=await secureSession.load();if(!saved&&import.meta.env.VITE_ARM_WORKSPACE_MODE==="demo"){const info=await invoke<WorkspaceInfo>("initialize_workspace");return {...info,workspace_mode:"demo",tenant_selected:false};}if(saved){if(!saved.selectedTenantId)return {firm_name:"Cloud account — select organization",actor_name:saved.session.user?.displayName??saved.session.user?.email??"Authenticated user",persona:"reviewer",clients_count:0,sync_state:"Select an active organization in Cloud connection before opening tenant data",workspace_mode:"cloud",tenant_selected:false,authenticated:true,permissions:[]};return cloudClients.workspace();}return {firm_name:"Sign in to cloud workspace",actor_name:"",persona:"reviewer",clients_count:0,sync_state:"Authentication required; demo workspace is disabled in this build",workspace_mode:"cloud",tenant_selected:false,authenticated:false,permissions:[]};},
 setPersona: async (persona:string) => {await requireDemoWorkspace();return invoke<WorkspaceInfo>("set_demo_persona",{persona});},
 clients: (includeArchived=false) => cloudOrDemo(()=>cloudClients.list(includeArchived),"list_clients",{includeArchived}),
 findClientDuplicates: (legalName:string,tinRaw:string,excludeId?:string) => cloudOrDemo(()=>cloudClients.duplicates(legalName,tinRaw,excludeId),"find_client_duplicates",{legalName,tinRaw,excludeId}),
 saveClient: (input:ClientInput) => cloudOrDemo(()=>cloudClients.save(input),"save_client",{input}),
 archiveClient: async (id:string) => await secureSession.load() ? cloudClients.transition(id,"archive") : demo<void>("archive_client",{id}),
 clientChanges: async (clientId:string) => await secureSession.load() ? cloudClients.history(clientId) : demo<ClientChange[]>("list_client_changes",{clientId}),
 activateClient: async (id:string) => await secureSession.load() ? cloudClients.transition(id,"activate") : demo<void>("activate_client",{id}),
 restoreClient: async (id:string) => await secureSession.load() ? cloudClients.transition(id,"restore") : demo<void>("restore_client",{id}),
 notes: async () => await secureSession.load() ? [] : demo<LocalNote[]>("list_notes"),
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
 notifications: async () => await secureSession.load() ? [] : demo<WorkspaceNotification[]>("list_workspace_notifications"),
 markNotificationRead: (id:string) => demo<void>("mark_workspace_notification_read",{id}),
 records: (recordType:string,clientId?:string) => demo<WorkspaceRecord[]>("list_records",{recordType,clientId:clientId??null}),
 saveRecord: (input:RecordInput) => demo<WorkspaceRecord>("save_record",{input}),
 events: async () => await secureSession.load() ? [] : demo<EventItem[]>("list_events"),
 reset: () => demo<void>("reset_demo_data"),
};
