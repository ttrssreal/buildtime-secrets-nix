#/usr/bin/env nix shell
#!nix nixpkgs#nix

nix eval \
  github:nixos/nixpkgs/e9f00bd893984bc8ce46c895c3bf7cac95331127#dwm-status
  >/dev/null
