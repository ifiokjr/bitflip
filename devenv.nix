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
      flyctl
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

  enterShell = ''
    set -e
    export PATH="$DEVENV_ROOT/.eget/bin:$PATH"
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
        HASH=$(nix hash path --base32 ./.eget/.eget.toml)
        echo "HASH: $HASH"
        if [ ! -f ./.eget/bin/hash ] || [ "$HASH" != "$(cat ./.eget/bin/hash)" ]; then
          echo "Updating eget binaries"
          eget -D --to "$DEVENV_ROOT/.eget/bin"
          echo "$HASH" > ./.eget/bin/hash
        else
          echo "eget binaries are up to date"
        fi
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
  };
}
