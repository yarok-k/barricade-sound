# Пакет barricade-bar.
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

  buildInputs = with pkgs; [
    pkg-config
    clang
    wrapGAppsHook3
    gcc
    gdb
    rustc
    cargo
    rust-analyzer
    usbutils
    exfatprogs
    playerctl
    libpulseaudio
    adwaita-icon-theme # символические иконки freedesktop

  ];

  # wpctl нужен виджету audio во время работы
  preFixup = ''
    gappsWrapperArgs+=(--prefix PATH : ${lib.makeBinPath [ pkgs.wireplumber ]})
  '';

  meta = {
    description = "Статусбар для niri на GTK3";
    homepage = "https://github.com/yarok-k/barricade-bar";
    license = lib.licenses.gpl3Plus;
    mainProgram = "barricade_bar";
    platforms = lib.platforms.linux;
  };
}
