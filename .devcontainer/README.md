# RSLint VSCode Dev Container

This directory configures a [VSCode Dev Container](https://code.visualstudio.com/docs/devcontainers/containers)
in order to provide a rust toolchain and all necessary tooling to develop `rslint`.

To use it you'll need

* [Visual Studio Code](https://code.visualstudio.com)
* [Docker](https://www.docker.com) (running)
* [Dev Containers extension](vscode:extension/ms-vscode-remote.remote-containers)

Once running you can reload your project and it should detect the `.devcontainer` directory
configuration and offer to reload the project within the container.