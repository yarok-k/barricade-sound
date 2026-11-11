# Пакет barricade-sound.
#   nix-build                       # сборка без flake
#   pkgs.callPackage ./default.nix {}  # из своей конфигурации
{
  pkgs ? import <nixpkgs> { },
  lib ? pkgs.lib,
  rustPlatform ? pkgs.rustPlatform,
}:

rustPlatform.buildRustPackage {
  pname = "barricade-sound";
  version = (lib.importTOML ./Cargo.toml).package.version;

  src = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.unions [
      ./Cargo.toml
      ./Cargo.lock
      ./src
    ];
  };

  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = with pkgs; [
    pkg-config
    wrapGAppsHook3
  ];

  # только то, что нужно для сборки и линковки
  buildInputs = with pkgs; [
    gtk3
    glib
    gdk-pixbuf
    gtk-layer-shell
    dbus
    libpulseaudio
    adwaita-icon-theme # символические иконки freedesktop
  ];

  # wpctl и playerctl нужны виджетам звука и плеера во время работы:
  # добавляем их в PATH обёртки, чтобы не зависеть от системного PATH
  preFixup = ''
    gappsWrapperArgs+=(--prefix PATH : ${lib.makeBinPath [ pkgs.wireplumber pkgs.playerctl ]})
  '';

  meta = {
    description = "Окно управления звуком для barricade";
    homepage = "https://github.com/yarok-k/barricade-sound";
    license = lib.licenses.gpl3Plus;
    # имя бинарника берётся из name в Cargo.toml (с заглавной буквы)
    mainProgram = "Barricade-sound";
    platforms = lib.platforms.linux;
  };
}
