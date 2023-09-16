# seed-finder

That is small program that iterates over words of partially known seed phrase. If you lost 1-2 words in your seed
you can restore it with that program.

Requirements:
* Local electrum server deployed. For instance, [electrs](https://github.com/romanz/electrs). If you use remote instance, you will wait for eternetiy even on 2 words.
* You can build either by [nix](https://nixos.org/) or manually with [rustup](https://rustup.rs/). There are no binaries prebuilt and you don't want to stick your private keys into wild binaries.
* You have the last word of the seed available as it used as checksum and makes the seed actually checkable.
* Your wallet is native segwit with derivation path `m/84'/0'/0'/0/*` or `m/84'/1'/0'/0/*`, if not, adjust the code accordingly.

# Building with Nix

```
nix-shell
export SEED="your seed here mark the missing words with * like this * * *"
cargo run --release 
```

You will see (maybe) restored seed in `found.txt`.

Cleanup you bash history from the seed after all.
