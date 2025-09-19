##
# Development Recipes
#
# This justfile is intended to be run from inside a Docker sandbox:
# https://github.com/Blobfolio/righteous-sandbox
#
# docker run \
#	--rm \
#	-v "{{ invocation_directory() }}":/share \
#	-it \
#	--name "righteous_sandbox" \
#	"righteous/sandbox:debian"
#
# Alternatively, you can just run cargo commands the usual way and ignore these
# recipes.
##

pkg_id      := "fyi"
pkg_name    := "FYI"
pkg_dir1    := justfile_directory() + "/fyi"
pkg_dir2    := justfile_directory() + "/fyi_ansi"
pkg_dir3    := justfile_directory() + "/fyi_msg"

cargo_dir   := "/tmp/" + pkg_id + "-cargo"
cargo_bin   := cargo_dir + "/release/" + pkg_id
doc_dir     := justfile_directory() + "/doc"
release_dir := justfile_directory() + "/release"

export RUSTFLAGS := "-Ctarget-cpu=x86-64-v3 -Cllvm-args=--cost-kind=throughput -Clinker-plugin-lto -Clink-arg=-fuse-ld=lld"
export CC        := "clang"
export CXX       := "clang++"
export CFLAGS    := `llvm-config --cflags` + " -march=x86-64-v3 -Wall -Wextra -flto"
export CXXFLAGS  := `llvm-config --cxxflags` + " -march=x86-64-v3 -Wall -Wextra -flto"
export LDFLAGS   := `llvm-config --ldflags` + " -fuse-ld=lld -flto"



# Bench it!
bench BENCH="":
	#!/usr/bin/env bash

	clear
	if [ -z "{{ BENCH }}" ]; then
		cargo bench \
			--benches \
			--workspace \
			--all-features \
			--target-dir "{{ cargo_dir }}"
	else
		cargo bench \
			--bench "{{ BENCH }}" \
			--workspace \
			--all-features \
			--target-dir "{{ cargo_dir }}"
	fi
	exit 0


# Bin Test!
@bin-test:
	[ -f "{{ cargo_bin }}" ] || just build

	"{{ cargo_bin }}" print "This message has no prefix."

	"{{ cargo_bin }}" blank

	"{{ cargo_bin }}" print -p "Pink" -c 199 "This message has a custom pink prefix."
	"{{ cargo_bin }}" print -p "Blue" -c 4 "This message has a custom blue prefix."

	"{{ cargo_bin }}" blank

	"{{ cargo_bin }}" aborted "Built-in prefix."
	"{{ cargo_bin }}" crunched "Built-in prefix."
	"{{ cargo_bin }}" debug "Built-in prefix."
	"{{ cargo_bin }}" done "Built-in prefix."
	"{{ cargo_bin }}" error "Built-in prefix."
	"{{ cargo_bin }}" found "Built-in prefix."
	"{{ cargo_bin }}" info "Built-in prefix."
	"{{ cargo_bin }}" notice "Built-in prefix."
	"{{ cargo_bin }}" review "Built-in prefix."
	"{{ cargo_bin }}" skipped "Built-in prefix."
	"{{ cargo_bin }}" success "Built-in prefix."
	"{{ cargo_bin }}" task "Built-in prefix."
	"{{ cargo_bin }}" warning "Built-in prefix."

	"{{ cargo_bin }}" blank

	"{{ cargo_bin }}" info -t "Messages can be timestamped."

	"{{ cargo_bin }}" blank

	"{{ cargo_bin }}" info "Messages can be indented."
	"{{ cargo_bin }}" info -i "Messages can be indented."

	"{{ cargo_bin }}" blank

	"{{ cargo_bin }}" confirm "Does this default no?" || "{{ cargo_bin }}" error "Well that sucks."
	"{{ cargo_bin }}" confirm -y "Did this work for you?" || "{{ cargo_bin }}" error "Well that sucks."

	"{{ cargo_bin }}" blank


# Build Release!
@build:
	# First let's build the Rust bit.
	cargo build \
		--bin "{{ pkg_id }}" \
		--release \
		--target-dir "{{ cargo_dir }}"

	# Fix ownership, etc.
	just _fix-chmod "{{ pkg_dir1 }}"
	just _fix-chown "{{ pkg_dir1 }}"


# Build Debian package!
@build-deb: clean credits build
	# cargo-deb doesn't support target_dir flags yet.
	[ ! -d "{{ justfile_directory() }}/target" ] || rm -rf "{{ justfile_directory() }}/target"
	mv "{{ cargo_dir }}" "{{ justfile_directory() }}/target"

	# Build the deb.
	cargo-deb \
		--no-build \
		--quiet \
		-p {{ pkg_id }} \
		-o "{{ release_dir }}"

	just _fix-chown "{{ release_dir }}"
	mv "{{ justfile_directory() }}/target" "{{ cargo_dir }}"


# Clean Cargo crap.
@clean:
	# Most things go here.
	[ ! -d "{{ cargo_dir }}" ] || rm -rf "{{ cargo_dir }}"

	# But some Cargo apps place shit in subdirectories even if
	# they place *other* shit in the designated target dir. Haha.
	[ ! -d "{{ justfile_directory() }}/target" ] || rm -rf "{{ justfile_directory() }}/target"
	[ ! -d "{{ pkg_dir1 }}/target" ] || rm -rf "{{ pkg_dir1 }}/target"
	[ ! -d "{{ pkg_dir2 }}/target" ] || rm -rf "{{ pkg_dir2 }}/target"
	[ ! -d "{{ pkg_dir3 }}/target" ] || rm -rf "{{ pkg_dir3 }}/target"

	cargo update -w


