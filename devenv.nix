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
      pkgs.sqlite
      pkgs.sql-formatter
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
  scripts.welds = {
    exec = ''
      set -e
      cargo bin welds $@
    '';
    description = "The `welds` executable for generating database models.";
  };
  scripts.sqlx = {
    exec = ''
      set -e
      cargo bin sqlx $@
    '';
    description = "The `sqlx` executable for database migrations.";
  };
  scripts."query-security-txt" = {
    exec = ''
      set -e
      cargo bin query-security-txt $@
    '';
    description = "The `query-security-txt` executable";
  };
  scripts."solana-verify" = {
    exec = ''
      set -e
      cargo bin solana-verify $@
    '';
    description = "The `solana-verify` executable";
  };
  scripts."install:all" = {
    exec = ''
      if [ ! -f "$DEVENV_ROOT/.env" ]; then
        cp "$DEVENV_ROOT/.env.example" "$DEVENV_ROOT/.env"
      fi

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
	scripts.bitflip = {
		exec = ''
			set -e
			deno task $@
		'';
		description = "The `bitflip` tasks";
	};
	scripts."test:all" = {
		exec = ''
			set -e
			cargo test_program
			cargo test_program_validator
			cargo test_bitflip_ssr
			# cargo test_bitflip_js
		'';
		description = "Run all tests";
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
			deno outdated -u --latest
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
	scripts."build:program:verified" = {
		exec = ''
			set -e
			solana-verify build --library-name bitflip_program
			solana-verify get-executable-hash $DEVENV_ROOT/target/deploy/bitflip_program.so > $DEVENV_ROOT/bitflip_program/program_hash.txt
		'';
		description = "Build the steel program.";
	};
  scripts."build:docker" = {
    exec = ''
      set -e
      docker build -t kj-dev -f $DEVENV_ROOT/bitflip/Dockerfile $DEVENV_ROOT
    '';
    description = "";
  };
}
