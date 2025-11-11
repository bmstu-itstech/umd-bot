#!/usr/bin/env bash

SCRIPTS_DIR="scripts"

PREV_LINE="\033[A"
RED="\033[0;31m"
GREEN="\033[0;32m"
NC="\033[0m"

error() {
	echo -e "${RED}error${NC}:" "$@" >&2
	exit 1
}

echo "Running pre-commit checks for project"

# Shell scripts formatting
echo "Running 'shfmt' for shell scripts..."
for script in "$SCRIPTS_DIR"/*.sh; do
	shfmt -w "$script"
done
echo -e "${PREV_LINE}Running 'shfmt' for shell scripts... ${GREEN}OK${NC}"

# Shell check
echo "Running 'shellcheck' for shell scripts..."
for script in "$SCRIPTS_DIR"/*.sh; do
	shellcheck "$script"
done
echo -e "${PREV_LINE}Running 'shellcheck' for shell scripts... ${GREEN}OK${NC}"

# Rust formatting
echo "Running cargo fmt check..."
cargo fmt -- --check || error "code formatting check failed. Run 'cargo fmt' to fix"
echo -e "${PREV_LINE}Running cargo fmt check... ${GREEN}OK${NC}"

# Rust linter
echo "Running clippy..."
cargo clippy -q -- -D warnings || error "clippy found issues, fix them before commiting"
echo -e "${PREV_LINE}Running clippy... ${GREEN}OK${NC}"

echo -e "${GREEN}All checks passed${NC}"
