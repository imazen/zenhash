# Release preparation

The crate is MIT OR Apache-2.0. Packages also retain the xxHash BSD-2-Clause
and rapidhash MIT notices in `LICENSE-XXHASH` and `LICENSE-RAPIDHASH`.
`Cargo.toml` carries the SPDX expression, repository, documentation URL and MSRV.

Edit `README.md`, then run `just readme`. The generated `README.crates.md` is
both the crates.io README and the rustdoc landing page. `just docs` builds the
same public features configured for docs.rs, including every optional algorithm
and `io`, without enabling the internal `_dev` feature. `just docs-open` opens
that local build. The public URL is <https://docs.rs/zenhash>; it returns 404
until the first successful publish and docs.rs build.

## Checks before approval

Run under the host's normal resource wrapper:

```sh
just release-check
just dependency-audit
# Initial preview: compare all enabled APIs with the pre-release main branch.
just semver-baseline
```

`release-check` runs default all-target tests, full-feature tests and doctests,
strict clippy, rustdoc, API snapshots, README freshness, packaged-copy verification
and `cargo publish --dry-run`. Benchmarks are compiled by clippy; timing runs are
separate from correctness checks. `cargo-semver-checks` 0.51.0 supports Rust 1.99's
rustdoc format; older installed versions may reject it before comparing APIs.
For releases after the first, compare against the published version with
`cargo semver-checks`, including the intended default/optional feature sets.
The initial all-features comparison does not validate the approved changes to
default feature availability recorded under QUEUED BREAKING CHANGES.

Before tagging, review the public API and README, update the changelog into a
dated release section, and obtain the maintainer's explicit publish approval.
New APIs are reviewed in a PR assigned to the maintainer. Push the exact release
commit and require green CI on every required platform, including ARM Windows,
ARM macOS and i686 via cross. Then create and push `v<version>`, create the GitHub
release, and publish that same commit. A dry run never grants approval to publish.

## GitHub publication setup

The existing `publish.yml` runs when a GitHub release is published. It checks
the tag against `Cargo.toml`, reruns the full CI matrix and publishes through
the `crates-io` environment using `CARGO_REGISTRY_TOKEN`.

As checked on 2026-10-27, that repository environment and token secret are not
configured. Check again before release; the values can change:

```sh
gh api repos/imazen/zenhash/environments --jq '.environments[].name'
gh secret list --repo imazen/zenhash
gh secret list --org imazen
```

Configure the environment and an appropriately scoped crates.io token before
using that automated publish path. Token values belong in secret storage.
After publishing, check the workflow result, the registry package contents and
the docs.rs build, including optional algorithms and the reader example.
