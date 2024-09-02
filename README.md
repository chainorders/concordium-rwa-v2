# Real World Asset (RWA) for Concordium

## Repository Setup

```bash
git clone git@github.com:chainorders/concordium-rwa.git
git submodule update --init --recursive
```

## Development Environment

- Download & Install the [VS code Development env extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers). Please follow through the prerequisites like docker needed by the extension.
- Deploying contract requires access to a testnet wallet.
  Export your wallet and copy it to the location [`.devcontainer/contracts/default_account.export`](.devcontainer/contracts/default_account.export).
  Or change the path in [contracts container file](.devcontainer/contracts/devcontainer.json)

### Environments

- [Contracts](./contracts/README.md) : For debugging and deploying contracts to the testnet
- [Backend](./backend/README.md) : For debugging and deploying backend services
- [Frontend](./frontend/README.md) : FOr debugging and deploying frontend

After installing the dev container extension. Execute the command `Dev Containers: Reopen in a Container`.
Which will present a list of mentioned development environments.

**Upon start of the container available yarn scripts will be shown on the terminal.**
