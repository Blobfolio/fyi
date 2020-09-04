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
pkg_dir2    := justfile_directory() + "/fyi_menu"
pkg_dir3    := justfile_directory() + "/fyi_msg"
pkg_dir4    := justfile_directory() + "/fyi_witcher"

cargo_dir   := "/tmp/" + pkg_id + "-cargo"
cargo_bin   := cargo_dir + "/x86_64-unknown-linux-gnu/release/" + pkg_id
release_dir := justfile_directory() + "/release"

# If we ever want to use Clang native:
# -Clinker-plugin-lto -Clinker=clang-9 -Clink-args=-fuse-ld=lld-9
rustflags   := "-C link-arg=-s"



# A/B Test Two Binaries (second is implied)
@ab BIN="/usr/bin/fyi" REBUILD="":
	[ -z "{{ REBUILD }}" ] || just build
	[ -f "{{ cargo_bin }}" ] || just build

	clear

	fyi print -p "{{ BIN }}" -c 209 "$( "{{ BIN }}" -V )"
	fyi print -p "{{ cargo_bin }}" -c 199 "$( "{{ cargo_bin }}" -V )"
	fyi blank

	just _ab "{{ BIN }}" 'error "Twinkle, twinkle little star, how I wonder what you are."' 2>/dev/null
	just _ab "{{ BIN }}" 'error -t "Twinkle, twinkle little star, how I wonder what you are."' 2>/dev/null
	just _ab "{{ BIN }}" 'error -i -t "Twinkle, twinkle little star, how I wonder what you are."' 2>/dev/null
	just _ab "{{ BIN }}" 'print -p "Iron Maiden" -c 199 "Let he who hath understanding reckon the number of the beast."' 2>/dev/null


# A/B Test Inner
@_ab BIN ARGS:
	"{{ BIN }}" {{ ARGS }}
	"{{ cargo_bin }}" {{ ARGS }}

	#sleep 30
	hyperfine --warmup 50 \
		--runs 1000 \
		--style color \
		'{{ BIN }} {{ ARGS }}' \
		'{{ cargo_bin }} {{ ARGS }}'

	echo "\n\033[2m-----\033[0m\n\n"


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


# Bin Test!
@bin-test:
	[ -f "{{ cargo_bin }}" ] || just build

	"{{ cargo_bin }}" print "This message has no prefix."

	"{{ cargo_bin }}" blank

	"{{ cargo_bin }}" print -p "Pink" -c 199 "This message has a custom pink prefix."
	"{{ cargo_bin }}" print -p "Blue" -c 4 "This message has a custom blue prefix."

	"{{ cargo_bin }}" blank

	"{{ cargo_bin }}" notice "So official!"
	"{{ cargo_bin }}" success "Hurray! You did it!"
	"{{ cargo_bin }}" warning "Hold it there, Sparky!"
	"{{ cargo_bin }}" error "Oopsie."

	"{{ cargo_bin }}" blank

	"{{ cargo_bin }}" debug "The devil is in the details."
	"{{ cargo_bin }}" info "Details without the word 'bug'."
	"{{ cargo_bin }}" task "Let's get to work!"

	"{{ cargo_bin }}" blank

	"{{ cargo_bin }}" crunched "Some hard work just happened."
	"{{ cargo_bin }}" done "As the French say, «FIN»."

	"{{ cargo_bin }}" blank

	"{{ cargo_bin }}" info -t "Messages can be timestamped."

	"{{ cargo_bin }}" blank

	"{{ cargo_bin }}" info "Messages can be indented."
	"{{ cargo_bin }}" info -i "Messages can be indented."

	"{{ cargo_bin }}" blank

	"{{ cargo_bin }}" confirm "Did this work for you?" || "{{ cargo_bin }}" error "Well that sucks."

	"{{ cargo_bin }}" blank


