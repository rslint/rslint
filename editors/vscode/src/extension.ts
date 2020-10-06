import * as client from "./client";
import * as lspClient from "vscode-languageclient";
import * as vscode from "vscode";

let languageClient: lspClient.LanguageClient;

export function activate(context: vscode.ExtensionContext): Promise<void> {
  return client.launch(context).then((result) => {
    languageClient = result;
  });
}

export function deactivate(): Promise<void> | undefined {
  if (null == languageClient) {
    return undefined;
  }
  return languageClient.stop();
}
