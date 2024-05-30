import type {} from "npm:@types/chrome";
// import type { Browser, Runtime, Tabs } from "npm:@types/webextension-polyfill";
import { postMessage, listenToMessage, listenToDisconnect } from "./messaging.ts";
import { PortableLoader } from "https://deno.land/x/esbuild_deno_loader@0.9.0/src/loader_portable.ts";

// declare const browser: Browser;
declare const chrome: typeof globalThis.chrome;

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

function updateWindowIcon(tab: chrome.tabs.Tab) {
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
        iconUrl: tab.url,
    });
}

chrome.tabs.onActivated.addListener(async (activeInfo) => {
    console.log("Tab activated: ", activeInfo);
    const tab = await chrome.tabs.get(activeInfo.tabId);
    updateWindowIcon(tab);
});

chrome.tabs.onUpdated.addListener(async (tabId, changeInfo, tab) => {
    // When changing the favicon url
    if (changeInfo.favIconUrl) {
        if (tab.active) {
            console.log("Updated tab");
            updateWindowIcon(tab);
        }
        // console.log("Tab updated: ", changeInfo.favIconUrl, tabId, tab.windowId);
    }
});

// browser.tabs.onHighlighted.addListener((highlightInfo) => {
//     console.log("Tab highlighted: ", highlightInfo);
//     browser.tabs.get(highlightInfo.tabIds[0]).then((tab) => {
//         updateWindowIcon(tab);
//     });
// });

let curWindowId: WindowId | null = null;
chrome.windows.onFocusChanged.addListener(async (windowId) => {
    // If windowId is -1, it means no browser window is focused.
    if (
        windowId === chrome.windows.WINDOW_ID_NONE ||
        windowId === chrome.windows.WINDOW_ID_CURRENT
    ) {
        return;
    }

    // Store the current windowId.
    curWindowId = windowId;

    // If we don't have window info, request it.
    if (!windowInfoMap.has(windowId)) {
        postMessage({
            type: "getActiveWindow",
        });
    }

    // Update icon of active tab in the window
    const tabs = await chrome.tabs.query({
        active: true,
        windowId: windowId,
    });

    if (tabs.length > 0) {
        updateWindowIcon(tabs[0]);
    }
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

        windowInfoMap.set(curWindowId, {
            hwnd: msg.hwnd,
            className: msg.className,
        });

        postMessage({
            type: "ungroupTaskbarButton",
            hwnd: msg.hwnd,
            newId: curWindowId.toString(),
        });
    }
});

listenToDisconnect(() => {
    console.log("Disconnected from native app.");
});

// Browser action
chrome.action.onClicked.addListener(() => {
    postMessage({
        type: "getActiveWindow",
    });
});
