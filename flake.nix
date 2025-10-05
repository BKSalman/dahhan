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

          libGL
          vulkan-headers vulkan-loader
          vulkan-tools vulkan-tools-lunarg
          vulkan-extension-layer
          vulkan-validation-layers

          libxkbcommon
          # WINIT_UNIX_BACKEND=wayland
          wayland

          # WINIT_UNIX_BACKEND=x11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
          xorg.libX11
        ];

        packages = pkgs: buildInputs;

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
              gdbgui
              (pkgs.buildFHSEnv {
                name = "renderdoc";
                targetPkgs = packages;
                runScript = ''
                  #!/usr/bin/env bash
                  ${renderdoc}/bin/qrenderdoc
                '';
              })
              valgrind
              heaptrack
            ];

            LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
            VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
            VULKAN_SDK = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
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
            VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
            VULKAN_SDK = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
          };
        };
      };
}
