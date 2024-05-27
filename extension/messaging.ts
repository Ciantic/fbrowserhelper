import type {} from "npm:@types/chrome";
// import type { Browser, Runtime, Tabs } from "npm:@types/webextension-polyfill";

// declare const browser: Browser;
declare const chrome: typeof globalThis.chrome;

type MessageFromBrowser =
    | { type: "getActiveWindow" }
    | { type: "ungroupTaskbarButton"; hwnd: number; newId: string }
    | { type: "setTaskbarIcon"; hwnd: number; iconUrl?: string }
    | { type: "quit" };

type MessageToBrowser =
    | { type: "activeWindow"; hwnd: number; className: string; processName: string; title: string }
    | { type: "ok" };

type MessageToError =
    | { type: "urlParsingError"; message: string }
    | { type: "error"; message: string }
    | { type: "ioError"; kind: string; message: string }
    | { type: "jsonParseError"; message: string }
    | { type: "panic"; message: string; file: string | null; line: number | null };

let port: chrome.runtime.Port | null = null;
let listeners = new Set<(msg: MessageToBrowser | MessageToError) => void>();
let disconnectListeners = new Set<(port: chrome.runtime.Port) => void>();

function openOrReusePort() {
    if (!port) {
        port = chrome.runtime.connectNative("f_browser_helper_app");
        port.onMessage.addListener((msg) => {
            for (const listener of listeners) {
                listener(msg);
            }
        });
        port.onDisconnect.addListener((port) => {
            for (const listener of disconnectListeners) {
                listener(port);
            }
        });
    }
    return port;
}

export function postMessage(msg: MessageFromBrowser) {
    try {
        openOrReusePort()?.postMessage(msg);
    } catch (e) {
        console.warn("Error posting message: ", e);
        port = null;
    }
}

export function listenToMessage(cb: (msg: MessageToBrowser | MessageToError) => void) {
    listeners.add(cb);
    openOrReusePort()?.onMessage.addListener(cb);
}

export function listenToDisconnect(cb: (port: chrome.runtime.Port) => void) {
    disconnectListeners.add(cb);
    openOrReusePort()?.onDisconnect.addListener(cb);
}
