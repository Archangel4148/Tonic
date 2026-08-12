/** Desktop fullscreen + keep-awake helpers. Failures are ignored where unsupported. */

type ScreenWakeLock = {
  request: (type: "screen") => Promise<WakeLockSentinelLike>;
};

type WakeLockSentinelLike = {
  release: () => Promise<void>;
  addEventListener: (type: "release", listener: () => void) => void;
};

let wakeLock: WakeLockSentinelLike | null = null;

export async function setStageFullscreen(enabled: boolean): Promise<void> {
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().setFullscreen(enabled);
    return;
  } catch {
    /* browser or mocked IPC */
  }
  try {
    if (enabled) {
      if (!document.fullscreenElement) {
        await document.documentElement.requestFullscreen?.();
      }
    } else if (document.fullscreenElement) {
      await document.exitFullscreen();
    }
  } catch {
    /* Fullscreen API unsupported or denied */
  }
}

export async function setKeepAwake(enabled: boolean): Promise<void> {
  try {
    if (!enabled) {
      await wakeLock?.release();
      wakeLock = null;
      return;
    }
    const nav = navigator as Navigator & { wakeLock?: ScreenWakeLock };
    if (!nav.wakeLock) {
      return;
    }
    wakeLock = await nav.wakeLock.request("screen");
    wakeLock.addEventListener("release", () => {
      wakeLock = null;
    });
  } catch {
    wakeLock = null;
  }
}

export async function refreshKeepAwake(): Promise<void> {
  if (document.visibilityState === "visible") {
    await setKeepAwake(true);
  }
}
