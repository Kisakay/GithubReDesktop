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
    environment.systemPackages = [
      cfg.package
      pkgs.git
    ];
    # Used to store the GitHub token securely.
    services.gnome.gnome-keyring.enable = lib.mkDefault true;
  };
}
