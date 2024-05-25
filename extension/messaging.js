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
function listenToDisconnect(cb) {
  disconnectListeners.add(cb);
  openOrReusePort()?.onDisconnect.addListener(cb);
}
export {
  listenToDisconnect,
  listenToMessage,
  postMessage
};
