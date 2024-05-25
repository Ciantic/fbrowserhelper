// extension/messaging.ts
if ("chrome" in window && !("browser" in window)) {
  browser = chrome;
}
var port = null;
var listeners = /* @__PURE__ */ new Set();
var disconnectListeners = /* @__PURE__ */ new Set();
function openOrReusePort() {
  if (!port) {
    port = browser.runtime.connectNative("f_browser_helper_app");
    port.onMessage.addListener((msg) => {
      for (const listener of listeners) {
        listener(msg);
      }
    });
    port.onDisconnect.addListener((port2) => {
      for (const listener of disconnectListeners) {
        listener(port2);
      }
    });
  }
  if (port.error) {
    console.warn("Error opening port: ", port.error);
    port = null;
  }
  return port;
}
function postMessage(msg) {
  try {
    openOrReusePort()?.postMessage(msg);
  } catch (e) {
    console.warn("Error posting message: ", e);
    port = null;
  }
}
function listenToMessage(cb) {
  listeners.add(cb);
  openOrReusePort()?.onMessage.addListener(cb);
}

// extension/background.ts
if ("chrome" in window && !("browser" in window)) {
  browser = chrome;
}
var windowInfoMap = /* @__PURE__ */ new Map();
var curWindowId = null;
browser.windows.onFocusChanged.addListener((windowId) => {
  curWindowId = windowId;
  if (windowId === -1) {
    return;
  }
  if (windowInfoMap.has(windowId)) {
    return;
  }
  postMessage({
    type: "getActiveWindow"
  });
});
listenToMessage((msg) => {
  if (msg.type === "activeWindow") {
    if (!curWindowId || curWindowId === -1) {
      return;
    }
    if (!msg.processName.endsWith("chrome.exe") && !msg.processName.endsWith("msedge.exe") && !msg.processName.endsWith("firefox.exe")) {
      return;
    }
    if (windowInfoMap.has(curWindowId)) {
      return;
    }
    console.info("Storing", curWindowId, msg);
    windowInfoMap.set(curWindowId, {
      hwnd: msg.hwnd,
      className: msg.className
    });
  }
});
browser.browserAction.onClicked.addListener(() => {
  postMessage({
    type: "getActiveWindow"
  });
});
