# Prebuilt binary from the GitHub release, patched for NixOS.
# Releases are immutable, so the hashes pinned in release.json never change.
{
  lib,
  stdenv,
  fetchurl,
  autoPatchelfHook,
  makeWrapper,
  git,
  libxkbcommon,
  libGL,
  vulkan-loader,
  wayland,
  libx11,
  libxcursor,
  libxi,
  libxrandr,
  libxcb,
  cosmic-icons,
  release ? lib.importJSON ./release.json,
}:

let
  inherit (release) version;
  arch = stdenv.hostPlatform.parsed.cpu.name;
  # Libraries opened with dlopen() at runtime (graphics & windowing).
  runtimeLibs = [
    libxkbcommon
    libGL
    vulkan-loader
    wayland
    libx11
    libxcursor
    libxi
    libxrandr
    libxcb
  ];
in
stdenv.mkDerivation {
  pname = "github-redesktop";
  inherit version;

  src = fetchurl {
    url = "https://github.com/Kisakay/GithubReDesktop/releases/download/v${version}/github-redesktop-${version}-linux-${arch}.tar.gz";
    hash = release.hashes.${stdenv.hostPlatform.system};
  };

  nativeBuildInputs = [
    autoPatchelfHook
    makeWrapper
  ];

  buildInputs = [
    libxkbcommon
    stdenv.cc.cc.lib
  ];

  dontConfigure = true;
  dontBuild = true;

  installPhase = ''
    runHook preInstall
    mkdir -p $out
    cp -r bin share $out/
    runHook postInstall
  '';

  postFixup = ''
    wrapProgram $out/bin/github-redesktop \
      --prefix PATH : ${lib.makeBinPath [ git ]} \
      --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath runtimeLibs} \
      --suffix XDG_DATA_DIRS : $out/share:${cosmic-icons}/share
  '';

  doInstallCheck = true;
  installCheckPhase = ''
    $out/bin/github-redesktop --version | grep -q ${version}
  '';

  meta = {
    description = "A fast, native GitHub Desktop for Linux (prebuilt binary)";
    homepage = "https://regit.v.bzh";
    license = lib.licenses.mit;
    mainProgram = "github-redesktop";
    platforms = [
      "x86_64-linux"
      "aarch64-linux"
    ];
    sourceProvenance = [ lib.sourceTypes.binaryNativeCode ];
  };
}
