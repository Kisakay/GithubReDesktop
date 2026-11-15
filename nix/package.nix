{
  lib,
  rustPlatform,
  libcosmicAppHook,
  just,
  pkg-config,
  makeWrapper,
  git,
  fontconfig,
  freetype,
  libxkbcommon,
  wayland,
  stdenv,
}:

let
  cargoToml = lib.importTOML ../Cargo.toml;
in
rustPlatform.buildRustPackage {
  pname = "github-redesktop";
  inherit (cargoToml.package) version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../src
      ../i18n
      ../i18n.toml
      ../resources
      ../justfile
      ../LICENSE
    ];
  };

  cargoLock = {
    lockFile = ../Cargo.lock;
    allowBuiltinFetchGit = true;
  };

  nativeBuildInputs = [
    just
    pkg-config
    libcosmicAppHook
    makeWrapper
  ];

  buildInputs = [
    fontconfig
    freetype
    libxkbcommon
    wayland
  ];

  # The test-suite drives a real git binary in a temporary repository.
  nativeCheckInputs = [ git ];

  dontUseJustBuild = true;
  dontUseJustCheck = true;

  justFlags = [
    "--set"
    "prefix"
    (placeholder "out")
    "--set"
    "cargo-target-dir"
    "target/${stdenv.hostPlatform.rust.cargoShortTarget}"
  ];

  # git is required at runtime: every repository operation shells out to it.
  preFixup = ''
    libcosmicAppWrapperArgs+=(--prefix PATH : ${lib.makeBinPath [ git ]})
  '';

  meta = {
    description = cargoToml.package.description;
    homepage = "https://github.com/Kisakay/GithubReDesktop";
    license = lib.licenses.mit;
    mainProgram = "github-redesktop";
    platforms = lib.platforms.linux;
  };
}
