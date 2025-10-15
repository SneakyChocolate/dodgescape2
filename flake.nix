{
  description = "bevy flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github.com/numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default =
          with pkgs;
          mkShell {
            buildInputs =
              [
                # Rust dependencies
                (rust-bin.stable.latest.default.override { extensions = [ "rust-src" ]; })
                pkg-config
              ]
              ++ lib.optionals (lib.strings.hasInfix "linux" system) [
                # for Linux
                # Audio
                alsa-lib
                # Cross Platform 3D Graphics API
                vulkan-loader
                # For debugging around vulkan
                vulkan-tools
                # Wayland support
                wayland # <<< ADDED: wayland-client development files
                # Other dependencies
                libudev-zero
                xorg.libX11
                xorg.libXcursor
                xorg.libXi
                xorg.libXrandr
                libxkbcommon
              ];

            # Set RUST_SRC_PATH for rust-analyzer
            RUST_SRC_PATH = "${rust-bin.stable.latest.rust.src}/lib/rustlib/src/rust/library";

            # Set LD_LIBRARY_PATH for dynamic linking
            # Use `lib.makeLibraryPath` with all relevant dependencies
            LD_LIBRARY_PATH = lib.makeLibraryPath [
              vulkan-loader
              wayland
              xorg.libX11
              xorg.libXi
              xorg.libXcursor
              libxkbcommon
            ];
          };
      }
    );
}
