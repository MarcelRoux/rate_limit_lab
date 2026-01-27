#!/usr/bin/env sh
set -eu

echo "🔧 Installing git hooks..."

git config core.hooksPath githooks

chmod +x githooks/pre-commit
chmod +x githooks/pre-push
chmod +x githooks/commit-msg

echo "✅ Git hooks installed successfully."
