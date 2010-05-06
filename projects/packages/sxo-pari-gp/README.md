# `@sxo/pari-gp`

PARI/GP dialect product package for SXO.

- **Publishable** via Trusted Publisher with other `@sxo/*` packages.
- **Feature Matrix required**: `tests/feature-matrix/` (declared by `sxo.featureMatrix`).
- Rust counterpart: `sxo-dialect-pari-gp` (`GpForm` scaffold; parse / lower still stubbed).
- Parser / lowering land incrementally; do not claim `supported` until matrix cases pass.

This package does not ship a PARI/GP runtime binary. Math still goes through Athena after dialect lowering.
