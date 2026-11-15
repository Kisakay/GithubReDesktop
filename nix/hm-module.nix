self:
{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.github-redesktop;
in
{
  options.programs.github-redesktop = {
    enable = lib.mkEnableOption "GitHub ReDesktop, a native GitHub Desktop for Linux";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
      defaultText = lib.literalExpression "github-redesktop.packages.\${system}.default";
      description = "The GitHub ReDesktop package to install.";
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];
  };
}
