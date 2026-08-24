/** Desktop fullscreen + keep-awake helpers. Failures are ignored where unsupported. */

type ScreenWakeLock = {
  request: (type: "screen") => Promise<WakeLockSentinelLike>;
};

type WakeLockSentinelLike = {
  release: () => Promise<void>;
  addEventListener: (type: "release", listener: () => void) => void;
};

let wakeLock: WakeLockSentinelLike | null = null;

function isAndroidWebView(): boolean {
  return /Android/i.test(navigator.userAgent);
}

export async function setStageFullscreen(enabled: boolean): Promise<void> {
  // Android immersive mode is handled natively in MainActivity; window
  // fullscreen APIs are desktop-oriented and can be unstable on mobile.
  if (isAndroidWebView()) {
    return;
  }
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

export async function getStageFullscreen(): Promise<boolean> {
  if (isAndroidWebView()) {
    return false;
  }
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    return await getCurrentWindow().isFullscreen();
  } catch {
    return Boolean(document.fullscreenElement);
  }
}

export async function toggleStageFullscreen(): Promise<boolean> {
  const next = !(await getStageFullscreen());
  await setStageFullscreen(next);
  return next;
}

/** F11 or Alt+Enter — typical desktop fullscreen toggles. */
export function isFullscreenHotkey(event: KeyboardEvent): boolean {
  if (event.key === "F11") {
    return true;
  }
  return (
    event.key === "Enter" && event.altKey && !event.ctrlKey && !event.metaKey
  );
}

export function subscribeStageFullscreen(
  onChange: (enabled: boolean) => void,
): () => void {
  let cancelled = false;
  const sync = () => {
    void getStageFullscreen().then((enabled) => {
      if (!cancelled) {
        onChange(enabled);
      }
    });
  };
  document.addEventListener("fullscreenchange", sync);
  window.addEventListener("focus", sync);
  sync();
  return () => {
    cancelled = true;
    document.removeEventListener("fullscreenchange", sync);
    window.removeEventListener("focus", sync);
  };
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
