import { invoke } from "@tauri-apps/api/core";
import type { CloudSession } from "./cloudAuth";

export type PersistedCloudSession = { apiBaseUrl: string; selectedTenantId: string; session: CloudSession };

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
  clear: () => invoke<void>("clear_cloud_session"),
};
