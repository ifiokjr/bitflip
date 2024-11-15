{ pkgs, lib, ... }:

{
  packages =
    [
      pkgs.binaryen
      pkgs.cargo-binstall
      pkgs.cargo-run-bin
      pkgs.coreutils
      pkgs.curl
      pkgs.deno
      pkgs.dprint
      pkgs.flyctl
      pkgs.jq
      pkgs.libiconv
      pkgs.mold
      pkgs.openssl
      pkgs.protobuf # needed for `solana-test-validator` in tests
      pkgs.nixfmt-rfc-style
      pkgs.rustup
      pkgs.shfmt
      pkgs.surrealdb
      pkgs.surrealdb-migrations
      pkgs.wasm-pack
    ]
    ++ lib.optionals pkgs.stdenv.isDarwin (
      with pkgs.darwin.apple_sdk;
      [
        frameworks.CoreFoundation
        frameworks.Security
        frameworks.System
        frameworks.SystemConfiguration
      ]
    );

  # disable dotenv since it breaks the variable interpolation supported by `direnv`
  dotenv.disableHint = true;

  scripts.anchor = {
    exec = ''
      set -e
      cargo bin anchor $@
    '';
    description = "The `anchor` executable";
  };
  scripts."install:all" = {
    exec = ''
      set -e
      install:cargo:bin
      install:solana
      install:deno
    '';
    description = "Install all packages.";
  };
  scripts."generate:keypair" = {
    exec = ''
      set -e
      solana-keygen new -s -o $DEVENV_ROOT/$1.json --no-bip39-passphrase || true
    '';
    description = "Generate a local solana keypair. Must provide a name.";
  };
  scripts."install:deno" = {
    exec = ''
      set -e
      deno install --allow-scripts
    '';
    description = "Install deno dependencies";
  };
  scripts."install:cargo:bin" = {
    exec = ''
      set -e
      cargo bin --install
    '';
    description = "Install cargo binaries locally.";
  };
  scripts."update:deps" = {
    exec = ''
            set -e
            cargo update
      			devenv update
    '';
    description = "Update dependencies.";
  };
  scripts."build:all" = {
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
  };
  scripts."build:docs" = {
    exec = ''
      RUSTDOCFLAGS="--cfg docsrs" cargo doc --all-features
    '';
    description = "Build documentation site.";
  };
  scripts."test:all" = {
    exec = ''
      set -e
      cargo test_bitflip_legacy_client
      cargo test_bitflip_legacy_client_validator
    '';
    description = "Run all tests across the crates";
  };
  scripts."db:start" = {
    exec = ''
      surreal start --log debug --user $SURREAL_USER --password $SURREAL_PASS
    '';
    description = "Start the surrealdb instance";
  };
  scripts."fix:all" = {
    exec = ''
      set -e
      fix:clippy
      fix:deno
      fix:format
    '';
    description = "Fix all autofixable problems.";
  };
  scripts."fix:format" = {
    exec = ''
      set -e
      dprint fmt --config "$DEVENV_ROOT/dprint.json"
    '';
    description = "Format files with dprint.";
  };
  scripts."fix:clippy" = {
    exec = ''
      set -e
      cargo clippy --fix --allow-dirty --allow-staged --all-features
    '';
    description = "Fix clippy lints for rust.";
  };
  scripts."fix:deno" = {
    exec = ''
      set -e
      deno lint --fix .
    '';
    description = "Fix lints for JS / TS.";
  };
  scripts."lint:all" = {
    exec = ''
      set -e
      lint:clippy
      lint:deno
      lint:format
    '';
    description = "Run all checks.";
  };
  scripts."lint:format" = {
    exec = ''
      set -e
      dprint check
    '';
    description = "Check that all files are formatted.";
  };
  scripts."lint:clippy" = {
    exec = ''
      set -e
      cargo clippy --all-features
    '';
    description = "Check that all rust lints are passing.";
  };
  scripts."lint:deno" = {
    exec = ''
      set -e
      deno lint
    '';
    description = "Check lints for all JS / TS files.";
  };
  scripts."setup:vscode" = {
    exec = ''
      set -e
      rm -rf .vscode
      cp -r $DEVENV_ROOT/setup/editors/vscode .vscode
    '';
    description = "Setup the environment for vscode.";
  };
  scripts."setup:helix" = {
    exec = ''
      set -e
      rm -rf .helix
      cp -r $DEVENV_ROOT/setup/editors/helix .helix
    '';
    description = "Setup for the helix editor.";
  };
  scripts."setup:ci" = {
    exec = ''
      set -e
      # update github ci path
      echo "$DEVENV_PROFILE/bin" >> $GITHUB_PATH
      echo "$GITHUB_WORKSPACE/.local-cache/solana-release/bin" >> $GITHUB_PATH

      # update github ci environment
      echo "DEVENV_PROFILE=$DEVENV_PROFILE" >> $GITHUB_ENV

      # prepend common compilation lookup paths
      echo "PKG_CONFIG_PATH=$PKG_CONFIG_PATH" >> $GITHUB_ENV
      echo "LD_LIBRARY_PATH=$LD_LIBRARY_PATH" >> $GITHUB_ENV
      echo "LIBRARY_PATH=$LIBRARY_PATH" >> $GITHUB_ENV
      echo "C_INCLUDE_PATH=$C_INCLUDE_PATH" >> $GITHUB_ENV

      # these provide shell completions / default config options
      echo "XDG_DATA_DIRS=$XDG_DATA_DIRS" >> $GITHUB_ENV
      echo "XDG_CONFIG_DIRS=$XDG_CONFIG_DIRS" >> $GITHUB_ENV

      echo "DEVENV_DOTFILE=$DEVENV_DOTFILE" >> $GITHUB_ENV
      echo "DEVENV_PROFILE=$DEVENV_PROFILE" >> $GITHUB_ENV
      echo "DEVENV_ROOT=$DEVENV_ROOT" >> $GITHUB_ENV
      echo "DEVENV_STATE=$DEVENV_STATE" >> $GITHUB_ENV

      fnm_env=$(fnm env --json)

      # Parse the JSON file contents
      PARSED_FNM_ENV=$(jq -r '.' <<< "$fnm_env")
      FNM_MULTISHELL_PATH=$(jq -r '.FNM_MULTISHELL_PATH' <<< "$PARSED_FNM_ENV")

      # Add fnm to the path
      echo "$FNM_MULTISHELL_PATH/bin" >> $GITHUB_PATH

      # add fnm environment variables
      for key in $(jq -r 'keys[]' <<< "$PARSED_FNM_ENV"); do
        value=$(jq -r ".$key" <<< "$PARSED_FNM_ENV")
        echo "$key=$value" >> $GITHUB_ENV
      done
    '';
    description = "Setup devenv for GitHub Actions";
  };
  scripts."setup:docker" = {
    exec = ''
      set -e
      # update path
      echo "export PATH=$DEVENV_PROFILE/bin:\$PATH" >> /etc/profile
      echo "export PATH=$DEVENV_ROOT/.local-cache/solana-release/bin:\$PATH" >> /etc/profile

      echo "export DEVENV_PROFILE=$DEVENV_PROFILE" >> /etc/profile
      echo "export PKG_CONFIG_PATH=$PKG_CONFIG_PATH" >> /etc/profile
      echo "export LD_LIBRARY_PATH=$LD_LIBRARY_PATH" >> /etc/profile
      echo "export LIBRARY_PATH=$LIBRARY_PATH" >> /etc/profile
      echo "export C_INCLUDE_PATH=$C_INCLUDE_PATH" >> /etc/profile
      echo "export XDG_DATA_DIRS=$XDG_DATA_DIRS" >> /etc/profile
      echo "export XDG_CONFIG_DIRS=$XDG_CONFIG_DIRS" >> /etc/profile

      echo "export DEVENV_DOTFILE=$DEVENV_DOTFILE" >> /etc/profile
      echo "export DEVENV_PROFILE=$DEVENV_PROFILE" >> /etc/profile
      echo "export DEVENV_ROOT=$DEVENV_ROOT" >> /etc/profile
      echo "export DEVENV_STATE=$DEVENV_STATE" >> /etc/profile

      fnm_env=$(fnm env --json)

      # Parse the JSON file contents
      PARSED_FNM_ENV=$(jq -r '.' <<< "$fnm_env")
      FNM_MULTISHELL_PATH=$(jq -r '.FNM_MULTISHELL_PATH' <<< "$PARSED_FNM_ENV")

      # add fnm to the path
      echo "export PATH=$FNM_MULTISHELL_PATH/bin:\$PATH" >> /etc/profile

      # add fnm environment variables
      for key in $(jq -r 'keys[]' <<< "$PARSED_FNM_ENV"); do
        value=$(jq -r ".$key" <<< "$PARSED_FNM_ENV")
        echo "export $key=$value" >> /etc/profile
      done
    '';
    description = "Setup devenv shell for docker.";
  };
  scripts."install:solana" = {
    exec = ''
      set -e
      SOLANA_DOWNLOAD_ROOT="https://github.com/anza-xyz/agave/releases/download"
      LOCAL_CACHE="$DEVENV_ROOT/.local-cache"
      VERSION=`cat $DEVENV_ROOT/setup/cache-versions.json | jq -r '.solana'`
      OS_TYPE="$(uname -s)"
      CPU_TYPE="$(uname -m)"

      case "$OS_TYPE" in
        Linux)
          OS_TYPE=unknown-linux-gnu
          ;;
        Darwin)
          if [[ $CPU_TYPE = arm64 ]]; then
            CPU_TYPE=aarch64
          fi
          OS_TYPE=apple-darwin
          ;;
        *)
          err "machine architecture is currently unsupported"
          ;;
      esac
      TARGET="$CPU_TYPE-$OS_TYPE"

      mkdir -p $LOCAL_CACHE
      TARBALL_PATH=solana-release-$TARGET.tar.bz2
      LOCAL_TARBALL_PATH=solana-$VERSION-release-$TARGET.tar.bz2
      FULL_TARBALL_PATH="$LOCAL_CACHE/$LOCAL_TARBALL_PATH"
      if [[ -e $FULL_TARBALL_PATH ]]
      then
        echo "Using cached solana release"
      else
        DOWNLOAD_URL="$SOLANA_DOWNLOAD_ROOT/$VERSION/$TARBALL_PATH"
        echo "Downloading $DOWNLOAD_URL to the local cache $FULL_TARBALL_PATH"
        curl --header "Authorization: Bearer $TEST_GITHUB_ACCESS_TOKEN" -sSfL "$DOWNLOAD_URL" -O
        mv $TARBALL_PATH $FULL_TARBALL_PATH
        tar jxf $FULL_TARBALL_PATH -C $LOCAL_CACHE
      fi
    '';
    description = "Install the version of solana or use one from the cache.";
  };
	scripts."build:program" = {
		exec = ''
			set -e
			cargo build-sbf --manifest-path $DEVENV_ROOT/bitflip_program/Cargo.toml --arch sbfv1
		'';
		description = "Build the steel program.";
	};
  scripts."build:program:legacy".exec = ''
    set -e
    anchor build
    generated_bitflip_legacy_program_idl="$DEVENV_ROOT/target/idl/bitflip_legacy_program.json"
    ci_bitflip_legacy_program_idl="$DEVENV_ROOT/idls/bitflip_legacy_program_ci.json"
    saved_bitflip_legacy_program_idl="$DEVENV_ROOT/idls/bitflip_legacy_program.json"

    if [ -n "$CI" ]; then
      echo "ℹ️ running inside CI"
      cp -f $generated_bitflip_legacy_program_idl $ci_bitflip_legacy_program_idl
      dprint fmt $DEVENV_ROOT/idls/*.json
      if cmp -s "$ci_bitflip_legacy_program_idl" "$saved_bitflip_legacy_program_idl"; then
        echo "✅ files are identical"
        rm $ci_bitflip_legacy_program_idl
        exit 0
      else
        echo "❌ idl files were not updated"
        echo "ℹ️ make sure to run `anchor:build` before pushing your code"
      fi

      rm $ci_bitflip_legacy_program_idl
      exit 1
    fi

    echo "ℹ️ running outside ci"
    cp -f $generated_bitflip_legacy_program_idl $saved_bitflip_legacy_program_idl
    dprint fmt $DEVENV_ROOT/idls/*.json
  '';
  tasks."build:docker" = {
    exec = ''
      set -e
      docker build -t kj-dev -f $DEVENV_ROOT/bitflip/Dockerfile $DEVENV_ROOT
    '';
    description = "";
  };
  tasks."watch:bitflip" = {
    exec = ''
      cargo make watch:bitflip
    '';
    description = "";
  };
  tasks."build:bitflip" = {
    exec = ''
      set -e
      cargo make build:bitflip:tailwind
      cargo leptos build --project bitflip --release -vv --features="prod"
    '';
    description = "";
  };
  tasks."serve:bitflip" = {
    exec = ''
      set -e
      cargo make build:bitflip:tailwind
      cargo leptos serve --project bitflip --release -vv --features="prod"
    '';
    description = "";
  };
  tasks."prepare:bitflip" = {
    exec = ''
      set -e
      rm -rf $DEVENV_ROOT/dist/bitflip
      mkdir -p $DEVENV_ROOT/dist/bitflip
      cp $DEVENV_ROOT/target/release/bitflip $DEVENV_ROOT/dist/bitflip/bitflip
      cp -r $DEVENV_ROOT/target/site/bitflip $DEVENV_ROOT/dist/bitflip/site
      cp -r $DEVENV_ROOT/bitflip/Cargo.toml $DEVENV_ROOT/dist/bitflip
    '';
    description = "";
  };
  tasks."watch:bitflip:leptos" = {
    exec = ''
      cargo leptos watch --hot-reload --project bitflip
    '';
    description = "";
  };
}
