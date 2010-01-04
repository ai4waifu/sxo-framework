# SXO Homepage

Public marketing site for **SXO** (symbolic computing). Built with VMZ as a static site and kept outside the core
product build, test, and npm publish graphs.

Browser demos may still pin a published WASM face for Cloudflare Pages (which cannot compile Rust in CI). That pin is a
site concern only — it must not pull `@sxo/core` native addons or dialect packages into the homepage dependency graph.
