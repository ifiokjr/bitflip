{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:

let
  extra = inputs.ifiokjr-nixpkgs.packages.${pkgs.stdenv.hostPlatform.system};
  appDir = "bitflip_app";
  serverWorkspaceDir = "bitflip_server";
  serverDir = "bitflip_server/bitflip_server_server";
  serverpodVersion = "4.0.0-rc.2";
  resolveFlutterSdk = ''
    unset GIT_ALTERNATE_OBJECT_DIRECTORIES GIT_CONFIG GIT_CONFIG_PARAMETERS
    unset GIT_CONFIG_COUNT GIT_OBJECT_DIRECTORY GIT_DIR GIT_WORK_TREE
    unset GIT_IMPLICIT_WORK_TREE GIT_GRAFT_FILE GIT_INDEX_FILE
    unset GIT_NO_REPLACE_OBJECTS GIT_REPLACE_REF_BASE GIT_PREFIX
    unset GIT_SHALLOW_FILE GIT_COMMON_DIR

    flutter_version="$(awk -F'"' '/"flutter"/ { print $4; exit }' "$DEVENV_ROOT/.fvmrc")"
    if ! [[ "$flutter_version" =~ ^[A-Za-z0-9._-]+$ ]]; then
      echo "The Flutter version in .fvmrc is missing or invalid." >&2
      exit 1
    fi

    local_flutter_sdk="$DEVENV_ROOT/.fvm/flutter_sdk"
    fvm_cache_root="''${FVM_CACHE_PATH:-''${FVM_HOME:-$HOME/fvm}}"
    cached_flutter_sdk="$fvm_cache_root/versions/$flutter_version"
    if [ "''${CI:-}" = "true" ] && [ -x "''${FLUTTER_ROOT:-}/bin/flutter" ]; then
      flutter_sdk="''${FLUTTER_ROOT}"
    elif [ -x "$local_flutter_sdk/bin/flutter" ]; then
      flutter_sdk="$local_flutter_sdk"
    elif [ -x "$cached_flutter_sdk/bin/flutter" ]; then
      flutter_sdk="$cached_flutter_sdk"
    else
      echo "Flutter $flutter_version is not installed. Run: fvm install $flutter_version" >&2
      exit 1
    fi
  '';
  resolveServerpod = ''
    ${resolveFlutterSdk}
    serverpod_cache="$DEVENV_ROOT/.tools/serverpod"
    serverpod_bin="$serverpod_cache/bin/serverpod"
    if [ ! -x "$serverpod_bin" ]; then
      echo "Serverpod ${serverpodVersion} is not installed. Run: install:all" >&2
      exit 1
    fi
    export PUB_CACHE="$serverpod_cache"
    export PATH="$flutter_sdk/bin:$PATH"
  '';
in
{
  packages =
    with pkgs;
    [
      cargo-audit
      cargo-deny
      cargo-run-bin
      curl
      dprint
      extra.agave
      extra.sbpf-linker
      extra.solana-verify
      extra.surfpool
      fvm
      git
      gitleaks
      jq
      libiconv
      nodejs_24
      openssl
      perl
      pkg-config
      rustup
      shfmt
      zlib
    ]
    ++ lib.optionals stdenv.isDarwin [
      coreutils
      podman
      vfkit
    ];

  apple.sdk = null;
  dotenv.disableHint = true;

  env = {
    PINA_BPF_TOOLCHAIN = "nightly-2025-11-20";
    PINA_LINT_TOOLCHAIN = "nightly-2026-02-20";
    SBF_TOOLS_VERSION = "v1.54";
    SOLANA_RPC_URL = "http://127.0.0.1:8899";
    SURFPOOL_RPC_URL = "http://127.0.0.1:8899";
    BITFLIP_METADATA_BASE_URL = "http://localhost:8082";
  };

  git-hooks = {
    package = pkgs.prek;
    hooks = {
      dprint = {
        enable = true;
        entry = "${pkgs.dprint}/bin/dprint fmt --allow-no-files";
        pass_filenames = true;
        stages = [ "pre-commit" ];
      };
      gitleaks = {
        enable = true;
        entry = "${pkgs.gitleaks}/bin/gitleaks protect --staged --verbose --redact";
        pass_filenames = false;
        stages = [ "pre-commit" ];
      };
    };
  };

  tasks."devenv:git-hooks:install".exec = lib.mkForce ''
    if ! ${pkgs.git}/bin/git rev-parse --git-dir &> /dev/null; then
      exit 0
    fi
    ${pkgs.git}/bin/git config --local --unset-all core.hooksPath 2>/dev/null || true
    GIT_CONFIG_GLOBAL=/dev/null ${pkgs.prek}/bin/prek install -f -c .pre-commit-config.yaml -t pre-commit
  '';

  scripts = {
    pnpm = {
      exec = ''
        set -euo pipefail
        exec corepack pnpm "$@"
      '';
      description = "Run the package.json-pinned pnpm through Corepack.";
      binary = "bash";
    };
    pina = {
      exec = ''
        set -euo pipefail
        if [ -n "''${PINA_BIN:-}" ]; then
          exec "$PINA_BIN" "$@"
        fi
        if [ -x "$DEVENV_ROOT/.tools/pina/bin/pina" ]; then
          exec "$DEVENV_ROOT/.tools/pina/bin/pina" "$@"
        fi
        cargo bin pina_cli "$@"
      '';
      description = "Run the workspace-pinned Pina CLI.";
      binary = "bash";
    };
    flutter = {
      exec = ''
        set -euo pipefail
        unset CC CXX LD AR NM RANLIB STRIP OBJCOPY OBJDUMP SIZE STRINGS
        unset NIX_CC NIX_BINTOOLS NIX_CFLAGS_COMPILE NIX_LDFLAGS
        unset NIX_HARDENING_ENABLE NIX_ENFORCE_NO_NATIVE
        unset SDKROOT MACOSX_DEPLOYMENT_TARGET CFLAGS CXXFLAGS LDFLAGS ARCHFLAGS
        unset PKG_CONFIG PKG_CONFIG_PATH LD_LIBRARY_PATH LD_DYLD_PATH cmakeFlags
        ${resolveFlutterSdk}
        exec "$flutter_sdk/bin/flutter" "$@"
      '';
      description = "Run the pinned Flutter SDK with an Xcode-safe environment.";
      binary = "bash";
    };
    serverpod = {
      exec = ''
        set -euo pipefail
        ${resolveServerpod}
        exec "$serverpod_bin" "$@"
      '';
      description = "Run the workspace-pinned Serverpod CLI with Flutter's Dart SDK.";
      binary = "bash";
    };
    dart = {
      exec = ''
        set -euo pipefail
        ${resolveFlutterSdk}
        exec "$flutter_sdk/bin/dart" "$@"
      '';
      description = "Run Dart from the pinned Flutter SDK.";
      binary = "bash";
    };
    "flutter:app" = {
      exec = ''
        set -euo pipefail
        ${resolveFlutterSdk}
        cd "$DEVENV_ROOT/${appDir}"
        exec "$flutter_sdk/bin/flutter" "$@"
      '';
      description = "Run Flutter from the Bitflip app package.";
      binary = "bash";
    };
    "serverpod:server" = {
      exec = ''
        set -euo pipefail
        ${resolveServerpod}
        cd "$DEVENV_ROOT/${serverDir}"
        exec "$serverpod_bin" "$@"
      '';
      description = "Run Serverpod CLI commands from the Bitflip server package.";
      binary = "bash";
    };
    "install:all" = {
      exec = ''
        set -euo pipefail
        ${resolveFlutterSdk}
        cargo fetch --locked
        CI=true pnpm install --frozen-lockfile
        export PUB_CACHE="$DEVENV_ROOT/.tools/serverpod"
        "$flutter_sdk/bin/dart" pub global activate \
          serverpod_cli ${serverpodVersion}
        cd "$DEVENV_ROOT/${serverWorkspaceDir}"
        "$flutter_sdk/bin/flutter" pub get
        cd "$DEVENV_ROOT/${appDir}"
        "$flutter_sdk/bin/flutter" pub get
      '';
      description = "Install locked Rust, generator, Serverpod, and Flutter dependencies.";
      binary = "bash";
    };
    "install:pina-lint" = {
      exec = ''
        set -euo pipefail
        rustup toolchain install "$PINA_LINT_TOOLCHAIN" \
          --profile minimal \
          --component llvm-tools-preview \
          --component rustc-dev \
          --component rust-src
      '';
      description = "Install Pina's pinned security-lint compiler components.";
      binary = "bash";
    };
    "generate:clients" = {
      exec = ''
        set -euo pipefail
        CI=true pnpm install --frozen-lockfile
        pina generate --project "$DEVENV_ROOT/bitflip_program" --client dart --npx node
      '';
      description = "Generate the Dart client from the Pina account model.";
      binary = "bash";
    };
    "generate:server" = {
      exec = ''
        set -euo pipefail
        ${resolveServerpod}
        cd "$DEVENV_ROOT/${serverDir}"
        "$serverpod_bin" generate
      '';
      description = "Generate the Serverpod protocol and typed Dart client.";
      binary = "bash";
    };
    "migration:create" = {
      exec = ''
        set -euo pipefail
        ${resolveServerpod}
        cd "$DEVENV_ROOT/${serverDir}"
        "$serverpod_bin" create-migration "$@"
      '';
      description = "Create a Serverpod database migration.";
      binary = "bash";
    };
    "server:start" = {
      exec = ''
        set -euo pipefail
        ${resolveServerpod}
        cd "$DEVENV_ROOT/${serverDir}"
        exec "$serverpod_bin" start "$@"
      '';
      description = "Start Serverpod with code generation, migrations, and hot reload.";
      binary = "bash";
    };
    "build:program" = {
      exec = ''
        set -euo pipefail
        pina build --project "$DEVENV_ROOT/bitflip_program"
      '';
      description = "Build the Bitflip Solana program and IDL.";
      binary = "bash";
    };
    "build:server" = {
      exec = ''
        set -euo pipefail
        ${resolveFlutterSdk}
        cd "$DEVENV_ROOT/${serverDir}"
        "$flutter_sdk/bin/dart" build cli \
          --target bin/main.dart \
          --output build
      '';
      description = "Compile the production Serverpod backend bundle.";
      binary = "bash";
    };
    "build:web" = {
      exec = ''
        set -euo pipefail
        cd "$DEVENV_ROOT"
        pnpm wallet:build
        ${resolveFlutterSdk}
        cd "$DEVENV_ROOT/${appDir}"
        "$flutter_sdk/bin/flutter" build web \
          --release \
          --wasm \
          --no-web-resources-cdn \
          --base-href / \
          --output "$DEVENV_ROOT/${serverDir}/web/app" \
          "$@"
      '';
      description = "Build the WASM Flutter site into Serverpod's web root.";
      binary = "bash";
    };
    "build:container" = {
      exec = ''
        set -euo pipefail
        build:web
        if command -v docker >/dev/null 2>&1; then
          container_runtime="docker"
        elif command -v podman >/dev/null 2>&1; then
          container_runtime="podman"
        else
          echo "Install Docker or Podman before building the container." >&2
          exit 1
        fi
        "$container_runtime" build \
          --file "$DEVENV_ROOT/${serverDir}/Dockerfile" \
          --tag bitflip-server:verification \
          "$DEVENV_ROOT"
      '';
      description = "Build the production Serverpod image with the Flutter site.";
      binary = "bash";
    };
    "fix:format" = {
      exec = ''
        set -euo pipefail
        dprint fmt
        ${resolveFlutterSdk}
        mapfile -d "" dart_sources < <(
          find \
            "$DEVENV_ROOT/${appDir}/lib" \
            "$DEVENV_ROOT/${appDir}/test" \
            "$DEVENV_ROOT/${appDir}/integration_test" \
            "$DEVENV_ROOT/${serverDir}/lib" \
            "$DEVENV_ROOT/${serverDir}/test" \
            -type f \
            -name "*.dart" \
            ! -name "*.g.dart" \
            ! -path "*/generated/*" \
            ! -path "*/test_tools/*" \
            -print0
        )
        if [ "''${#dart_sources[@]}" -gt 0 ]; then
          "$flutter_sdk/bin/dart" format "''${dart_sources[@]}"
        fi
      '';
      description = "Format Rust-adjacent files and Dart sources.";
      binary = "bash";
    };
    "lint:rust" = {
      exec = ''
        set -euo pipefail
        install:pina-lint
        cargo clippy --workspace --all-targets --all-features -- -D warnings

        lint_sysroot="$(rustup run "$PINA_LINT_TOOLCHAIN" rustc --print sysroot)"
        export LIBRARY_PATH="$lint_sysroot/lib''${LIBRARY_PATH:+:$LIBRARY_PATH}"
        export RUSTFLAGS="-Lnative=$lint_sysroot/lib''${RUSTFLAGS:+ $RUSTFLAGS}"
        if [ "$(uname -s)" = "Darwin" ]; then
          export DYLD_FALLBACK_LIBRARY_PATH="$lint_sysroot/lib''${DYLD_FALLBACK_LIBRARY_PATH:+:$DYLD_FALLBACK_LIBRARY_PATH}"
        else
          export LD_LIBRARY_PATH="$lint_sysroot/lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
        fi

        RUSTUP_TOOLCHAIN="$PINA_LINT_TOOLCHAIN" \
          pina lint --project "$DEVENV_ROOT/bitflip_program"
      '';
      description = "Run strict Rust lints.";
      binary = "bash";
    };
    "lint:dart" = {
      exec = ''
        set -euo pipefail
        ${resolveFlutterSdk}
        cd "$DEVENV_ROOT/${appDir}"
        "$flutter_sdk/bin/flutter" analyze
        cd "$DEVENV_ROOT/${serverDir}"
        "$flutter_sdk/bin/dart" analyze
      '';
      description = "Run strict Flutter and Serverpod static analysis.";
      binary = "bash";
    };
    "lint:wallet" = {
      exec = ''
        set -euo pipefail
        cd "$DEVENV_ROOT"
        pnpm wallet:check
      '';
      description = "Type-check the browser Wallet Standard bridge.";
      binary = "bash";
    };
    "lint:all" = {
      exec = ''
        set -euo pipefail
        lint:rust
        lint:wallet
        if [ -d "$DEVENV_ROOT/${appDir}" ]; then lint:dart; fi
      '';
      description = "Run all project lints.";
      binary = "bash";
    };
    "audit:security" = {
      exec = ''
        set -euo pipefail
        cargo audit
        cargo deny check
        pnpm audit --audit-level high
        gitleaks dir --no-banner --redact --timeout 60 "$DEVENV_ROOT"
      '';
      description = "Audit Rust advisories, licenses, sources, and repository secrets.";
      binary = "bash";
    };
    "test:rust" = {
      exec = ''
        set -euo pipefail
        cargo test --workspace --all-features
      '';
      description = "Run native Rust tests.";
      binary = "bash";
    };
    "test:surfpool" = {
      exec = ''
        set -euo pipefail
        pina test --project "$DEVENV_ROOT/bitflip_program"
      '';
      description = "Build the real SBF artifact and test it in isolated Surfpool.";
      binary = "bash";
    };
    "fetch:surfpool-programs" = {
      exec = ''
        set -euo pipefail
        bash "$DEVENV_ROOT/setup/scripts/fetch_surfpool_programs.sh"
      '';
      description = "Fetch and verify pinned Bubblegum Surfpool program artifacts.";
      binary = "bash";
    };
    "test:surfpool:cnft" = {
      exec = ''
        set -euo pipefail
        build:program
        fetch:surfpool-programs
        ${resolveFlutterSdk}
        cd "$DEVENV_ROOT/${serverDir}"
        "$flutter_sdk/bin/dart" test test/surfpool/bitflip_cnft_test.dart
      '';
      description = "Test Bitflip's atomic Bubblegum cNFT mint on Surfpool.";
      binary = "bash";
    };
    "test:flutter" = {
      exec = ''
        set -euo pipefail
        ${resolveFlutterSdk}
        cd "$DEVENV_ROOT/${appDir}"
        "$flutter_sdk/bin/flutter" test
      '';
      description = "Run Flutter unit and widget tests.";
      binary = "bash";
    };
    "test:wallet" = {
      exec = ''
        set -euo pipefail
        cd "$DEVENV_ROOT"
        pnpm wallet:test
      '';
      description = "Test Wallet Standard discovery and signing boundaries.";
      binary = "bash";
    };
    "test:flutter:integration" = {
      exec = ''
        set -euo pipefail
        ${resolveFlutterSdk}
        cd "$DEVENV_ROOT/${appDir}"
        "$flutter_sdk/bin/flutter" test integration_test
      '';
      description = "Run the permanent Flutter integration flow.";
      binary = "bash";
    };
    "test:server" = {
      exec = ''
        set -euo pipefail
        ${resolveFlutterSdk}
        cd "$DEVENV_ROOT/${serverDir}"
        if [ ! -f config/passwords.yaml ]; then
          export SERVERPOD_DATABASE_PASSWORD=bitflip-test-only
        fi
        "$flutter_sdk/bin/dart" test \
          --exclude-tags surfpool \
          --reporter expanded \
          --chain-stack-traces
      '';
      description = "Run Serverpod unit and embedded-PostgreSQL integration tests.";
      binary = "bash";
    };
    "test:all" = {
      exec = ''
        set -euo pipefail
        test:rust
        test:surfpool
        test:surfpool:cnft
        test:server
        test:wallet
        if [ -d "$DEVENV_ROOT/${appDir}" ]; then test:flutter; fi
      '';
      description = "Run Rust, Surfpool, and Flutter tests.";
      binary = "bash";
    };
  };

  processes.serverpod.exec = "server:start";
}
