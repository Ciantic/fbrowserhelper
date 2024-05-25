import type { Browser, Runtime } from "npm:@types/webextension-polyfill";
import { postMessage, listenToMessage, listenToDisconnect } from "./messaging.ts";

declare const browser: Browser;

if ("chrome" in window && !("browser" in window)) {
    // @ts-ignore: Chrome is not defined
    browser = chrome;
}

type WindowId = number;
type HWND = number;
type WindowInfo = {
    hwnd: HWND;
    className: string;
};

const windowInfoMap = new Map<WindowId, WindowInfo>();
const taskbarButtonGroups = new Map<string, WindowId[]>();

/*
browser.tabs.onActivated.addListener((activeInfo) => {
    console.log("Tab activated: ", activeInfo);
}); 

browser.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
    console.log("Tab updated: ", tabId, changeInfo, tab);
});
*/

let curWindowId: WindowId | null = null;
browser.windows.onFocusChanged.addListener((windowId) => {
    curWindowId = windowId;

    // If windowId is -1, it means no browser window is focused.
    if (windowId === -1) {
        return;
    }

    // If we already have the window info, no need to get it again.
    if (windowInfoMap.has(windowId)) {
        return;
    }

    postMessage({
        type: "getActiveWindow",
    });
});

listenToMessage((msg) => {
    if (msg.type === "activeWindow") {
        // Ignore if no window is focused.
        if (!curWindowId || curWindowId === -1) {
            return;
        }

        // Ignore non-browser windows.
        if (
            !msg.processName.endsWith("chrome.exe") &&
            !msg.processName.endsWith("msedge.exe") &&
            !msg.processName.endsWith("firefox.exe")
        ) {
            return;
        }

        // Ignore if the window is already stored.
        if (windowInfoMap.has(curWindowId)) {
            return;
        }

        console.info("Storing", curWindowId, msg);

        windowInfoMap.set(curWindowId, {
            hwnd: msg.hwnd,
            className: msg.className,
        });
    }
});

// Browser action
browser.browserAction.onClicked.addListener(() => {
    postMessage({
        type: "getActiveWindow",
    });
});
