{ APPNAME
, lib
, makeRustPlatform
, fenix
, writeShellScriptBin
, pkg-config
, alsa-lib
, udev
, vulkan-loader
, libxkbcommon
, libX11
, libxcb
, pkgs
, ...
}: let
APPDRV = (makeRustPlatform fenix.packages.x86_64-linux.default).buildRustPackage {
  pname = APPNAME;
  version = "0.0.0";
  src = ./.;
  nativeBuildInputs = [ pkg-config ];
  buildInputs = with pkgs; [
    alsa-lib
    udev
    vulkan-loader
    llvmPackages.bintools
    clang
    pkg-config
    libX11
    libxcb
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

  cargoLock = {
    lockFileContents = builtins.readFile ./Cargo.lock;
  };

};
in
writeShellScriptBin APPNAME ''
  export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${lib.makeLibraryPath [ alsa-lib udev vulkan-loader libxkbcommon]}"
  exec ${APPDRV}/bin/${APPNAME} "$@"
''
