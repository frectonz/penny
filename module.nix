{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.penny;
  settingsFormat = pkgs.formats.toml { };
  configFile = settingsFormat.generate "penny.toml" cfg.settings;
in
{
  options.services.penny = {
    enable = lib.mkEnableOption "penny, a serverless reverse proxy for personal servers";

    package = lib.mkPackageOption pkgs "penny" { };

    address = lib.mkOption {
      type = lib.types.str;
      default = "0.0.0.0:80";
      description = "The HTTP address to bind to.";
    };

    httpsAddress = lib.mkOption {
      type = lib.types.str;
      default = "0.0.0.0:443";
      description = "The HTTPS address to bind to.";
    };

    environmentFile = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      description = ''
        Path to an environment file loaded by the service.
        Useful for setting secrets like `PENNY_PASSWORD` without
        exposing them in the Nix store.
      '';
      example = "/run/secrets/penny.env";
    };

    dataDir = lib.mkOption {
      type = lib.types.path;
      default = "/var/lib/penny";
      description = "Directory for penny's state (database, certificates).";
    };

    settings = lib.mkOption {
      type = lib.types.submodule {
        freeformType = settingsFormat.type;

        options = {
          database_url = lib.mkOption {
            type = lib.types.str;
            default = "sqlite://penny.db";
            description = "Database URL for penny's SQLite database.";
          };
        };
      };
      default = { };
      description = ''
        Configuration for penny in Nix attribute set form.
        This will be serialized to TOML and written to penny.toml.
        See <https://github.com/frectonz/penny> for available options.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.penny = {
      description = "Penny - Serverless reverse proxy";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];

      environment = {
        RUST_LOG = "tracing=info,penny=info";
      };

      serviceConfig = {
        Type = "exec";
        DynamicUser = true;
        StateDirectory = "penny";
        WorkingDirectory = cfg.dataDir;

        ExecStart =
          let
            args = lib.cli.toCommandLineShellGNU { } {
              address = cfg.address;
              https-address = cfg.httpsAddress;
            };
          in
          "${lib.getExe cfg.package} serve ${configFile} ${args}";

        Restart = "on-failure";
        RestartSec = 5;

        # Capabilities for binding to privileged ports
        AmbientCapabilities = [ "CAP_NET_BIND_SERVICE" ];
        CapabilityBoundingSet = [ "CAP_NET_BIND_SERVICE" ];

        # Hardening
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        PrivateDevices = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
        RestrictSUIDSGID = true;
        ReadWritePaths = [ cfg.dataDir ];
      }
      // lib.optionalAttrs (cfg.environmentFile != null) {
        EnvironmentFile = cfg.environmentFile;
      };
    };
  };
}
