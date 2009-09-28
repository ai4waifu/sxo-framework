# Native Platform Artifact: Linux x64

This package is an optional Linux x64 native addon for the SXO high-level packages. Install the high-level package instead of importing this artifact directly. npm selects it when the operating system and CPU match the published binary.

Deployment images must provide the runtime libraries required by the produced binary. Test the exact container and Node.js ABI used in production. If loading fails, inspect the diagnostic, package manager optional-dependency settings, libc environment, and architecture before filing an issue. Apache License 2.0. See [LICENSE](../../../../LICENSE).
