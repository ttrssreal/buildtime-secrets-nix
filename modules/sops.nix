{ config, lib, ... }:
let
  cfg = config.buildtimeSecrets.sops;
in
{
  options.buildtimeSecrets.sops = {
    enable = lib.mkEnableOption "the sops backend";

    sopsFile = lib.mkOption {
      type = lib.types.path;
    };

    keyFile = lib.mkOption {
      type = lib.types.str;
    };
  };

  config = lib.mkIf cfg.enable {
    buildtimeSecrets.config = {
      backend_config.sops = {
        sops_file = cfg.sopsFile;
        environment.SOPS_AGE_SSH_PRIVATE_KEY_FILE = cfg.keyFile;
      };
    };
  };
}
