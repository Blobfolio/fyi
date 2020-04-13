##
# Development Recipes
#
# This requires Just: https://github.com/casey/just
#
# To see possible tasks, run:
# just --list
##

cargo_dir     := "/tmp/fyi-cargo"
release_dir   := justfile_directory() + "/release"



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
@build-deb: build-man
	[ $( command -v cargo-deb ) ] || cargo install cargo-deb

	# cargo-deb doesn't support target_dir flags yet.
	[ ! -d "{{ justfile_directory() }}/target" ] || rm -rf "{{ justfile_directory() }}/target"
	mv "{{ cargo_dir }}" "{{ justfile_directory() }}/target"

	# First let's build the Rust bit.
	RUSTFLAGS="-C link-arg=-s" cargo-deb \
		-p fyi \
		-o "{{ justfile_directory() }}/release"

	just _fix-chown "{{ release_dir }}"
	mv "{{ justfile_directory() }}/target" "{{ cargo_dir }}"


# Build Man.
@build-man: build
	# Pre-clean.
	rm "{{ release_dir }}/man"/*

	# Use help2man to make a crappy MAN page.
	help2man -o "{{ release_dir }}/man/fyi.1" -N "{{ cargo_dir }}/release/fyi"

	# Strip some ugly out.
	sd 'FYI [0-9.]+\nBlobfolio, LLC. <hello@blobfolio.com>\n' '' "{{ release_dir }}/man/fyi.1"

	# Gzip it and reset ownership.
	gzip -k -f -9 "{{ release_dir }}/man/fyi.1"
	just _fix-chown "{{ release_dir }}/man"


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
	_ver1="$( toml get "{{ justfile_directory() }}/fyi_core/Cargo.toml" package.version | \
		sed 's/"//g' )"

	# Find out if we want to bump it.
	_ver2="$( whiptail --inputbox "Set FYI version:" --title "Release Version" 0 0 "$_ver1" 3>&1 1>&2 2>&3 )"

	exitstatus=$?
	if [ $exitstatus != 0 ] || [ "$_ver1" = "$_ver2" ]; then
		exit 0
	fi

	fyi success "Setting plugin version to $_ver2."

	# Set the release version!
	toml set "{{ justfile_directory() }}/fyi_core/Cargo.toml" \
		package.version \
		"$_ver2" > /tmp/Cargo.toml
	mv "/tmp/Cargo.toml" "{{ justfile_directory() }}/fyi_core/Cargo.toml"
	just _fix-chown "{{ justfile_directory() }}/fyi_core/Cargo.toml"

	toml set "{{ justfile_directory() }}/fyi/Cargo.toml" \
		package.version \
		"$_ver2" > /tmp/Cargo.toml
	mv "/tmp/Cargo.toml" "{{ justfile_directory() }}/fyi/Cargo.toml"
	just _fix-chown "{{ justfile_directory() }}/fyi/Cargo.toml"


# Init dependencies.
@_init:
	[ ! -f "{{ justfile_directory() }}/Cargo.lock" ] || rm "{{ justfile_directory() }}/Cargo.lock"
	cargo update

# Fix file/directory permissions.
@_fix-chmod PATH:
	[ ! -e "{{ PATH }}" ] || find "{{ PATH }}" -type f -exec chmod 0644 {} +
	[ ! -e "{{ PATH }}" ] || find "{{ PATH }}" -type d -exec chmod 0755 {} +


# Fix file/directory ownership.
@_fix-chown PATH:
	[ ! -e "{{ PATH }}" ] || chown -R --reference="{{ justfile() }}" "{{ PATH }}"
