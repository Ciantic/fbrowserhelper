// extension/messaging.ts
var port = null;
var listeners = /* @__PURE__ */ new Set();
var disconnectListeners = /* @__PURE__ */ new Set();
function openOrReusePort() {
  if (!port) {
    port = chrome.runtime.connectNative("f_browser_helper_app");
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
function listenToDisconnect(cb) {
  disconnectListeners.add(cb);
  openOrReusePort()?.onDisconnect.addListener(cb);
}

// extension/background.ts
var windowInfoMap = /* @__PURE__ */ new Map();
function updateWindowIcon(tab) {
  if (!tab.windowId) {
    console.warn("No windowId for tab: ", tab);
    return;
  }
  const windowInfo = windowInfoMap.get(tab.windowId);
  if (!windowInfo) {
    console.warn("No window info for window: ", tab.windowId);
    return;
  }
  console.log("Setting taskbar icon for window: ", windowInfo.hwnd, tab.url);
  postMessage({
    type: "setTaskbarIcon",
    hwnd: windowInfo.hwnd,
    iconUrl: tab.url
  });
}
chrome.tabs.onActivated.addListener(async (activeInfo) => {
  console.log("Tab activated: ", activeInfo);
  const tab = await chrome.tabs.get(activeInfo.tabId);
  updateWindowIcon(tab);
});
chrome.tabs.onUpdated.addListener(async (tabId, changeInfo, tab) => {
  if (changeInfo.favIconUrl) {
    if (tab.active) {
      updateWindowIcon(tab);
    }
  }
});
var curWindowId = null;
chrome.windows.onFocusChanged.addListener(async (windowId) => {
  if (windowId === chrome.windows.WINDOW_ID_NONE || windowId === chrome.windows.WINDOW_ID_CURRENT) {
    return;
  }
  curWindowId = windowId;
  if (!windowInfoMap.has(windowId)) {
    postMessage({
      type: "getActiveWindow"
    });
  }
  const tabs = await chrome.tabs.query({
    active: true,
    windowId
  });
  if (tabs.length > 0) {
    updateWindowIcon(tabs[0]);
  }
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
    postMessage({
      type: "ungroupTaskbarButton",
      hwnd: msg.hwnd,
      newId: curWindowId.toString()
    });
  }
});
listenToDisconnect(() => {
  console.log("Disconnected from native app.");
});
chrome.action.onClicked.addListener(() => {
  postMessage({
    type: "getActiveWindow"
  });
});
