import path from "path";

import * as vscode from "vscode";
import { LanguageClient } from "vscode-languageclient/node";

import { resetDom, subscribeToDom, unsubscribeFromDom } from "./extension";
import { ScrollSyncer } from "./scroll-syncer";
import { statusBar } from "./status-bar";

/**
 * A map of document view panels used to ensure that only one
 * view of a document exists at a time
 */
const documentViewPanels = new Map<vscode.Uri, vscode.WebviewPanel>();

/**
 * A map of the "disposables" for each document that can be disposed of when
 * the view is closed
 */
const documentViewHandlers = new Map<vscode.Uri, vscode.Disposable[]>();

/**
 * A map of patch handler function for each subscription to a
 * document's DOM HTML
 */
export const documentPatchHandlers: Record<string, (patch: unknown) => void> =
  {};

/**
 * Register a handler for "stencila/publishDom" notifications that forwards
 * the patch onto the handler to the appropriate webview
 */
export function registerSubscriptionNotifications(
  context: vscode.ExtensionContext,
  client: LanguageClient
) {
  const handler = client.onNotification(
    "stencila/publishDom",
    ({ subscriptionId, patch }: { subscriptionId: string; patch: unknown }) => {
      const handler = documentPatchHandlers[subscriptionId];
      if (!handler) {
        console.error(`No handler for subscription ${subscriptionId}`);
      } else {
        handler(patch);
      }
    }
  );
  context.subscriptions.push(handler);
}

type ReceivedMessage = DomResetMessage | CommandMessage | ScrollSyncMessage;

interface DomResetMessage {
  type: "dom-reset";
}

interface CommandMessage {
  type: "command";
  command: string;
  args?: string[];
  nodeType?: string;
  nodeIds?: string[];
  nodeProperty?: [string, unknown];
  scope?: string;
}

interface ScrollSyncMessage {
  type: "scroll-sync";
  startId?: string;
  endId?: string;
  cursorId?: string;
}

/**
 * Create a WebView panel that display the document
 *
 * @param nodeId The id of the node that the document should scroll to
 * @param expand Whether the node card should be in expanded to show authorship/provenance
 */
export async function createDocumentViewPanel(
  context: vscode.ExtensionContext,
  editor: vscode.TextEditor,
  nodeId?: string,
  expandAuthors?: boolean
): Promise<vscode.WebviewPanel> {
  const documentUri = editor.document.uri;

  if (documentViewPanels.has(documentUri)) {
    // If there is already a panel open for this document, reveal it
    const panel = documentViewPanels.get(documentUri) as vscode.WebviewPanel;
    panel.reveal();

    // If `nodeId` param is defined, scroll webview to target node.
    if (nodeId) {
      panel.webview.postMessage({
        type: "view-node",
        nodeId,
        expandAuthors,
      });
    }

    return panel;
  }

  const filename = path.basename(documentUri.fsPath);

  // Folder containing bundled JS and other assets for the web view
  const webDist = vscode.Uri.joinPath(context.extensionUri, "out", "web");

  const panel = vscode.window.createWebviewPanel(
    "document-view",
    `Preview ${filename}`,
    vscode.ViewColumn.Beside,
    {
      enableScripts: true,
      retainContextWhenHidden: true,
      localResourceRoots: [webDist],
    }
  );
  panel.iconPath = vscode.Uri.joinPath(
    context.extensionUri,
    "icons",
    "stencila-128.png"
  );

  // Subscribe to updates of DOM HTML for document and get theme
  const [subscriptionId, themeName, viewHtml] = await subscribeToDom(
    documentUri,
    (patch: unknown) => {
      panel.webview.postMessage({
        type: "dom-patch",
        patch,
      });
    }
  );

  const themeCss = panel.webview.asWebviewUri(
    vscode.Uri.joinPath(webDist, "themes", `${themeName}.css`)
  );
  const viewCss = panel.webview.asWebviewUri(
    vscode.Uri.joinPath(webDist, "views", "vscode.css")
  );
  const viewJs = panel.webview.asWebviewUri(
    vscode.Uri.joinPath(webDist, "views", "vscode.js")
  );

  panel.webview.html = `
    <!DOCTYPE html>
      <html lang="en">
        <head>
            <title>Stencila: Document Preview</title>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <link rel="preconnect" href="https://fonts.googleapis.com">
            <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
            <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:ital,wght@0,100;0,200;0,300;0,400;0,500;0,600;0,700;1,100;1,200;1,300;1,400;1,500;1,600;1,700&family=Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900&display=swap" rel="stylesheet">
            <link title="theme:${themeName}" rel="stylesheet" type="text/css" href="${themeCss}">
            <link rel="stylesheet" type="text/css" href="${viewCss}">
            <script type="text/javascript" src="${viewJs}"></script>
        </head>
        <body style="background: white;">
          <stencila-vscode-view theme="${themeName}">
            ${viewHtml}
          </stencila-vscode-view>
          <script>
            const vscode = acquireVsCodeApi()
          </script>
        </body>
    </html>
  `;

  const disposables: vscode.Disposable[] = [];

  // Create a scroller sync for the view
  const scrollSync = new ScrollSyncer(editor, panel);
  disposables.push(scrollSync);

  // Listen to the view state changes of the webview panel to update status bar
  panel.onDidChangeViewState(
    (e) => {
      statusBar.updateForDocumentView(e.webviewPanel.active);
    },
    null,
    disposables
  );

  // Handle when the webview is disposed
  panel.onDidDispose(
    () => {
      // Unsubscribe from updates to DOM HTML
      unsubscribeFromDom(subscriptionId);

      // Remove from list of panels
      documentViewPanels.delete(documentUri);

      // Dispose handlers and remove from lists
      documentViewHandlers
        .get(documentUri)
        ?.forEach((handler) => handler.dispose());
      documentViewHandlers.delete(documentUri);
    },
    null,
    disposables
  );

  // Handle messages from the webview
  // It is necessary to translate the names of the Stencila document
  // command to the command and argument convention that the LSP uses
  panel.webview.onDidReceiveMessage(
    (message: ReceivedMessage) => {
      if (message.type === "dom-reset") {
        resetDom(subscriptionId);
        return;
      }

      if (message.type !== "command") {
        // Skip messages handled elsewhere
        return;
      }

      let command = message.command;

      let args;
      if (message.args) {
        args = message.args;
      } else {
        args = [
          message.nodeType,
          ...(message.nodeIds ? message.nodeIds : []),
          ...(message.nodeProperty ? message.nodeProperty : []),
        ];
      }

      if (command === "execute-nodes") {
        if (message.scope === "plus-before") {
          command = "run-before";
        } else if (message.scope === "plus-after") {
          command = "run-after";
        } else {
          command = "run-node";
        }
      }

      vscode.commands.executeCommand(
        `stencila.${command}`,
        documentUri.toString(),
        ...args
      );
    },
    null,
    disposables
  );

  // Track the webview by adding it to the map
  documentViewPanels.set(documentUri, panel);
  documentViewHandlers.set(documentUri, disposables);

  // If `nodeId` param is defined, scroll webview panel to target node.
  if (nodeId) {
    panel.webview.postMessage({
      type: "view-node",
      nodeId,
      expandAuthors,
    });
  }

  return panel;
}

