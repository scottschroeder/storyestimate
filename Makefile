SHELL := /bin/bash

CARGO = cargo
CARGO_OPTS =

VERSION=$(shell grep -Em1 "^version" Cargo.toml | sed -r 's/.*"(.*)".*/\1/')
RUSTC_VERSION=$(shell rustc -V)
NAME := story-estimates
BUILD_DIR := ./build

ESTIMATE_BIN := ./target/release/estimate
BIN_DIR := /opt/storyestimates/bin

#.DEFAULT: config
.PHONY: all build clean deb version update_vagrant

all: deb

deb: $(BUILD_DIR)/$(NAME)_$(VERSION)_all.deb

version:
	echo "$(RUSTC_VERSION)" > RUSTC_VERSION

build:
	$(CARGO) $(CARGO_OPTS) build --release --features redis_estimates

clean:
	rm -fv $(BUILD_DIR)/$(NAME)_*_all.deb RUSTC_VERSION
	$(CARGO) $(CARGO_OPTS) clean

$(BUILD_DIR)/$(NAME)_$(VERSION)_all.deb: build version
	./fpm_build.sh $(VERSION)
