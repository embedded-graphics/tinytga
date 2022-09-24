targets := "arm-unknown-linux-gnueabi armv7-unknown-linux-gnueabihf x86_64-unknown-linux-gnu x86_64-unknown-linux-musl thumbv6m-none-eabi thumbv7em-none-eabi thumbv7em-none-eabihf thumbv7m-none-eabi"
target_dir := "target"

#----------
# Building
#----------

build: check-formatting test build-benches check-readme check-links

# Build the benches
build-benches:
    cargo bench --no-run

# Run cargo test
test:
    cargo test

# Check the formatting
check-formatting:
    cargo fmt --all -- --check

# Cross compiles tinytga for a target
build-target target *args:
    cargo build --target {{target}} {{args}}

# Cross compiles tinytga for all targets
build-targets *args:
    #!/usr/bin/env bash
    set -e
    for target in {{targets}}; do just build-target $target {{args}}; done

# Install all targets used in the `build-targets` command
install-targets:
    #!/usr/bin/env bash
    set -e

    sysroot=$(rustc --print sysroot)

    for target in {{targets}}; do
      if [[ ! "$sysroot" =~ "$target" ]]; then
        rustup target add $target
      else
        echo "Target $target is already installed"
      fi
    done

#------
# Docs
#------

# Generates the docs
generate-docs:
    cargo clean --doc
    cargo doc --all-features

# Runs cargo-deadlinks on the docs
check-links: generate-docs
    cargo deadlinks

#----------------------
# README.md generation
# ---------------------

# Generate README.md for a single crate
generate-readme: (_build-readme)
    cp {{target_dir}}/README.md README.md

# Check README.md for a single crate
@check-readme: (_build-readme)
    diff -q {{target_dir}}/README.md README.md || ( \
        echo -e "\033[1;31mError:\033[0m README.md needs to be regenerated."; \
        echo -e "       Run 'just generate-readme' to regenerate.\n"; \
        exit 1 \
    )

# Builds README.md for a single crate
_build-readme:
    #!/usr/bin/env bash
    set -e -o pipefail
    mkdir -p {{target_dir}}/readme
    echo "Building README.md"
    cargo readme | sed -E -f filter_readme.sed > {{target_dir}}/README.md
