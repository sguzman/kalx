# Build policy

- Use crate2nix for Nix builds, not Crane, cargo2nix or a monolithic dependency bundle.
- Preserve per-crate compiled Nix derivations and reuse via the Nix store.
- Keep Cargo.lock authoritative. Regenerate Cargo.nix with the pinned
  `nix develop --command crate2nix generate` when Cargo metadata changes;
  track generated Cargo.nix and any crate-hashes.json.
- Use `nom build` for local Nix build verification.
- Preserve the read-only MCP boundary and sandbox CA certificates.
