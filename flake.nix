{
  description = "Kalx: Kalshi CLI and read-only MCP server";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      eachSystem = nixpkgs.lib.genAttrs systems;
      pkgsFor = system: import nixpkgs { inherit system; };
    in {
      packages = eachSystem (system:
        let
          pkgs = pkgsFor system;
          package = pkgs.rustPlatform.buildRustPackage {
            pname = "kalx";
            version = "0.1.0";
            src = pkgs.lib.fileset.toSource {
              root = ./.;
              fileset = pkgs.lib.fileset.unions [
                ./Cargo.toml ./Cargo.lock ./src ./config ./tests
              ];
            };
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [ pkgs.pkg-config pkgs.cmake pkgs.perl ];
            strictDeps = true;
            doCheck = true;
            meta.mainProgram = "kalx";
          };
        in { default = package; kalx = package; });
      apps = eachSystem (system: {
        default = self.apps.${system}.kalx;
        kalx = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/kalx";
        };
        mcp = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/kalx-mcp";
        };
      });
      checks = eachSystem (system: { inherit (self.packages.${system}) kalx; });
      devShells = eachSystem (system:
        let pkgs = pkgsFor system; in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo rustc rustfmt clippy rust-analyzer pkg-config cmake perl
              nix-output-monitor
            ];
          };
        });
    };
}
