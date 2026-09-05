{
  pkgs,
  lib,
  config,
  ...
}:

{
  packages =
    with pkgs;
    [
      bash
      binaryen
      cargo-binstall
      cargo-run-bin
      coreutils
      curl
      dprint
      eget
      jq
      libiconv
      nixfmt-rfc-style
      nodejs_24
      openssl
      pnpm
      protobuf # needed for `solana-test-validator` in tests
      rustup
      shfmt
      sql-formatter
      sqlite
      wasm-pack
    ]
    ++ lib.optionals stdenv.isDarwin [
      coreutils
    ];

  env.EGET_CONFIG = "${config.env.DEVENV_ROOT}/.eget/.eget.toml";
  env.LIBCLANG_PATH = lib.makeSearchPath "lib" [ pkgs.llvmPackages.libclang.lib ];
  env.LD_LIBRARY_PATH = lib.optionalString pkgs.stdenv.isLinux (
    lib.makeLibraryPath [ pkgs.stdenv.cc.cc.lib ]
  );

  enterShell = ''
    set -e
    export PATH="$DEVENV_ROOT/.eget/bin:$PATH"

    CARGO_BIN_ROOT="$DEVENV_ROOT/.bin/rust-$(rustc --version | cut -d ' ' -f 2)"
    for cargo_bin_dir in "$CARGO_BIN_ROOT"/*/*/bin; do
      if [ -d "$cargo_bin_dir" ]; then
        export PATH="$cargo_bin_dir:$PATH"
      fi
    done
  '';

  # Rely on the global sdk for now as the nix apple sdk is not working for me.
  apple.sdk = null;

  scripts = {
    bitflip = {
      exec = ''
        set -e
        cargo run -p bitflip_cli -- $@
      '';
      description = "The `bitflip` executable for running the Bitflip CLI.";
      binary = "bash";
    };
    welds = {
      exec = ''
        set -e
        cargo bin welds $@
      '';
      description = "The `welds` executable for generating database models.";
      binary = "bash";
    };
    sqlx = {
      exec = ''
        set -e
        cargo bin sqlx $@
      '';
      description = "The `sqlx` executable for database migrations.";
      binary = "bash";
    };
    "query-security-txt" = {
      exec = ''
        set -e
        cargo bin query-security-txt $@
      '';
      description = "The `query-security-txt` executable";
      binary = "bash";
    };
    "solana-verify" = {
      exec = ''
        set -e
        cargo bin solana-verify $@
      '';
      description = "The `solana-verify` executable";
      binary = "bash";
    };

    "generate:keypair" = {
      exec = ''
        set -e
        solana-keygen new -s -o $DEVENV_ROOT/$1.json --no-bip39-passphrase || true
      '';
      description = "Generate a local solana keypair. Must provide a name.";
      binary = "bash";
    };

    "test:all" = {
      exec = ''
        set -e
        cargo test_program
        cargo test_program_validator
        cargo test_bitflip_ssr
        # cargo test_bitflip_js
      '';
      description = "Run all tests";
      binary = "bash";
    };
    "install:all" = {
      exec = ''
        set -e

        install:cargo:bin
        install:eget
      '';
      description = "Install all packages.";
      binary = "bash";
    };

    "install:cargo:bin" = {
      exec = ''
        set -e
        cargo bin --install
      '';
      description = "Install cargo binaries locally.";
      binary = "bash";
    };
    "install:eget" = {
      exec = ''
        # Agave 2.1.21 scans this directory before its SDK installer gets a
        # chance to create it. Fresh CI runners otherwise panic in
        # cargo-build-sbf while looking for cached platform tools.
        mkdir -p "$HOME/.cache/solana"

        HASH=$(nix hash path --base32 ./.eget/.eget.toml)
        echo "HASH: $HASH"
        if [ ! -f ./.eget/bin/hash ] || [ "$HASH" != "$(cat ./.eget/bin/hash)" ]; then
          echo "Updating eget binaries"
          eget -D --to "$DEVENV_ROOT/.eget/bin"
          echo "$HASH" > ./.eget/bin/hash
        else
          echo "eget binaries are up to date"
        fi

        # eget intentionally flattens executable files from release archives.
        # Restore the SDK script layout expected by cargo-build-sbf.
        for sdk_root in sdk platform-tools-sdk; do
          SBF_ROOT="$DEVENV_ROOT/.eget/bin/$sdk_root/sbf"
          mkdir -p "$SBF_ROOT"
          cp "$DEVENV_ROOT/.eget/sbf/env.sh" "$SBF_ROOT/env.sh"
          SBF_SCRIPTS="$DEVENV_ROOT/.eget/bin/$sdk_root/sbf/scripts"
          mkdir -p "$SBF_SCRIPTS"
          for script in dump install objcopy package strip; do
            cp "$DEVENV_ROOT/.eget/bin/$script.sh" "$SBF_SCRIPTS/$script.sh"
            chmod +x "$SBF_SCRIPTS/$script.sh"
          done
        done
      '';
      description = "Install github binaries with eget.";
    };
    "update:deps" = {
      exec = ''
        set -e
        devenv update
        cargo update
      '';
      description = "Update dependencies.";
      binary = "bash";
    };
    "build:all" = {
      exec = ''
        set -e
        if [ -z "$CI" ]; then
          echo "Builing project locally"
          cargo build --all-features
        else
          echo "Building in CI"
          cargo build --all-features --locked
        fi
      '';
      description = "Build all crates with all features activated.";
      binary = "bash";
    };
    "build:css" = {
      exec = ''
        set -e
        pnpm dlx @tailwindcss/cli@4.1.2 \
          --input "$DEVENV_ROOT/bitflip/style/input.css" \
          --output "$DEVENV_ROOT/bitflip/style/output.css" \
          --minify
      '';
      description = "Build the Tailwind CSS bundle.";
      binary = "bash";
    };
    "build:web" = {
      exec = ''
        set -e
        build:css
        cargo bin wasm-bindgen-cli --version

        CARGO_BIN_ROOT="$DEVENV_ROOT/.bin/rust-$(rustc --version | cut -d ' ' -f 2)"
        for cargo_bin_dir in "$CARGO_BIN_ROOT"/*/*/bin; do
          if [ -d "$cargo_bin_dir" ]; then
            export PATH="$cargo_bin_dir:$PATH"
          fi
        done

        cargo bin cargo-leptos build --release --lib-cargo-args="--locked" --bin-cargo-args="--locked"
      '';
      description = "Build the production web application.";
      binary = "bash";
    };
    "build:program" = {
      exec = ''
        set -e
        cargo build-sbf --manifest-path $DEVENV_ROOT/bitflip_program/Cargo.toml --arch sbfv1
      '';
      description = "Build the steel program.";
      binary = "bash";
    };
    "build:program:verified" = {
      exec = ''
        set -e
        solana-verify build --library-name bitflip_program
        solana-verify get-executable-hash $DEVENV_ROOT/target/deploy/bitflip_program.so > $DEVENV_ROOT/bitflip_program/program_hash.txt
      '';
      description = "Build the steel program.";
      binary = "bash";
    };
    "lint:format" = {
      exec = ''
        set -e
        cargo fmt --all -- --check
        dprint check
      '';
      description = "Check Rust and repository formatting.";
      binary = "bash";
    };
    "lint:rust" = {
      exec = ''
        set -e
        cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
      '';
      description = "Run Clippy across the workspace.";
      binary = "bash";
    };
  };
}
