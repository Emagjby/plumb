# Makefile for plumb
# 
# Usage:
#   make                Show this help
#   make build          Build plumb (release)
#   make install        Install to ~/.local/bin and clean build artifacts
#   make i              alias for install
#   make uninstall      Remove ~/.local/bin/plumb
#   make clean          cargo clean in ./plumb
#   make check          cargo check in ./plumb
#   make fmt            cargo fmt in ./plumb
#   make clippy         cargo clippy in ./plumb
#   make test           cargo test in ./plumb


BIN      := plumb
PROFILE  := release

PREFIX   ?= $(HOME)/.local
BINDIR   ?= $(PREFIX)/bin
INSTALL  ?= install
STRIP    ?= strip

PLUMB_DIR := plumb

.DEFAULT_GOAL := help

.PHONY: help build install uninstall clean check fmt clippy test i

help:
	@sed -n '1,/^$$/p' $(MAKEFILE_LIST)

build:
	@cd "$(PLUMB_DIR)" && cargo build --$(PROFILE)

install: build
	@mkdir -p "$(BINDIR)"
	@cd "$(PLUMB_DIR)" && \
		"$(INSTALL)" -m 0755 "target/$(PROFILE)/$(BIN)" "$(BINDIR)/$(BIN)"
	@command -v "$(STRIP)" >/dev/null 2>&1 && "$(STRIP)" "$(BINDIR)/$(BIN)" || true
	@cd "$(PLUMB_DIR)" && cargo clean
	@echo "Installed $(BIN) -> $(BINDIR)/$(BIN)"

uninstall:
	@rm -f "$(BINDIR)/$(BIN)"
	@echo "Removed $(BINDIR)/$(BIN)"

clean:
	@cd "$(PLUMB_DIR)" && cargo clean

check:
	@cd "$(PLUMB_DIR)" && cargo check

fmt:
	@cd "$(PLUMB_DIR)" && cargo fmt

clippy:
	@cd "$(PLUMB_DIR)" && cargo clippy -- -D warnings

test:
	@cd "$(PLUMB_DIR)" && cargo test

i: install
