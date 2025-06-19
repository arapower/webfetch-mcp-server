このエラーは、Rustの依存クレート `openssl-sys` が**システムにOpenSSLの開発用ライブラリ（ヘッダファイルや.pcファイル）を見つけられない**ために発生しています[2][6]。

---

## ビルドエラー

ビルド環境によってはOpenSSL開発パッケージがないためにエラーとなる可能性があります。
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

