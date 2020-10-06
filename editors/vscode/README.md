<div align="center">
  <h1>vscode-rslint</h1>
  <strong>Visual Studio Code client for the RSLint Language Server</strong>
</div>

### Installing the Client Extension

First ensure that you have the [node toolchain](https://nodejs.org/en/download/) installed, then proceed as follows:

```bash
npm i -g vsce
npm i
vsce package
```

This will produce a `vscode-rslint-<version>.vsix` file in the project root.

Next, open Code and show the command palette (`CTRL+SHIFT+P` or `CMD+SHIFT+P`) and type `install`, then select `Extensions: Install from VSIX...` from the list. Point the file selector to the previously generated `vscode-rslint-<version>.vsix`. Finally, hit the `reload` button when prompted.

### Testing the Client Extension

Ensure you have Node installed as above, then just open this `vscode` directory in a new VS Code window and hit F5 to start the extension host.
