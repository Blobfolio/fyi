##
# Development Recipes
#
# This requires Just: https://github.com/casey/just
#
# To see possible tasks, run:
# just --list
##

cargo_dir     := "/tmp/fyi-cargo"
debian_dir    := "/tmp/fyi-release/fyi"
release_dir   := justfile_directory() + "/release"

build_ver     := "1"



# Build Release!
@bench:
	# First let's build the Rust bit.
	cd "{{ justfile_directory() }}/fyi_core" && cargo bench \
		--features progress,witcher \
		--target-dir "{{ cargo_dir }}"


# Build Release!
@build:
	# First let's build the Rust bit.
	RUSTFLAGS="-C link-arg=-s" cargo build \
		--release \
		--target-dir "{{ cargo_dir }}"


# Build Debian package!
@build-deb:
	[ $( command -v cargo-deb ) ] || cargo install cargo-deb

	# First let's build the Rust bit.
	RUSTFLAGS="-C link-arg=-s" cargo-deb \
		-p fyi \
		-o "{{ justfile_directory() }}/release"

	chown -R --reference="{{ justfile() }}" "{{ release_dir }}"


# Check Release!
@check:
	# First let's build the Rust bit.
	RUSTFLAGS="-C link-arg=-s" cargo check \
		--release \
		--target-dir "{{ cargo_dir }}"


# Get/Set FYI version.
version:
	#!/usr/bin/env bash

	# Current version.
	_ver1="$( cat "{{ justfile_directory() }}/fyi/Cargo.toml" | \
		grep version | \
		head -n 1 | \
		sed 's/[^0-9\.]//g' )"

	# Find out if we want to bump it.
	_ver2="$( whiptail --inputbox "Set FYI version:" --title "Release Version" 0 0 "$_ver1" 3>&1 1>&2 2>&3 )"

	exitstatus=$?
	if [ $exitstatus != 0 ] || [ "$_ver1" = "$_ver2" ]; then
		exit 0
	fi

	fyi success "Setting plugin version to $_ver2."

	# Set the release version!
	just _version "{{ justfile_directory() }}/fyi/Cargo.toml" "$_ver2" >/dev/null 2>&1
	just _version "{{ justfile_directory() }}/fyi_core/Cargo.toml" "$_ver2" >/dev/null 2>&1


# Truly set version.
_version TOML VER:
	#!/usr/bin/env php
	<?php
	if (! is_file("{{ TOML }}") || ! preg_match('/^\d+.\d+.\d+$/', "{{ VER }}")) {
		exit(1);
	}

	$content = file_get_contents("{{ TOML }}");
	$content = explode("\n", $content);
	$section = null;

	foreach ($content as $k=>$v) {
		if (\preg_match('/^\[[^\]]+\]$/', $v)) {
			$section = $v;
			continue;
		}
		elseif ('[package]' === $section && 0 === \strpos($v, 'version')) {
			$content[$k] = \sprintf(
				'version = "%s"',
				"{{ VER }}"
			);
			break;
		}
	}

	$content = implode("\n", $content);
	file_put_contents("{{ TOML }}", $content);


# Init dependencies.
@_init:
	[ ! -f "{{ justfile_directory() }}/Cargo.lock" ] || rm "{{ justfile_directory() }}/Cargo.lock"
	cargo update
