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
pkg_dir1    := justfile_directory() + "/fyi"
pkg_dir2    := justfile_directory() + "/fyi_msg"
pkg_dir3    := justfile_directory() + "/fyi_progress"
pkg_dir4    := justfile_directory() + "/fyi_witcher"

cargo_dir   := "/tmp/" + pkg_id + "-cargo"
cargo_bin   := cargo_dir + "/x86_64-unknown-linux-gnu/release/" + pkg_id
pgo_dir     := "/tmp/pgo-data"
release_dir := justfile_directory() + "/release"

rustflags   := "-C llvm-args=--vectorize-slp -Clinker-plugin-lto -Clinker=clang-9 -Clink-args=-fuse-ld=lld-9 -C link-arg=-s"



# Bench it!
bench BENCH="" FILTER="":
	#!/usr/bin/env bash

	clear

	if [ -z "{{ BENCH }}" ]; then
		cargo bench \
			-q \
			--workspace \
			--all-features \
			--target x86_64-unknown-linux-gnu \
			--target-dir "{{ cargo_dir }}" -- "{{ FILTER }}"
	else
		cargo bench \
			-q \
			--bench "{{ BENCH }}" \
			--workspace \
			--all-features \
			--target x86_64-unknown-linux-gnu \
			--target-dir "{{ cargo_dir }}" -- "{{ FILTER }}"
	fi
	exit 0


# Build Release!
@build:
	# First let's build the Rust bit.
	RUSTFLAGS="{{ rustflags }}" cargo build \
		--bin "{{ pkg_id }}" \
		--release \
		--target x86_64-unknown-linux-gnu \
		--target-dir "{{ cargo_dir }}"


# Build Debian package!
@build-deb: build-man
	# cargo-deb doesn't support target_dir flags yet.
	[ ! -d "{{ justfile_directory() }}/target" ] || rm -rf "{{ justfile_directory() }}/target"
	mv "{{ cargo_dir }}" "{{ justfile_directory() }}/target"

	# First let's build the Rust bit.
	cargo-deb \
		--no-build \
		-p {{ pkg_id }} \
		-o "{{ justfile_directory() }}/release"

	just _fix-chown "{{ release_dir }}"
	mv "{{ justfile_directory() }}/target" "{{ cargo_dir }}"


# Build Man.
@build-man: build-pgo
	# Pre-clean.
	find "{{ release_dir }}/man" -type f -delete

	# Use help2man to make a crappy MAN page.
	help2man -o "{{ release_dir }}/man/{{ pkg_id }}.1" \
		-N "{{ cargo_bin }}"

	# Strip some ugly out.
	sd '{{ pkg_name }} [0-9.]+\nBlobfolio, LLC. <hello@blobfolio.com>\n' \
		'' \
		"{{ release_dir }}/man/{{ pkg_id }}.1"

	# Gzip it and reset ownership.
	gzip -k -f -9 "{{ release_dir }}/man/{{ pkg_id }}.1"
	just _fix-chown "{{ release_dir }}/man"


