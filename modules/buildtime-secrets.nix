{
  # needed to use with perSystem
  config,
  ...
}@perSystem:

{
  lib,
  config,
  pkgs,
  ...
}:
let
  cfg = config.buildtimeSecrets;

  hookName = lib.getName perSystem.config.packages.default;
  backendTools = [ pkgs.sops ];
in
{
  imports = [
    ./sops.nix
  ];

  options.buildtimeSecrets = {
    enable = lib.mkEnableOption "buildtime secrets";

    secretDirectory = lib.mkOption {
      type = lib.types.str;
      default = "/run/buildtime-secrets";
    };

    config = lib.mkOption {
      type = lib.types.attrs;
    };
  };

  config = lib.mkIf cfg.enable {
    buildtimeSecrets.config = {
      secret_dir = cfg.secretDirectory;
    };

    nix.settings = {
      system-features = [ "buildtime-secrets" ];

      pre-build-hook =
        pkgs.runCommand "${hookName}-wrap"
          {
            nativeBuildInputs = [ pkgs.makeWrapper ];
          }
          ''
            makeWrapper ${lib.getExe perSystem.config.packages.default} $out \
              --set RUST_LOG "debug" \
              --prefix PATH : ${lib.makeBinPath backendTools} \
              --set LOG_FILE "/var/log/buildtime-secrets/log" \
              --set CONFIG_FILE "${pkgs.writeText "${hookName}-config.json" (builtins.toJSON cfg.config)}"
          '';
    };
  };
}
