#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR/ui"

VERSION="$(node -p \"require('./package.json').version\")"
GIT_SHA="$(git rev-parse --short HEAD)"

export VITE_SENTRY_RELEASE="int3rceptor-ui@${VERSION}+${GIT_SHA}"
export SENTRY_RELEASE="$VITE_SENTRY_RELEASE"
export VITE_SENTRY_ENVIRONMENT="${VITE_SENTRY_ENVIRONMENT:-development}"
export SENTRY_PROJECT="${SENTRY_PROJECT:-int3rceptor-ui}"

: \"${SENTRY_AUTH_TOKEN:?SENTRY_AUTH_TOKEN is required}\"
: \"${SENTRY_ORG:?SENTRY_ORG is required}\"
: \"${VITE_SENTRY_DSN:?VITE_SENTRY_DSN is required}\"

echo \"Building UI with release=$VITE_SENTRY_RELEASE (env=$VITE_SENTRY_ENVIRONMENT)\"
npm install --no-progress
npm run build

echo \"Done. Sourcemaps uploaded if the Sentry env vars were present.\"
