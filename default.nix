{ APPNAME
, rustPlatform
, src
, writeShellScriptBin
, pkg-config
, alsa-lib
, libudev-zero
, ...
}: let
APPDRV = rustPlatform.buildRustPackage {
  pname = APPNAME;
  version = "0.0.0";
  src = src;
  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ alsa-lib libudev-zero ];

  cargoLock = {
    lockFileContents = builtins.readFile "${src}/Cargo.lock";
  };

};
in
writeShellScriptBin APPNAME ''
  export AOC_INPUT=''${1:-./${APPNAME}/input};
  exec ${APPDRV}/bin/${APPNAME}
''
