SHELL := /bin/bash

CARGO = cargo
CARGO_OPTS =

VERSION=$(shell grep -Em1 "^version" Cargo.toml | sed -r 's/.*"(.*)".*/\1/')
NAME := story-estimates
BUILD_DIR := ./build

ESTIMATE_BIN := ./target/release/estimate
BIN_DIR := /opt/storyestimates/bin

#.DEFAULT: config
.PHONY: all build clean deb version update_vagrant

all: deb

deb: $(BUILD_DIR)/$(NAME)_$(VERSION)_all.deb

version:
	echo "$(VERSION)" > VERSION

build:
	$(CARGO) $(CARGO_OPTS) build --release

clean:
	rm -fv $(BUILD_DIR)/$(NAME)_*_all.deb
	$(CARGO) $(CARGO_OPTS) clean
	rm -rfv $(STAGING_DIR)

$(BUILD_DIR)/$(NAME)_$(VERSION)_all.deb: build
	./fpm_build.sh $(VERSION)
