##
# Development Recipes
#
# This requires Just: https://github.com/casey/just
#
# To see possible tasks, run:
# just --list
##

pkg_id      := "fyi"
pkg_name    := "FYI"
pkg_dir1    := justfile_directory() + "/fyi_core"
pkg_dir2    := justfile_directory() + "/fyi"

cargo_dir   := "/tmp/" + pkg_id + "-cargo"
release_dir := justfile_directory() + "/release"



# Build Release!
bench BENCH="" FILTER="":
	#!/usr/bin/env bash

	clear

	if [ -z "{{ BENCH }}" ]; then
		cargo bench \
			-q \
			--workspace \
			--all-features \
			--target-dir "{{ cargo_dir }}" -- "{{ FILTER }}"
	else
		cargo bench \
			-q \
			--bench "{{ BENCH }}" \
			--workspace \
			--all-features \
			--target-dir "{{ cargo_dir }}" -- "{{ FILTER }}"
	fi
	exit 0


# Build Release!
@build:
	# First let's build the Rust bit.
	RUSTFLAGS="-C link-arg=-s" cargo build \
		--bin fyi \
		--release \
		--target-dir "{{ cargo_dir }}"


# Build Debian package!
@build-deb: build-man
	# cargo-deb doesn't support target_dir flags yet.
	[ ! -d "{{ justfile_directory() }}/target" ] || rm -rf "{{ justfile_directory() }}/target"
	mv "{{ cargo_dir }}" "{{ justfile_directory() }}/target"

	# First let's build the Rust bit.
	RUSTFLAGS="-C link-arg=-s" cargo-deb \
		-p {{ pkg_id }} \
		-o "{{ justfile_directory() }}/release"

	just _fix-chown "{{ release_dir }}"
	mv "{{ justfile_directory() }}/target" "{{ cargo_dir }}"


# Build Man.
@build-man: build
	# Pre-clean.
	rm "{{ release_dir }}/man"/*

	# Use help2man to make a crappy MAN page.
	help2man -o "{{ release_dir }}/man/{{ pkg_id }}.1" \
		-N "{{ cargo_dir }}/release/{{ pkg_id }}"

	# Strip some ugly out.
	sd '{{ pkg_name }} [0-9.]+\nBlobfolio, LLC. <hello@blobfolio.com>\n' \
		'' \
		"{{ release_dir }}/man/{{ pkg_id }}.1"

	# Gzip it and reset ownership.
	gzip -k -f -9 "{{ release_dir }}/man/{{ pkg_id }}.1"
	just _fix-chown "{{ release_dir }}/man"


# Check Release!
@check:
	# First let's build the Rust bit.
	RUSTFLAGS="-C link-arg=-s" cargo check \
		--release \
		--all-features \
		--target-dir "{{ cargo_dir }}"


# Clean Cargo crap.
@clean:
	# Most things go here.
	[ ! -d "{{ cargo_dir }}" ] || rm -rf "{{ cargo_dir }}"

	# But some Cargo apps place shit in subdirectories even if
	# they place *other* shit in the designated target dir. Haha.
	[ ! -d "{{ justfile_directory() }}/target" ] || rm -rf "{{ justfile_directory() }}/target"
	[ ! -d "{{ justfile_directory() }}/fyi/target" ] || rm -rf "{{ justfile_directory() }}/fyi/target"
	[ ! -d "{{ justfile_directory() }}/fyi_core/target" ] || rm -rf "{{ justfile_directory() }}/fyi_core/target"
	[ ! -d "{{ justfile_directory() }}/fyi_witch/target" ] || rm -rf "{{ justfile_directory() }}/fyi_witch/target"


# Build Release!
demo-progress:
	#!/usr/bin/env bash

	clear

	cargo run \
		-q \
		-p fyi_core \
		--example progress \
		--all-features \
		--target-dir "{{ cargo_dir }}"


# Unit tests!
@test:
	RUST_TEST_THREADS=1 cargo test \
		--tests \
		--all-features \
		--release \
		--workspace \
		--target-dir "{{ cargo_dir }}" -- \
			--format terse \


# Get/Set version.
version:
	#!/usr/bin/env bash

	# Current version.
	_ver1="$( toml get "{{ pkg_dir1 }}/Cargo.toml" package.version | \
		sed 's/"//g' )"

	# Find out if we want to bump it.
	_ver2="$( whiptail --inputbox "Set {{ pkg_name }} version:" --title "Release Version" 0 0 "$_ver1" 3>&1 1>&2 2>&3 )"

	exitstatus=$?
	if [ $exitstatus != 0 ] || [ "$_ver1" = "$_ver2" ]; then
		exit 0
	fi

	fyi success "Setting version to $_ver2."

	# Set the release version!
	toml set "{{ pkg_dir1 }}/Cargo.toml" \
		package.version \
		"$_ver2" > /tmp/Cargo.toml
	mv "/tmp/Cargo.toml" "{{ pkg_dir1 }}/Cargo.toml"
	just _fix-chown "{{ pkg_dir1 }}/Cargo.toml"

	toml set "{{ pkg_dir2 }}/Cargo.toml" \
		package.version \
		"$_ver2" > /tmp/Cargo.toml
	mv "/tmp/Cargo.toml" "{{ pkg_dir2 }}/Cargo.toml"
	just _fix-chown "{{ pkg_dir2 }}/Cargo.toml"


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
