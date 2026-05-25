#!/usr/bin/env bash
set -euo pipefail

if ! command -v uv >/dev/null 2>&1; then
  curl -LsSf https://astral.sh/uv/install.sh | sh
fi

export PATH="$HOME/.local/bin:$PATH"

if ! command -v serena >/dev/null 2>&1; then
  uv tool install -p 3.13 serena-agent@latest --prerelease=allow
fi

if [[ ! -f "$HOME/.serena/serena_config.yml" ]]; then
  serena init
fi

if [[ ! -f ".serena/project.yml" ]]; then
  serena project create "$PWD" \
    --name "rust-ownership-error-navigation" \
    --language typescript \
    --language json \
    --language markdown \
    --language yaml
fi

serena --version