use clap::builder::TypedValueParser as _;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of BitcoinNetwork
    #[arg(
        short,
        long,
        default_value_t = parser::BitcoinNetwork::Mainnet,
        value_parser = clap::builder::PossibleValuesParser::new(["mainnet","testnet","regtest","signet"])
            .map(|s| s.parse::<parser::BitcoinNetwork>().unwrap()),
    )]
    network_name: parser::BitcoinNetwork,
}

mod parser {
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub enum BitcoinNetwork {
        Mainnet,
        Testnet,
        Regtest,
        Signet,
    }

    impl std::fmt::Display for BitcoinNetwork {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let s = match self {
                Self::Mainnet => "mainnet",
                Self::Testnet => "testnet",
                Self::Regtest => "regtest",
                Self::Signet => "signet",
            };
            s.fmt(f)
        }
    }

    impl std::str::FromStr for BitcoinNetwork {
        type Err = String;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "mainnet" => Ok(Self::Mainnet),
                "testnet" => Ok(Self::Testnet),
                "regtest" => Ok(Self::Regtest),
                "signet" => Ok(Self::Signet),
                _ => Err(format!("Unknown butcoin network: {s}")),
            }
        }
    }
}

pub fn get_bitcoin_network() -> rgb_lib::BitcoinNetwork {
    let args = Args::parse();

    if args.network_name == parser::BitcoinNetwork::Mainnet {
        rgb_lib::BitcoinNetwork::Mainnet
    } else if args.network_name == parser::BitcoinNetwork::Testnet {
        rgb_lib::BitcoinNetwork::Testnet
    } else if args.network_name == parser::BitcoinNetwork::Regtest {
        rgb_lib::BitcoinNetwork::Regtest
    } else if args.network_name == parser::BitcoinNetwork::Signet {
        rgb_lib::BitcoinNetwork::Signet
    } else {
        panic!("Internal error: an wrong args.network_name should be checked in the parser.")
    }
}