# Build Release!
@build: clean
	# For perf runs, use RUSTFLAGS="-C force-frame-pointers=y -g", and update
	# Cargo.toml: no lto, opt-level 1, debug = true

	# First let's build the Rust bit.
	RUSTFLAGS="{{ rustflags }}" cargo -Z package-features build \
		--bin "{{ pkg_id }}" \
		--features simd \
		--release \
		--target x86_64-unknown-linux-gnu \
		--target-dir "{{ cargo_dir }}"


# Build Debian package!
@build-deb: build-man build
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
@build-man:
	# Pre-clean.
	find "{{ pkg_dir1 }}/man" -type f -delete

	# Build a quickie version with the unsexy help so help2man can parse it.
	RUSTFLAGS="{{ rustflags }}" cargo -Z package-features build \
		--bin "{{ pkg_id }}" \
		--release \
		--features man \
		--target x86_64-unknown-linux-gnu \
		--target-dir "{{ cargo_dir }}"

	# Use help2man to make a crappy MAN page.
	help2man -o "{{ pkg_dir1 }}/man/{{ pkg_id }}.1" \
		-N "{{ cargo_bin }}"

	# Gzip it and reset ownership.
	gzip -k -f -9 "{{ pkg_dir1 }}/man/{{ pkg_id }}.1"
	just _fix-chown "{{ pkg_dir1 }}/man"


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

	# But some Cargo apps place shit in subdirectories even if
	# they place *other* shit in the designated target dir. Haha.
	[ ! -d "{{ justfile_directory() }}/target" ] || rm -rf "{{ justfile_directory() }}/target"
	[ ! -d "{{ pkg_dir1 }}/target" ] || rm -rf "{{ pkg_dir1 }}/target"
	[ ! -d "{{ pkg_dir2 }}/target" ] || rm -rf "{{ pkg_dir2 }}/target"
	[ ! -d "{{ pkg_dir3 }}/target" ] || rm -rf "{{ pkg_dir3 }}/target"
	[ ! -d "{{ pkg_dir4 }}/target" ] || rm -rf "{{ pkg_dir4 }}/target"

	cargo update


# Clippy.
@clippy:
	clear
	RUSTFLAGS="{{ rustflags }}" cargo clippy \
		--workspace \
		--release \
		--all-features \
		--target x86_64-unknown-linux-gnu \
		--target-dir "{{ cargo_dir }}"


# Build and Run Example.
@ex DEMO:
	clear
	cargo run \
		-q \
		--example "{{ DEMO }}" \
		--target x86_64-unknown-linux-gnu \
		--target-dir "{{ cargo_dir }}"


# Test Run.
@run +ARGS:
	RUSTFLAGS="{{ rustflags }}" cargo run \
		--bin "{{ pkg_id }}" \
		--release \
		--target x86_64-unknown-linux-gnu \
		--target-dir "{{ cargo_dir }}" \
		-- {{ ARGS }}


# Unit tests!
@test:
	clear

	fyi notice "Testing All Features"

	RUST_TEST_THREADS=1 cargo test \
		--tests \
		--all-features \
		--release \
		--workspace \
		--target x86_64-unknown-linux-gnu \
		--target-dir "{{ cargo_dir }}" -- \
			--format terse \

	fyi notice "Testing Default"

	RUST_TEST_THREADS=1 cargo test \
		--tests \
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
	rustup default nightly
	rustup component add clippy --toolchain nightly
	[ ! -f "{{ justfile_directory() }}/Cargo.lock" ] || rm "{{ justfile_directory() }}/Cargo.lock"
	cargo update


# Fix file/directory permissions.
@_fix-chmod PATH:
	[ ! -e "{{ PATH }}" ] || find "{{ PATH }}" -type f -exec chmod 0644 {} +
	[ ! -e "{{ PATH }}" ] || find "{{ PATH }}" -type d -exec chmod 0755 {} +


# Fix file/directory ownership.
@_fix-chown PATH:
	[ ! -e "{{ PATH }}" ] || chown -R --reference="{{ justfile() }}" "{{ PATH }}"