# Build PGO.
@build-pgo: clean
	# First let's build the Rust bit.
	RUSTFLAGS="{{ rustflags }} -Cprofile-generate={{ pgo_dir }}" \
		cargo build \
			--bin "{{ pkg_id }}" \
			--release \
			--target x86_64-unknown-linux-gnu \
			--target-dir "{{ cargo_dir }}"

	clear

	# Instrumentation!
	"{{ cargo_bin }}" print "This is a message."
	"{{ cargo_bin }}" print -p "Prefix" "This is a message."
	"{{ cargo_bin }}" print -p "Prefix" -c 10 "This is a message."
	"{{ cargo_bin }}" print -t "This is a message."
	"{{ cargo_bin }}" print -i "This is a message."
	"{{ cargo_bin }}" print --stderr "This is a message."

	"{{ cargo_bin }}" crunched "This is a crunch method."
	"{{ cargo_bin }}" debug "This is a debug method."
	"{{ cargo_bin }}" done "This is a done method."
	"{{ cargo_bin }}" error "This is a error method."
	"{{ cargo_bin }}" info "This is a info method."
	"{{ cargo_bin }}" notice "This is a notice method."
	"{{ cargo_bin }}" success "This is a success method."
	"{{ cargo_bin }}" task "This is a task method."
	"{{ cargo_bin }}" warning "This is a warning method."

	clear
	"{{ cargo_bin }}" debug "Fine-tune the prompt:"
	"{{ cargo_bin }}" confirm "Type 'Y':" || true
	"{{ cargo_bin }}" confirm "Type 'e' and then 'n':" || true
	"{{ cargo_bin }}" confirm "Just hit <ENTER>:" || true
	"{{ cargo_bin }}" blank
	"{{ cargo_bin }}" blank -c 1
	"{{ cargo_bin }}" blank -c 2
	"{{ cargo_bin }}" print -t "This is a message."
	"{{ cargo_bin }}" -h
	"{{ cargo_bin }}" -V
	"{{ cargo_bin }}" badanswer || true
	clear

	# Merge the data back in.
	llvm-profdata-9 \
		merge -o "{{ pgo_dir }}/merged.profdata" "{{ pgo_dir }}"

	RUSTFLAGS="{{ rustflags }} -Cprofile-use={{ pgo_dir }}/merged.profdata" \
		cargo build \
			--bin "{{ pkg_id }}" \
			--release \
			--target x86_64-unknown-linux-gnu \
			--target-dir "{{ cargo_dir }}"


# Check Release!
@check:
	# First let's build the Rust bit.
	RUSTFLAGS="{{ rustflags }}" cargo check \
		--release \
		--all-features \
		--target x86_64-unknown-linux-gnu \
		--target-dir "{{ cargo_dir }}"


# Clean Cargo crap.
@clean:
	# Most things go here.
	[ ! -d "{{ cargo_dir }}" ] || rm -rf "{{ cargo_dir }}"
	[ ! -d "{{ pgo_dir }}" ] || rm -rf "{{ pgo_dir }}"

	# But some Cargo apps place shit in subdirectories even if
	# they place *other* shit in the designated target dir. Haha.
	[ ! -d "{{ justfile_directory() }}/target" ] || rm -rf "{{ justfile_directory() }}/target"
	[ ! -d "{{ pkg_dir1 }}/target" ] || rm -rf "{{ pkg_dir1 }}/target"
	[ ! -d "{{ pkg_dir2 }}/target" ] || rm -rf "{{ pkg_dir2 }}/target"
	[ ! -d "{{ pkg_dir3 }}/target" ] || rm -rf "{{ pkg_dir3 }}/target"
	[ ! -d "{{ pkg_dir4 }}/target" ] || rm -rf "{{ pkg_dir4 }}/target"


# Clippy.
@clippy:
	clear
	RUSTFLAGS="{{ rustflags }}" cargo clippy \
		--workspace \
		--release \
		--all-features \
		--target x86_64-unknown-linux-gnu \
		--target-dir "{{ cargo_dir }}"


# Example: Progress Bar
example-progress:
	#!/usr/bin/env bash

	clear

	cargo run \
		-q \
		-p fyi_progress \
		--example progress \
		--target x86_64-unknown-linux-gnu \
		--target-dir "{{ cargo_dir }}"


# Example: Witcher Find/Progress
example-witcher:
	#!/usr/bin/env bash

	clear

	cargo run \
		-q \
		-p fyi_witcher \
		--example witcher \
		--target x86_64-unknown-linux-gnu \
		--target-dir "{{ cargo_dir }}"


# Unit tests!
@test:
	clear
	RUST_TEST_THREADS=1 cargo test \
		--tests \
		--all-features \
		--release \
		--workspace \
		--target x86_64-unknown-linux-gnu \
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
	just _version "{{ pkg_dir1 }}" "$_ver2"
	just _version "{{ pkg_dir2 }}" "$_ver2"
	just _version "{{ pkg_dir3 }}" "$_ver2"
	just _version "{{ pkg_dir4 }}" "$_ver2"


# Set version for real.
@_version DIR VER:
	[ -f "{{ DIR }}/Cargo.toml" ] || exit 1

	# Set the release version!
	toml set "{{ DIR }}/Cargo.toml" package.version "{{ VER }}" > /tmp/Cargo.toml
	just _fix-chown "/tmp/Cargo.toml"
	mv "/tmp/Cargo.toml" "{{ DIR }}/Cargo.toml"


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
