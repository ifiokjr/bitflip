# Development container image

The optional editor container mounts the checkout and runs `devenv test`. Its base image in `.devcontainer.json` must include an immutable SHA-256 digest. The version tag describes the image; the digest selects the bytes, even if the tag later moves.

## Current pin provenance

Verified against the public `ghcr.io/cachix/devenv` registry on 2026-10-28:

| Item                    | Value                                                                                                                          |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| Published tag           | `v1.4`                                                                                                                         |
| Multi-platform manifest | `sha256:e916e7886fd87e0c63967cefef68c4b5c0cc24168a725fbf3d002de46699c596`                                                      |
| Linux amd64 manifest    | `sha256:bf0fdbc57eb83f4b962de97b267bd2813849198516a5ef6d96d440d78c237da4`                                                      |
| Linux arm64 manifest    | `sha256:07263b8f618b5cb997ff1ce5972f5cb2fd7f233940d67e6d32d104f23107a220`                                                      |
| Image source label      | `https://github.com/cachix/devenv`                                                                                             |
| Image revision label    | [`6c987a8795eedea872afe4d1c1ac518d0c7f6db1`](https://github.com/cachix/devenv/commit/6c987a8795eedea872afe4d1c1ac518d0c7f6db1) |

Both `latest` and `v1.4` resolved to this manifest at verification time, so the initial pin preserves the existing image bytes. SHA-256 hashes of the raw manifest, both platform manifests, and both image configurations matched their registry descriptors. Source and revision labels are publisher metadata; this check does not establish a signed build attestation.

CI's `devcontainer` job rejects unpinned image references, pulls the exact digest, and starts its `devenv` and Nix binaries without network access or a workspace mount. The normal CI jobs validate the repository independently. This smoke check does not run the editor's full `devenv test` bootstrap or exercise the arm64 image.

## Updating the image

1. Choose a reviewed upstream release from `cachix/devenv`. Resolve its multi-platform manifest from the public GHCR registry and verify the SHA-256 of the raw response against the registry digest. Verify the platform manifest and configuration hashes and inspect their source/revision labels against the upstream release.
2. Update `.devcontainer.json` and this provenance record together in a dependency PR. Retain the multi-platform digest so both supported Linux architectures remain available.
3. Require the current-head `devcontainer` check and normal repository CI to pass. Rebuild the editor container with the new pin, run `devenv test`, and record the platform, command outcome, and any bootstrap limitations in the PR before merging an image upgrade.

The initial change only freezes the already selected image. An image upgrade requires the rebuild evidence above; a digest does not establish that an image is free of vulnerabilities.
