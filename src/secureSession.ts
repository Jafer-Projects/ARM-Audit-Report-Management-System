import { invoke } from "@tauri-apps/api/core";
import { cloudAuth, type CloudSession } from "./cloudAuth";

export type PersistedCloudSession = { apiBaseUrl: string; selectedTenantId: string; session: CloudSession };

let rotation: Promise<PersistedCloudSession> | null = null;

export const secureSession = {
  load: async (): Promise<PersistedCloudSession | null> => {
    const raw = await invoke<string | null>("load_cloud_session");
    if (!raw) return null;
    try {
      const value = JSON.parse(raw) as PersistedCloudSession;
      if (!value.apiBaseUrl || !value.session?.accessToken || !value.session?.refreshToken) {
        throw new Error("The saved cloud session is incomplete.");
      }
      return value;
    } catch (error) {
      await invoke("clear_cloud_session");
      throw new Error("The saved cloud session was invalid and has been cleared.", { cause: error });
    }
  },
  save: (value: PersistedCloudSession) => invoke<void>("store_cloud_session", { sessionJson: JSON.stringify(value) }),
  refresh: (): Promise<PersistedCloudSession> => {
    if (!rotation) {
      rotation = (async () => {
        const current = await secureSession.load();
        if (!current) throw new Error("No saved cloud session is available to refresh.");
        const next = await cloudAuth.refresh(current.apiBaseUrl, current.session.refreshToken);
        const updated = { ...current, session: { ...current.session, ...next } };
        await secureSession.save(updated);
        return updated;
      })().finally(() => { rotation = null; });
    }
    return rotation;
  },
  clear: () => invoke<void>("clear_cloud_session"),
};
