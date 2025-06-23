# webfetch-mcp-server

## Overview
Rust製のMCPサーバーで、MCPプロトコル経由でURLを指定してWebページの内容を取得する`fetch`ツールを提供します。

## Features
- シンプルなWebページダウンローダ

## Getting Started / Usage
`.devcontainer/devcontainer.json` を以下のように作成します。

## devcontainer.json

インストールには`cargo`コマンドが必要です。

```jsonc
{
    "image": "mcr.microsoft.com/devcontainers/rust:1-bullseye",
    "postCreateCommand": "cargo install --git https://github.com/arapower/webfetch-mcp-server.git"
}
```

### VSCodeでのMCPクライアント設定例
`.vscode/mcp.json` を以下のように作成します。
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
- `webfetch`は標準入出力でMCPサーバとして動作します。

### fetchツールの利用例
MCPクライアントから
```json
{
  "url": "https://example.com"
}
```
のようなリクエストを送信すると、Webページの内容が取得できます。

## ビルドエラー

ビルド環境によってはOpenSSL開発パッケージがないためにエラーとなる可能性があります。
このエラーは、Rustの依存クレート `openssl-sys` がシステムにOpenSSLの開発用ライブラリ（ヘッダファイルや.pcファイル）を見つけられないために発生しています。
以下のコマンドでOpenSSL開発パッケージをインストールすると解消されます。

- **Ubuntu/Debian系:**
  ```sh
  sudo apt-get update
  sudo apt-get install pkg-config libssl-dev
  ```

- **Fedora/RHEL系:**
  ```sh
  sudo dnf install pkg-config openssl-devel
  ```

- **Arch Linux:**
  ```sh
  sudo pacman -S pkgconf openssl
  ```

## Troubleshooting / FAQ

- **Q:** `webfetch`コマンドが見つからない  
  **A:** Dev Container内で`cargo install --git https://github.com/arapower/webfetch-mcp-server.git`を実行してください。

- **Q:** MCPクライアントから接続できない  
  **A:** `mcp.json`の`command`設定が正しいか、サーバが起動しているか確認してください。

## License
MIT

## Acknowledgments & References
- [modelcontextprotocol/rust-sdk examples/servers](https://github.com/modelcontextprotocol/rust-sdk/tree/main/examples/servers)
