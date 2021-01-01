import * as lspClient from "vscode-languageclient";
import * as vscode from "vscode";

export async function launch(context: vscode.ExtensionContext): Promise<lspClient.LanguageClient> {
  const run: lspClient.Executable = {
    command: "rslint-lsp",
  };
  const debug: lspClient.Executable = {
    command: "rslint-lsp",
    options: {
      env: {
        RUST_BACKTRACE: 1,
        RUST_LOG: "info",
        ...process.env,
      },
    },
  };
  const serverOptions: lspClient.ServerOptions = { debug, run };
  const clientOptions: lspClient.LanguageClientOptions = {
    diagnosticCollectionName: "rslint-lsp",
    documentSelector: [
      { language: "javascript", scheme: "file" },
      { language: "javascript", scheme: "untitled" },
      { language: "toml", scheme: "file" },
      { language: "toml", scheme: "untitled" },
      { language: "typescript", scheme: "file" },
      { language: "typescript", scheme: "untitled" },
    ],
    synchronize: {
      fileEvents: [
        vscode.workspace.createFileSystemWatcher("**/*.js"),
        vscode.workspace.createFileSystemWatcher("**/*.mjs"),
        vscode.workspace.createFileSystemWatcher("**/*.ts"),
        vscode.workspace.createFileSystemWatcher("**/rslintrc.toml"),
      ],
    },
    middleware: {} as lspClient.Middleware,
  };
  const languageClient = new lspClient.LanguageClient(
    "rslint-lsp",
    "RSLint Language Server",
    serverOptions,
    clientOptions,
  );
  const session = languageClient.start();
  context.subscriptions.push(session);
  await languageClient.onReady();
  return languageClient;
}