# Clippy.
@clippy:
	clear

	fyi task "Clippy (Workspace)"
	cargo clippy \
		--workspace \
		--all-features \
		--target-dir "{{ cargo_dir }}"

	fyi task "Clippy (Lib)."
	cargo clippy \
		--manifest-path "{{ pkg_dir2 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo clippy \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo clippy \
		--features=fitted \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo clippy \
		--features=timestamps \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo clippy \
		--features=fitted,timestamps \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo clippy \
		--features=progress \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo clippy \
		--features=signals_sigwinch \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo clippy \
		--features=signals_sigint \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo clippy \
		--all-features \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"


# Generate CREDITS.
@credits:
	cargo bashman -m "{{ pkg_dir1 }}/Cargo.toml" -t x86_64-unknown-linux-gnu
	just _fix-chown "{{ justfile_directory() }}/CREDITS.md"


# Build Docs.
@doc:
	# Make the docs.
	cargo +nightly rustdoc \
		--manifest-path "{{ pkg_dir2 }}/Cargo.toml" \
		--release \
		--target-dir "{{ cargo_dir }}" \
		-- \
		--cfg docsrs
	cargo +nightly rustdoc \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--release \
		--features fitted,progress,signals,timestamps \
		--target-dir "{{ cargo_dir }}" \
		-- \
		--cfg docsrs

	# Move the docs and clean up ownership.
	[ ! -d "{{ doc_dir }}" ] || rm -rf "{{ doc_dir }}"
	mv "{{ cargo_dir }}/doc" "{{ justfile_directory() }}"
	just _fix-chown "{{ doc_dir }}"


# Build and Run Example.
@ex DEMO:
	clear
	cargo run \
		-q \
		--all-features \
		--release \
		--example "{{ DEMO }}" \
		--target-dir "{{ cargo_dir }}"


# Test Run.
@run +ARGS:
	cargo run \
		--bin "{{ pkg_id }}" \
		--release \
		--target-dir "{{ cargo_dir }}" \
		-- {{ ARGS }}


# Unit tests!
@test:
	clear
	fyi task "Testing Bin (Release)."
	cargo test \
		--release \
		--manifest-path "{{ pkg_dir1 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"

	fyi task "Testing Lib (Release)."
	cargo test \
		--release \
		--manifest-path "{{ pkg_dir2 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo test \
		--release \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo test \
		--release \
		--features=fitted \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo test \
		--release \
		--features=timestamps \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo test \
		--release \
		--features=fitted,timestamps \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo test \
		--release \
		--features=progress \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo test \
		--release \
		--features=signals_sigwinch \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo test \
		--release \
		--features=signals_sigint \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo test \
		--release \
		--all-features \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"

	just _test-debug


# Test (Debug).
_test-debug:
	#!/usr/bin/env bash
	set -e

	unset -v RUSTFLAGS CC CXX CFLAGS CXXFLAGS LDFLAGS

	fyi task "Testing Bin (Debug)."
	cargo test \
		--manifest-path "{{ pkg_dir1 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"

	fyi task "Testing Lib (Debug)."
	cargo test \
		--manifest-path "{{ pkg_dir2 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo test \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo test \
		--features=fitted \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo test \
		--features=timestamps \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo test \
		--features=fitted,timestamps \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo test \
		--features=progress \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo test \
		--features=signals_sigwinch \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo test \
		--features=signals_sigint \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"
	cargo test \
		--all-features \
		--manifest-path "{{ pkg_dir3 }}/Cargo.toml" \
		--target-dir "{{ cargo_dir }}"


# Get/Set version.
version:
	#!/usr/bin/env bash
	set -e

	# Current version.
	_ver1="$( tomli query -f "{{ pkg_dir1 }}/Cargo.toml" package.version | \
		sed 's/[" ]//g' )"

	# Find out if we want to bump it.
	set +e
	_ver2="$( whiptail --inputbox "Set {{ pkg_name }} version:" --title "Release Version" 0 0 "$_ver1" 3>&1 1>&2 2>&3 )"

	exitstatus=$?
	if [ $exitstatus != 0 ] || [ "$_ver1" = "$_ver2" ]; then
		exit 0
	fi
	set -e

	# Set the release version!
	tomli set -f "{{ pkg_dir1 }}/Cargo.toml" -i package.version "$_ver2"
	tomli set -f "{{ pkg_dir3 }}/Cargo.toml" -i package.version "$_ver2"

	fyi success "Set version to $_ver2 (except for fyi_ansi)."


# Fix file/directory permissions.
@_fix-chmod PATH:
	[ ! -e "{{ PATH }}" ] || find "{{ PATH }}" -type f -exec chmod 0644 {} +
	[ ! -e "{{ PATH }}" ] || find "{{ PATH }}" -type d -exec chmod 0755 {} +


# Fix file/directory ownership.
@_fix-chown PATH:
	[ ! -e "{{ PATH }}" ] || chown -R --reference="{{ justfile() }}" "{{ PATH }}"
