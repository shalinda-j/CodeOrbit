﻿{
  mkShell,
  makeFontsConf,

  CodeOrbit-editor,

  rust-analyzer,
  cargo-nextest,
  cargo-hakari,
  cargo-machete,
  nixfmt-rfc-style,
  protobuf,
  nodejs_22,
}:
(mkShell.override { inherit (CodeOrbit-editor) stdenv; }) {
  inputsFrom = [ CodeOrbit-editor ];
  packages = [
    rust-analyzer
    cargo-nextest
    cargo-hakari
    cargo-machete
    nixfmt-rfc-style
    # TODO: package protobuf-language-server for editing CodeOrbit.proto
    # TODO: add other tools used in our scripts

    # `build.nix` adds this to the `CodeOrbit-editor` wrapper (see `postFixup`)
    # we'll just put it on `$PATH`:
    nodejs_22
  ];

  env =
    let
      baseEnvs =
        (CodeOrbit-editor.overrideAttrs (attrs: {
          passthru = { inherit (attrs) env; };
        })).env; # exfil `env`; it's not in drvAttrs
    in
    (removeAttrs baseEnvs [
      "LK_CUSTOM_WEBRTC" # download the staticlib during the build as usual
      "ZED_UPDATE_EXPLANATION" # allow auto-updates
      "CARGO_PROFILE" # let you specify the profile
      "TARGET_DIR"
    ])
    // {
      # note: different than `$FONTCONFIG_FILE` in `build.nix` â€“ this refers to relative paths
      # outside the nix store instead of to `$src`
      FONTCONFIG_FILE = makeFontsConf {
        fontDirectories = [
          "./assets/fonts/plex-mono"
          "./assets/fonts/plex-sans"
        ];
      };
      PROTOC = "${protobuf}/bin/protoc";
    };
}
