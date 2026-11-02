{ pkgs ? import <nixpkgs> {}, ... }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
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
  ];

  buildInputs = with pkgs; [
    gtk3
    glib
    cairo
    pango
    gdk-pixbuf
    gtk-layer-shell
    networkmanager
    libpulseaudio
  ];
  LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
}
