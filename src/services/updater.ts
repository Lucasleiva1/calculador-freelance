import { getVersion } from "@tauri-apps/api/app";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";

export interface UpdateProgress {
  downloadedBytes: number;
  totalBytes: number | null;
  finished: boolean;
}

export interface AvailableAppUpdate {
  currentVersion: string;
  version: string;
  date: string | null;
  notes: string | null;
  install: (onProgress: (progress: UpdateProgress) => void) => Promise<void>;
  dispose: () => Promise<void>;
}

function progressReporter(onProgress: (progress: UpdateProgress) => void) {
  let downloadedBytes = 0;
  let totalBytes: number | null = null;
  return (event: DownloadEvent) => {
    if (event.event === "Started") {
      totalBytes = event.data.contentLength ?? null;
      onProgress({ downloadedBytes, totalBytes, finished: false });
    } else if (event.event === "Progress") {
      downloadedBytes += event.data.chunkLength;
      onProgress({ downloadedBytes, totalBytes, finished: false });
    } else {
      onProgress({ downloadedBytes, totalBytes, finished: true });
    }
  };
}

function availableUpdate(update: Update): AvailableAppUpdate {
  return {
    currentVersion: update.currentVersion,
    version: update.version,
    date: update.date ?? null,
    notes: update.body ?? null,
    install: async (onProgress) => {
      await update.downloadAndInstall(progressReporter(onProgress));
      await relaunch();
    },
    dispose: () => update.close(),
  };
}

export const appUpdater = {
  currentVersion: () => getVersion(),
  check: async (): Promise<AvailableAppUpdate | null> => {
    const update = await check({ timeout: 15_000, allowDowngrades: false });
    return update ? availableUpdate(update) : null;
  },
};
