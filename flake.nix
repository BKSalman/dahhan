{
  description = "basic rust gui development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    ...
  }:
      let
        system = "x86_64-linux";

        pkgs = import nixpkgs { inherit system; overlays = [ rust-overlay.overlays.default ]; };

        nativeBuildInputs = with pkgs; [
          pkg-config
          cmake
          mesa
        ];

        buildInputs = with pkgs; [
          fontconfig
          freetype

          vulkan-headers
          vulkan-loader
          libGL

          libxkbcommon
          # WINIT_UNIX_BACKEND=wayland
          wayland

          # WINIT_UNIX_BACKEND=x11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
          xorg.libX11
        ];
      in with pkgs; {
        devShells.${system} = {
          default = mkShell {
            inherit buildInputs nativeBuildInputs;

            packages = with pkgs; [
              (rust-bin.stable.latest.default.override {
                extensions = [ "rust-src" "rust-analyzer" ];
              })
              cargo-watch
              gdb
              renderdoc
              valgrind
              heaptrack
            ];

            LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
          };
          nightly = mkShell {
            inherit buildInputs nativeBuildInputs;

            packages = with pkgs; [
              (rust-bin.nightly.latest.default.override {
                extensions = [ "rust-src" "rust-std" "miri" ];
              })
              cargo-watch
            ];

            LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
          };
        };
      };
}
