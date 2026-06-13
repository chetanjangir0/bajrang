{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = with pkgs; [
    wayland
    wayland-protocols
    libxkbcommon
  ];

  shellHook = ''
    export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath [
      pkgs.wayland
      pkgs.libxkbcommon
    ]}:$LD_LIBRARY_PATH
  '';
}
