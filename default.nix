let
  sources = import ./nix/sources.nix;
  nixpkgs-mozilla = import sources.nixpkgs-mozilla;
  pkgs = import sources.nixpkgs {
    overlays =
      [
        nixpkgs-mozilla
        (self: super:
            let chan = self.rustChannelOf { date = "2023-07-12"; channel = "nightly"; };
            in {
              rustc = chan.rust;
              cargo = chan.rust;
            }
        )
      ];
  };
  naersk = pkgs.callPackage sources.naersk {};
  merged-openssl = pkgs.symlinkJoin { name = "merged-openssl"; paths = [ pkgs.openssl.out pkgs.openssl.dev ]; };
in
naersk.buildPackage {
  name = "seed-finder";
  root = pkgs.lib.sourceFilesBySuffices ./. [".rs" ".toml" ".lock" ".html" ".css" ".png" ".sh" ".sql"];
  buildInputs = with pkgs; [ openssl pkgconfig clang llvm llvmPackages.libclang zlib cacert ];
  LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
  OPENSSL_DIR = "${merged-openssl}";
}
