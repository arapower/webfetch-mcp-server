#!/bin/bash
set -eu

Usage() {
    cat >&2 <<EOS
Usage: $0 <ORG_NAME> <VERSION> <TAG>
  例: $0 arapower 0.2.2-alpha.4 v0.2.2-alpha.4
EOS
}

if [ "$#" -lt 3 ]; then
  Usage
  exit 1
fi

ORG_NAME="$1"
VERSION="$2"
TAG="$3"
REPO_URL="https://github.com/${ORG_NAME}/app_name.git"
FORMULA_VER_PATH="homebrew-tap/Formula/app_name@${VERSION}.rb"
FORMULA_MAIN_PATH="homebrew-tap/Formula/app_name.rb"

# Generate class name from version（e.g.: 0.2.2-alpha.4 → AppNameAT022Alpha4）
CLASS_VERSION=$(echo "$VERSION" \
    | sed 's/-\([a-z]\)/-\U\1/g' \
    | sed 's/^/AT/' \
    | tr -d '.-' )
CLASS_NAME="AppName${CLASS_VERSION}"

cat > "$FORMULA_VER_PATH" <<EOF
class $CLASS_NAME < Formula
  desc "App Description"
  homepage "https://github.com/${ORG_NAME}/app_name"
  url "$REPO_URL", tag: "$TAG"
  version "$VERSION"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--root", prefix, "--path", "."
  end

  test do
    system "#{bin}/app_name", "--help"
  end
end
EOF

case "$VERSION" in
  *-*)
    # None
    ;;
  *)
    # Stable version: Generate main formula
    sed "s/class $CLASS_NAME/class AppName/" "$FORMULA_VER_PATH" > "$FORMULA_MAIN_PATH"
    ;;
esac

# Commit and push changes
cd homebrew-tap
git config user.name "GitHub Actions"
git config user.email "actions@github.com"
git add Formula/*.rb
git commit -m "Update app_name to $VERSION"
git push
