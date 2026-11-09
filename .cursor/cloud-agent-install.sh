#!/usr/bin/env bash

# Idempotent toolchain and dependency install for Cursor cloud agents.
# Installs single-user Nix (flakes and nix-command), devenv from the public
# devenv binary cache, warms that shell, then installs the pinned Flutter SDK
# and locked workspace dependencies.

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

# Nix and Flutter invoke git. A cloud-agent sandbox can export GIT_* variables
# that point those invocations at the wrong repository.
if [[ -n "${GIT_CONFIG_COUNT:-}" ]]; then
	git_config_count="${GIT_CONFIG_COUNT}"
	for ((git_config_index = 0; git_config_index < git_config_count; git_config_index++)); do
		unset "GIT_CONFIG_KEY_${git_config_index}" "GIT_CONFIG_VALUE_${git_config_index}" || true
	done
fi
unset GIT_ALTERNATE_OBJECT_DIRECTORIES GIT_CONFIG GIT_CONFIG_PARAMETERS || true
unset GIT_CONFIG_COUNT GIT_OBJECT_DIRECTORY GIT_DIR GIT_WORK_TREE || true
unset GIT_IMPLICIT_WORK_TREE GIT_GRAFT_FILE GIT_INDEX_FILE || true
unset GIT_NO_REPLACE_OBJECTS GIT_REPLACE_REF_BASE GIT_PREFIX || true
unset GIT_SHALLOW_FILE GIT_COMMON_DIR || true

export NIXPKGS_ALLOW_UNFREE=1
export CI="${CI:-true}"

nix_profile="${HOME}/.nix-profile/etc/profile.d/nix.sh"
nix_conf="${HOME}/.config/nix/nix.conf"

load_nix() {
	if [[ -f "${nix_profile}" ]]; then
		# shellcheck disable=SC1090
		. "${nix_profile}"
	fi
}

ensure_nix_config() {
	# Single-user Nix does not install /etc/nix/nix.conf, and login shells do
	# not enable flakes unless the user config does.
	mkdir -p "${HOME}/.config/nix"
	if [[ ! -f "${nix_conf}" ]] || ! grep -q 'experimental-features' "${nix_conf}"; then
		cat >"${nix_conf}" <<'EOF'
experimental-features = nix-command flakes
accept-flake-config = true
max-jobs = 2
cores = 2
EOF
	fi
}

install_nix() {
	load_nix
	if ! command -v nix >/dev/null 2>&1; then
		curl --proto '=https' --tlsv1.2 -sSfL https://nixos.org/nix/install -o /tmp/nix-install.sh
		sh /tmp/nix-install.sh --no-daemon --yes
		load_nix
	fi
	ensure_nix_config
	command -v nix >/dev/null 2>&1
}

install_profile_hook() {
	sudo tee /etc/profile.d/nix-cloud-agent.sh >/dev/null <<EOF
if [ -f "\${HOME}/.nix-profile/etc/profile.d/nix.sh" ]; then
	. "\${HOME}/.nix-profile/etc/profile.d/nix.sh"
fi
EOF

	# Login shells source /etc/profile.d. Non-login commands still find the
	# tools because these wrappers load the Nix profile before exec.
	sudo tee /usr/local/bin/nix >/dev/null <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [ -f "${HOME}/.nix-profile/etc/profile.d/nix.sh" ]; then
	# shellcheck disable=SC1090
	. "${HOME}/.nix-profile/etc/profile.d/nix.sh"
fi
exec "${HOME}/.nix-profile/bin/nix" "$@"
EOF
	sudo tee /usr/local/bin/devenv >/dev/null <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [ -f "${HOME}/.nix-profile/etc/profile.d/nix.sh" ]; then
	# shellcheck disable=SC1090
	. "${HOME}/.nix-profile/etc/profile.d/nix.sh"
fi
exec "${HOME}/.nix-profile/bin/devenv" "$@"
EOF
	sudo chmod 755 /usr/local/bin/nix /usr/local/bin/devenv
}

install_devenv() {
	load_nix
	local profile_bin="${HOME}/.nix-profile/bin"
	# Wrappers in /usr/local/bin exist before the profile binaries do, so
	# `command -v` is not a reliable installation check.
	if [[ ! -x "${profile_bin}/cachix" ]]; then
		nix profile add --accept-flake-config nixpkgs#cachix
		load_nix
	fi
	"${profile_bin}/cachix" use devenv
	if [[ ! -x "${profile_bin}/devenv" ]]; then
		# Nix 2.35 installs profile packages with `nix profile add`.
		# `nix profile install` is not a subcommand in this release.
		nix profile add --accept-flake-config nixpkgs#devenv
		load_nix
	fi
	[[ -x "${profile_bin}/devenv" ]]
}

warm_devenv_shell() {
	devenv shell -- true
}

install_flutter() {
	local flutter_version
	flutter_version="$(jq -er '.flutter' "${root}/.fvmrc")"
	if ! [[ "${flutter_version}" =~ ^[A-Za-z0-9._-]+$ ]]; then
		echo "The Flutter version in .fvmrc is missing or invalid." >&2
		exit 1
	fi

	devenv shell -- fvm install "${flutter_version}"
}

prefetch_rust_toolchain() {
	# Same race as CI: the first devenv shell runs dprint's parallel rustfmt
	# hook, and each rustfmt invokes rustup. Install the pinned toolchain
	# once before that shell exists.
	if ! command -v rustup >/dev/null 2>&1 && [[ -x "${HOME}/.cargo/bin/rustup" ]]; then
		export PATH="${HOME}/.cargo/bin:${PATH}"
	fi
	if command -v rustup >/dev/null 2>&1; then
		rustup toolchain install
	fi
}

echo "Installing Nix..."
install_nix
echo "Installing devenv..."
install_profile_hook
install_devenv
echo "Installing the pinned Rust toolchain..."
prefetch_rust_toolchain
echo "Warming the devenv shell..."
warm_devenv_shell
echo "Installing Flutter..."
install_flutter
echo "Installing workspace dependencies..."
devenv shell -- install:all
echo "Cloud agent install completed."
