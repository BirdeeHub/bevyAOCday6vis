{ shellPkg
, pkg-config
, APPNAME
, mkShell
, pkgs
, lib
, ...
}: let
# dev shells should not contain the final program.
# They should have the environment
# needed to BUILD (and run) the final program.
  DEVSHELL = mkShell (rec {
    packages = [];
    inputsFrom = [];
    DEVSHELL = 0;
    inherit APPNAME;
    nativeBuildInputs = [ pkg-config ];
    buildInputs = with pkgs; [
      alsa-lib
      udev
      vulkan-loader
      lld
      clang
      pkg-config
      xorg.libX11
      xorg.libXcursor
      xorg.libXrandr
      xorg.libXi
      xorg.libxkbfile
      libxkbcommon
      vulkan-tools
      vulkan-headers
      vulkan-loader
      vulkan-validation-layers
    ];
    shellHook = ''
      export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${lib.makeLibraryPath (with pkgs; [ alsa-lib udev vulkan-loader libxkbcommon])}"
      exec ${shellPkg}
    '';
  });
in
DEVSHELL
