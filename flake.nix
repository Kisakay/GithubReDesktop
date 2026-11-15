{
  description = "GitHub ReDesktop — a native GitHub Desktop for Linux, built with Rust & libcosmic";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      # Prebuilt release binary when available (no compilation), source build otherwise.
      packages = forAllSystems (
        pkgs:
        let
          system = pkgs.stdenv.hostPlatform.system;
          release = nixpkgs.lib.importJSON ./nix/release.json;
          hasBinary = release.version != null && release.hashes ? ${system};
          source = pkgs.callPackage ./nix/package.nix { };
          binary = pkgs.callPackage ./nix/bin.nix { inherit release; };
        in
        {
          github-redesktop-source = source;
          github-redesktop = if hasBinary then binary else source;
          default = self.packages.${system}.github-redesktop;
        }
        // nixpkgs.lib.optionalAttrs hasBinary { github-redesktop-bin = binary; }
      );

      apps = forAllSystems (pkgs: rec {
        github-redesktop = {
          type = "app";
          program = "${self.packages.${pkgs.stdenv.hostPlatform.system}.default}/bin/github-redesktop";
        };
        default = github-redesktop;
      });

      overlays.default = final: _prev: {
        github-redesktop = self.packages.${final.stdenv.hostPlatform.system}.github-redesktop;
      };

      nixosModules.default = import ./nix/module.nix self;
      nixosModules.github-redesktop = self.nixosModules.default;

      homeManagerModules.default = import ./nix/hm-module.nix self;

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          inputsFrom = [ self.packages.${pkgs.stdenv.hostPlatform.system}.github-redesktop-source ];
          packages = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
            just
          ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
            with pkgs;
            [
              wayland
              libxkbcommon
              vulkan-loader
              libGL
              xorg.libX11
              xorg.libXcursor
              xorg.libXi
              xorg.libXrandr
            ]
          );
        };
      });

      formatter = forAllSystems (pkgs: pkgs.nixfmt-rfc-style);
    };
}
