# Модуль NixOS: programs.barricade-bar.enable = true;
{ config, lib, pkgs, ... }:

let
  cfg = config.programs.barricade-bar;
in
{
  options.programs.barricade-bar = {
    enable = lib.mkEnableOption "barricade-sound, a sound applet for barricade";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.callPackage ./default.nix { };
      defaultText = lib.literalExpression "pkgs.callPackage ./default.nix { }";
      description = "Пакет barricade-sound.";
    };

    autostart = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = ''
        Запускать апплет пользовательской службой systemd вместе с графической
        сессией (graphical-session.target). Отключите, если запускаете бар
        сами, например через spawn-at-startup в конфиге niri.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    # бару нужны NetworkManager (сеть) и PipeWire/WirePlumber (звук)
    networking.networkmanager.enable = lib.mkDefault true;

    systemd.user.services.barricade-bar = lib.mkIf cfg.autostart {
      description = "Barricade Sound";
      wantedBy = [ "graphical-session.target" ];
      partOf = [ "graphical-session.target" ];
      after = [ "graphical-session.target" ];
      serviceConfig = {
        ExecStart = lib.getExe cfg.package;
        Restart = "on-failure";
        RestartSec = 2;
      };
    };
  };
}
