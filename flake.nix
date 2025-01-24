{
  description = "wasmer-interview-challenge";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      fenix,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            fenix.overlays.default
          ];
        };

        fenixPkgs = fenix.packages.${system};
        rustToolchain = fenixPkgs.combine [
          fenixPkgs.complete.toolchain
          fenixPkgs.targets.wasm32-unknown-unknown.stable.completeToolchain

          (fenixPkgs.complete.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rustc"
            "rustfmt"
          ])
        ];
      in
      rec {
        name = "wasmer-interview-challenge";

        devShell = pkgs.mkShell {
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          buildInputs = [
            rustToolchain
            pkgs.rust-analyzer-nightly
            pkgs.ldproxy
            pkgs.wasm-tools
            pkgs.cargo-rdme
          ];
        };

        packages.wasmlet =
          (pkgs.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          }).buildRustPackage
            {
              pname = "wasmlet";
              version = "0.1.0";

              src = ./wasmlet;

              cargoLock = {
                lockFile = ./wasmlet/Cargo.lock;
              };

              nativeBuildInputs = [
                pkgs.pkg-config
              ];

              meta = {
                description = "wasmer toy project";
                homepage = "https://github.com/zebreus/wasmer-interview-challenge";
                license = pkgs.lib.licenses.agpl3Plus;
              };
            };
        packages.default = packages.wasmlet;

        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
