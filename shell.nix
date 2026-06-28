{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = with pkgs; [
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
    export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath [
      pkgs.libglvnd
      pkgs.mesa
      pkgs.vulkan-loader
      pkgs.wayland
      pkgs.libxkbcommon
    ]}:/run/opengl-driver/lib:/run/opengl-driver-32/lib:$LD_LIBRARY_PATH
  '';
}
