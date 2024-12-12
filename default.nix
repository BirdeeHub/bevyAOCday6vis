{ APPNAME
, rustPlatform
, src
, writeShellScriptBin
, pkg-config
, alsa-lib
, libudev-zero
, libX11
, libxcb
, ...
}: let
APPDRV = rustPlatform.buildRustPackage {
  pname = APPNAME;
  version = "0.0.0";
  src = src;
  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ libX11 libxcb alsa-lib libudev-zero ];

  cargoLock = {
    lockFileContents = builtins.readFile ./Cargo.lock;
  };

};
in
writeShellScriptBin APPNAME ''
  export AOC_INPUT=''${1:-./${APPNAME}/input};
  exec ${APPDRV}/bin/${APPNAME}
''
