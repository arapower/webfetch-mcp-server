# webfetch-mcp-server

## Overview

A Rust-based MCP server that provides a `fetch` tool for retrieving the content of a web page by URL via the MCP protocol.

## Features

- Simple web page downloader

## Getting Started / Usage

Create `.devcontainer/devcontainer.json` as follows:

### devcontainer.json

This is a minimal devcontainer configuration required to use this repository as an MCP server.  
Place the following file at `.devcontainer/devcontainer.json` in your project.

> **Note:**  
> This is a sample configuration.  
> The `cargo` command is required for installation and is available in the specified devcontainer image.  
> This configuration allows you to quickly set up a development environment for the webfetch-mcp-server using Visual Studio Code and the Dev Containers extension.  
> After opening your project in VS Code, select **Dev Containers: Reopen in Container** from the Command Palette to start developing inside the container.

```json
{
    "image": "mcr.microsoft.com/devcontainers/rust:1-bullseye",
    "postCreateCommand": "cargo install --git https://github.com/arapower/webfetch-mcp-server.git"
}
```

### MCP Client Settings in VSCode

You can configure the MCP client in VSCode by creating `.vscode/mcp.json`.  
Below are sample configurations for each `type`.

#### For `type: "stdio"`

```json
{
  "servers": {
    "webfetch": {
      "type": "stdio",
      "command": "webfetch"
    }
  }
}
```
- `webfetch` runs as an MCP server using standard input/output.

#### For `type: "http"`

```json
{
  "servers": {
    "webfetch-api": {
      "type": "http",
      "url": "http://host.docker.internal:8040/mcp"
    }
  }
}
```
- `webfetch-api` connects to the MCP server via HTTP at the specified URL.

## Example Usage of the fetch Tool

From an MCP client, send a request like the following:

```json
{
  "url": "https://example.com"
}
```
You will receive the content of the web page.

## Build Errors

Depending on your build environment, you may encounter errors if the OpenSSL development package is missing.  
This error occurs because the Rust dependency crate `openssl-sys` cannot find the OpenSSL development libraries (such as header files and .pc files) on your system.  
You can resolve this by installing the OpenSSL development package using the following commands:

- **Ubuntu/Debian:**
  ```sh
  sudo apt-get update
  sudo apt-get install pkg-config libssl-dev
  ```

- **Fedora/RHEL:**
  ```sh
  sudo dnf install pkg-config openssl-devel
  ```

- **Arch Linux:**
  ```sh
  sudo pacman -S pkgconf openssl
  ```

## Troubleshooting / FAQ

- **Q:** `webfetch` command not found  
  **A:** Please run `cargo install --git https://github.com/arapower/webfetch-mcp-server.git` inside the Dev Container.

- **Q:** Cannot connect from MCP client  
  **A:** Check if the `command` setting in `mcp.json` is correct and ensure the server is running.

## License

MIT

## Acknowledgments & References

- [modelcontextprotocol/rust-sdk examples/servers](https://github.com/modelcontextprotocol/rust-sdk/tree/main/examples/servers)
