{ pkgs ? import <nixpkgs> { } }:

with pkgs;

mkShell rec {
  nativeBuildInputs = [
    pkg-config
    cargo
    rustc
    libgcc
  ];
  buildInputs = [
    udev alsa-lib-with-plugins vulkan-loader
    xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr # To use the x11 feature
    libxkbcommon wayland # To use the wayland feature
    rust-analyzer
  ];
  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
}
