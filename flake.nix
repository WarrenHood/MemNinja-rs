{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
        workspace = ./.; # Your Cargo workspace directory
        ldLibraryPath = pkgs.lib.makeLibraryPath (with pkgs; [
          libxkbcommon
        ]);
      in
      {
        defaultPackage = naersk-lib.buildPackage {
          src = workspace;
          pname = "memninja-tui"; # Set package name explicitly
        };
        devShell = with pkgs; mkShell {
          nativeBuildInputs = [
            pkg-config
          ];
          buildInputs = [
            cargo
            rustc
            rustfmt
            pre-commit
            rust-analyzer
            rustPackages.clippy
            udev
            alsa-lib
            libxkbcommon
          ];
          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
          LD_LIBRARY_PATH = ldLibraryPath;
        };

        shellHook = ''
          export LD_LIBRARY_PATH="${ldLibraryPath}:$LD_LIBRARY_PATH"
        '';
      }
    );
}
