{ pkgs ? import <nixpkgs> {} }:

let
  windowsTarget = "x86_64-pc-windows-gnu";
  mingwPthreads = pkgs.pkgsCross.mingwW64.windows.pthreads.overrideAttrs (old: {
    meta = old.meta // {
      platforms = pkgs.lib.platforms.all;
    };
  });
  linuxGuiLibraryPath = pkgs.lib.makeLibraryPath [
    pkgs.libglvnd
    pkgs.mesa
    pkgs.vulkan-loader
    pkgs.wayland
    pkgs.libxkbcommon
  ];
in
pkgs.mkShell {
  packages = with pkgs; [
    pkg-config
    rustup

    # Windows cross-compilation toolchain for Rust's *-pc-windows-gnu target.
    pkgsCross.mingwW64.stdenv.cc
    mingwPthreads

    libglvnd
    mesa
    mesa-demos
    pciutils
    vulkan-loader
    vulkan-tools
    wayland
    wayland-protocols
    libxkbcommon
  ];

  shellHook = ''
    export LD_LIBRARY_PATH=${linuxGuiLibraryPath}:/run/opengl-driver/lib:/run/opengl-driver-32/lib:$LD_LIBRARY_PATH

    export WIN_CARGO_TARGET=${windowsTarget}
    export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER="$(command -v x86_64-w64-mingw32-gcc)"
    export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_AR="$(command -v x86_64-w64-mingw32-ar)"

    echo "Windows GUI build: cargo build -p gui --release --target $WIN_CARGO_TARGET"
    echo "If the target is missing: rustup target add $WIN_CARGO_TARGET"
    echo "If Rustup references a deleted Nix ld-wrapper: rustup toolchain install stable --force"
  '';
}