/**
 * Close all document view panels
 *
 * This is necessary when the server is restarted because the client that the
 * panels are subscribed to will no longer exist.
 */
export function closeDocumentViewPanels() {
  if (documentViewPanels.size > 0) {
    vscode.window.showInformationMessage("Closing document view panels");
  } else {
    return;
  }

  for (const panel of documentViewPanels.values()) {
    panel.dispose();
  }

  documentViewPanels.clear();
}

export async function createChatAssistentPanel(
  context: vscode.ExtensionContext,
  editor: vscode.TextEditor,
) {
  const webDist = vscode.Uri.joinPath(context.extensionUri, "out", "web");

  const panel = vscode.window.createWebviewPanel(
    "chat-assistant-view",
    "Chat Assistant",
    vscode.ViewColumn.Beside,
    {
      enableScripts: true,
      retainContextWhenHidden: true,
      localResourceRoots: [webDist],
    }
  );
  panel.iconPath = vscode.Uri.joinPath(
    context.extensionUri,
    "icons",
    "stencila-128.png"
  );

  const viewCss = panel.webview.asWebviewUri(
    vscode.Uri.joinPath(webDist, "views", "vscode-chat.css")
  );
  const viewJs = panel.webview.asWebviewUri(
    vscode.Uri.joinPath(webDist, "views", "vscode-chat.js")
  );

  panel.webview.html = `
    <!DOCTYPE html>
    <html lang="en">
      <head>
          <title>Stencila: Document Preview</title>
          <meta charset="UTF-8">
          <meta name="viewport" content="width=device-width, initial-scale=1.0">
          <link rel="preconnect" href="https://fonts.googleapis.com">
          <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
          <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:ital,wght@0,100;0,200;0,300;0,400;0,500;0,600;0,700;1,100;1,200;1,300;1,400;1,500;1,600;1,700&family=Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900&display=swap" rel="stylesheet">
          <link rel="stylesheet" type="text/css" href="${viewCss}">
          <script type="text/javascript" src="${viewJs}"></script>
          <style>
            body, html {
              height: 100%;
              margin: 0;
              padding: 0;
              font-size: 16px;
            }
          </style>
      </head>
      <body>
        <stencila-vscode-chat-assistant-view>
          <p>I am a slotted element</p>
          <p>I am another slotted element</p>
        </stencila-vscode-chat-assistant-view>
        <script>
          const vscode = acquireVsCodeApi()
        </script>
      </body>
    </html>
  `;
}