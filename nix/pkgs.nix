# To update nix-prefetch-git https://github.com/NixOS/nixpkgs
import ((import <nixpkgs> {}).fetchFromGitHub {
  owner = "NixOS";
  repo = "nixpkgs";
  rev = "57eac89459226f3ec743ffa6bbbc1042f5836843";
  sha256  = "sha256:19820jsr4na0a2d0xfv0qdmz22r4qmq6kpv1fr3hwv79v1ns5kb0";
})
