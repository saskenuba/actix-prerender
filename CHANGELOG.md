# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.2.3 (2022-03-08)

### New Features

 - <csr-id-2d974698ac0ce9e4f7124fc3d77dda3651f495ee/> added gzip auto-decoding


### Commit Statistics

<csr-read-only-do-not-edit/>

 - 1 commit contributed to the release.
 - 1 commit where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - added gzip auto-decoding ([`2d97469`](https://github.comgit//saskenuba/actix-prerender/commit/2d974698ac0ce9e4f7124fc3d77dda3651f495ee))
</details>

## 0.2.2 (2022-03-07)

### New Features

 - <csr-id-95e861de8513d8a5805021e1a0d5bf88125aa371/> added `set_before_render_fn`, to customize request headers
   before it is sent to prerender service;

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 4 calendar days.
 - 4 days passed between releases.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release actix-prerender v0.2.2 ([`64bd575`](https://github.comgit//saskenuba/actix-prerender/commit/64bd575b250832146911337336ae02f48bd64b50))
    - added `set_before_render_fn`, to customize request headers ([`95e861d`](https://github.comgit//saskenuba/actix-prerender/commit/95e861de8513d8a5805021e1a0d5bf88125aa371))
    - improved README.md ([`29f6f6a`](https://github.comgit//saskenuba/actix-prerender/commit/29f6f6a04c2b6eb9a1802b54fb4ad7c402227557))
</details>

## 0.2.1 (2022-03-02)

### New Features

 - <csr-id-669d8eb3109438437cdf2c9450d62c8121863ad3/> added special handling to `cf-visitor` and `X-Forwarded-Proto`
   .. and tests
 - <csr-id-47c258c8c8f8daafff11b847595a6ba89dbc9552/> added `forward_headers` on builder. defaults to false

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release actix-prerender v0.2.1 ([`efae229`](https://github.comgit//saskenuba/actix-prerender/commit/efae229a5786d05f0ea99797645711f1295223ad))
    - added special handling to `cf-visitor` and `X-Forwarded-Proto` ([`669d8eb`](https://github.comgit//saskenuba/actix-prerender/commit/669d8eb3109438437cdf2c9450d62c8121863ad3))
    - added `forward_headers` on builder. defaults to false ([`47c258c`](https://github.comgit//saskenuba/actix-prerender/commit/47c258c8c8f8daafff11b847595a6ba89dbc9552))
</details>

## v0.2.0 (2022-03-02)

### New Features

 - <csr-id-06d36fe6d779584d110214236697f62f6043ad67/> rustfmt file
 - <csr-id-e885d20622751c17db4ef08c56a858fd5501857e/> first commit

### Bug Fixes

 - <csr-id-cdb519c2f7a48f056ac4f77a95c0ea0b776980ce/> removed "prerender" token from builder if using custom URL
 - <csr-id-215a29197af06325c3a96cd0b07d89f6e8edfa9d/> repo updated, changed changelog
 - <csr-id-6882c0705a33b23b1a889560f911fea88d95708e/> now working properly by changing BoxBody -> EitherBody

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 11 commits contributed to the release over the course of 1 calendar day.
 - 9 commits where understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' where seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Release actix-prerender v0.2.0 ([`6850a8c`](https://github.comgit//saskenuba/actix-prerender/commit/6850a8c73f4d18d3d41fe13d380c5b43c9b38ca4))
    - removed "prerender" token from builder if using custom URL ([`cdb519c`](https://github.comgit//saskenuba/actix-prerender/commit/cdb519c2f7a48f056ac4f77a95c0ea0b776980ce))
    - repo updated, changed changelog ([`215a291`](https://github.comgit//saskenuba/actix-prerender/commit/215a29197af06325c3a96cd0b07d89f6e8edfa9d))
    - Release actix-prerender v0.1.0 ([`deb3d1c`](https://github.comgit//saskenuba/actix-prerender/commit/deb3d1ceb8c368542b7e699a4fda43043046da2e))
    - now working properly by changing BoxBody -> EitherBody ([`6882c07`](https://github.comgit//saskenuba/actix-prerender/commit/6882c0705a33b23b1a889560f911fea88d95708e))
    - refactored into multiple modules, export builders ([`e44643b`](https://github.comgit//saskenuba/actix-prerender/commit/e44643b73340461b9adfb0f45e9f8fd6b37fbde4))
    - more work, improved inner ergonomics, added error types ([`91d35eb`](https://github.comgit//saskenuba/actix-prerender/commit/91d35eb0e76e420b45e12c45d3fa025afa24d63e))
    - rustfmt file ([`06d36fe`](https://github.comgit//saskenuba/actix-prerender/commit/06d36fe6d779584d110214236697f62f6043ad67))
    - added basic skeleton of Service and Transform to ... ([`21d5f54`](https://github.comgit//saskenuba/actix-prerender/commit/21d5f54f13abe310c25207568e517ee099ec0f1f))
    - create ci.yml ([`35ef291`](https://github.comgit//saskenuba/actix-prerender/commit/35ef29199680903e82d5d82849f42ae4df1c1e85))
    - first commit ([`e885d20`](https://github.comgit//saskenuba/actix-prerender/commit/e885d20622751c17db4ef08c56a858fd5501857e))
</details>

## v0.1.0 (2022-03-02)

### New Features

 - Initial release

### Bug Fixes

 - <csr-id-ae353aa4753ffeccc2984db7420a3a2b13ff6201/> now working properly by changing BoxBody -> EitherBody

