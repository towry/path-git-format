{
  description = "path-git-format";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
      manifest = (nixpkgs.lib.importTOML ./Cargo.toml).package;
      rust = fenix.packages.${system}.fromToolchainFile {
        dir = ./.;
        sha256 = "sha256-jUR2Sd3jhDJIvcuMLBgxtTqa3ELWF38V7IqoZT8EzVU=";
        # sha256 = nixpkgs.lib.fakeSha256;
      };
    in {
      devShell = pkgs.mkShell {
        packages = [rust];
      };
      packages = {
        default = pkgs.stdenv.mkDerivation {
          pname = "${manifest.name}";
          version = "${manifest.version}";
          src = self;
          nativeBuildInputs = [rust pkgs.pkg-config];
          buildInputs =
            [
              pkgs.openssl
            ]
            ++ nixpkgs.lib.optionals pkgs.stdenv.isDarwin [pkgs.libiconv pkgs.darwin.apple_sdk.frameworks.Security];

          buildPhase = ''
            export HOME=$(pwd)
            cargo --version
            make release
          '';
          installPhase = ''
            cp ./target/release/path-git-format $out/bin/
          '';
          meta = {
            description = "A simple utility to format path with git repo info from stdin";
            homepage = "https://github.com/towry/path-git-format";
            license = pkgs.lib.licenses.unlicense;
          };
        };
      };
    });
}