use bdk::blockchain::ElectrumBlockchain;
use bdk::database::MemoryDatabase;
use bdk::electrum_client::Client;
use bdk::{SyncOptions, Wallet};
use bip39::{Language, Mnemonic, Seed};
use bitcoin::secp256k1::{All, Secp256k1};
use bitcoin::util::bip32::{ChildNumber, ExtendedPrivKey};
use bitcoin::Network;
use clap::Parser;
use futures::StreamExt;
use futures::TryStreamExt;
use itertools::free::join;
use lazy_static::lazy_static;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

lazy_static! {
    static ref SECP: Secp256k1<All> = Secp256k1::new();
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Partial seed phrase, use * for unknown words
    #[arg(short, long, env = "SEED")]
    seed: String,

    /// Is testnet network
    #[arg(short, long)]
    testnet: bool,

    /// Start from the given offset of guesses
    #[arg(short, long, default_value = "0")]
    offset: u128,

    /// Output file where to save found seed phrase
    #[arg(short, long, default_value = "found.txt")]
    outfile: PathBuf,

    /// Output file where to save checkpoints
    #[arg(short, long, default_value = "ssl://bitcoin.grey.pw:50002")]
    electrum_url: String,
}

struct PartialSeed {
    words: Vec<Option<String>>,
}

impl PartialSeed {
    fn new(seed: &str) -> Self {
        let words = seed
            .split(' ')
            .map(|w| if w == "*" { None } else { Some(w.to_owned()) })
            .collect();
        PartialSeed { words }
    }

    fn unknown_words(&self) -> usize {
        self.words.iter().filter(|w| w.is_none()).count()
    }

    fn guess_seed(&self, mut offset: u128, lang: Language) -> String {
        let mut words = self.words.clone();
        let wordlist = lang.wordlist();
        for word in words.iter_mut().filter(|w| w.is_none()) {
            let i = offset % 2048;
            offset /= 2048;
            *word = Some(wordlist.get_word((i as u16).into()).to_owned());
        }
        join(words.into_iter().flatten(), " ")
    }
}

async fn sync_wallet(electrum_url: &str, network: Network, seed: &[u8]) -> Result<u64, bdk::Error> {
    let priv_key = ExtendedPrivKey::new_master(network, seed)?;
    // println!("Private master key: {priv_key}");
    let derived_key = priv_key.derive_priv(
        &Secp256k1::new(),
        &vec![
            ChildNumber::Hardened { index: 84 },
            ChildNumber::Hardened {
                index: if network == Network::Bitcoin { 0 } else { 1 },
            },
            ChildNumber::Hardened { index: 0 },
        ],
    )?;

    let client = Client::new(electrum_url)?;
    let blockchain = ElectrumBlockchain::from(client);
    let wallet = Wallet::new(
        &format!("wpkh({derived_key}/0/*)"),
        Some(&format!("wpkh({derived_key}/1/*)")),
        network,
        MemoryDatabase::default(),
    )?;
    // let first_address = wallet.get_address(AddressIndex::Peek(0))?;
    // println!("First address: {}", first_address);

    wallet.sync(&blockchain, SyncOptions::default())?;
    let balance = wallet.get_balance()?;
    Ok(balance.confirmed)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let seed = PartialSeed::new(&args.seed);
    let language = Language::English;
    println!("First seed: {}", seed.guess_seed(args.offset, language));
    let unknown = seed.unknown_words();
    println!("Unknown words: {unknown}");
    let max_guesses = 2048u128.pow(unknown as u32);
    println!("Need to scan {} seeds", max_guesses - args.offset);

    futures::stream::iter(args.offset..max_guesses)
        .map(Ok)
        .try_for_each_concurrent(1000, |offset| {
            let outfile = args.outfile.clone();
            let guess = seed.guess_seed(offset, language);
            let electrum_url = args.electrum_url.clone();
            tokio::spawn(async move {
                if let Ok(valid_guess) = Mnemonic::from_phrase(&guess, language) {
                    println!("Valid guessed seed {offset} is: {valid_guess}");
                    let balance = sync_wallet(
                        &electrum_url,
                        if args.testnet {
                            Network::Testnet
                        } else {
                            Network::Bitcoin
                        },
                        Seed::new(&valid_guess, "").as_bytes(),
                    )
                    .await
                    .expect("Wallet");
                    if balance > 0 {
                        println!("Balance: {balance}");

                        let mut file = OpenOptions::new()
                            .write(true)
                            .append(true)
                            .create(true)
                            .open(&outfile)
                            .unwrap();
                        file.write_all(format!("{valid_guess}\n").as_bytes()).unwrap();

                        println!("WE FOUND IT! See the file {outfile:?}");
                        std::process::exit(0);
                    }
                }
            })
        })
        .await?;
    Ok(())
}
