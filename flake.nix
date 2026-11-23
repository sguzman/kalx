{
  description = "Kalx: Kalshi CLI and read-only MCP server";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, crane }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      eachSystem = nixpkgs.lib.genAttrs systems;
      perSystem = eachSystem (system:
        let
          pkgs = import nixpkgs { inherit system; };
          craneLib = crane.mkLib pkgs;
          src = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [
              ./Cargo.toml ./Cargo.lock ./src ./config ./tests
            ];
          };
          commonArgs = {
            inherit src;
            strictDeps = true;
            cargoExtraArgs = "--locked";
            nativeBuildInputs = [ pkgs.pkg-config pkgs.cmake pkgs.perl ];
          };
          # Crane uses a dummy source here, so changes to Rust implementation
          # do not invalidate the compiled dependency artifact.
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
          package = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
            doCheck = true;
            # reqwest's platform verifier loads native trust roots even
            # for the local HTTP mock tests in the Nix sandbox.
            SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
            meta.mainProgram = "kalx";
          });
        in {
          packages = {
            default = package;
            kalx = package;
            cargo-artifacts = cargoArtifacts;
          };
          apps = rec {
            default = kalx;
            kalx = { type = "app"; program = "${package}/bin/kalx"; };
            mcp = { type = "app"; program = "${package}/bin/kalx-mcp"; };
          };
          checks = { kalx = package; };
          devShell = craneLib.devShell {
            inputsFrom = [ package ];
            packages = with pkgs; [
              rustfmt clippy rust-analyzer pkg-config cmake perl nix-output-monitor
            ];
          };
        });
    in {
      packages = eachSystem (system: perSystem.${system}.packages);
      apps = eachSystem (system: perSystem.${system}.apps);
      checks = eachSystem (system: perSystem.${system}.checks);
      devShells = eachSystem (system: { default = perSystem.${system}.devShell; });
    };
}
