use clap::builder::TypedValueParser as _;
use clap::Parser;
use rgb_lib::wallet::WalletData;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to data_dir
    #[arg(long, default_value = "/tmp/shiro-wallet")]
    data_dir: String,

    /// Name of BitcoinNetwork
    #[arg(
        env = "BITCOIN_NETWORK_NAME",
        short,
        long,
        default_value_t = parser::BitcoinNetwork::Mainnet,
        value_parser = clap::builder::PossibleValuesParser::new(["mainnet","testnet","regtest","signet"])
            .map(|s| s.parse::<parser::BitcoinNetwork>().unwrap()),
    )]
    network_name: parser::BitcoinNetwork,

    #[arg(
        long,
        default_value_t = parser::DatabaseType::Sqlite,
        value_parser = clap::builder::PossibleValuesParser::new(["sqlite"])
            .map(|s| s.parse::<parser::DatabaseType>().unwrap()),
    )]
    database_type: parser::DatabaseType,

    #[arg(env = "ELECTRUM_URL",
          long = "electrum-url",
          default_value = "127.0.0.1:50001")]
    pub electrum_url: String,

    #[arg(env = "RGB_PROXY_URL",
          long = "proxy-url",
          default_value = "http://proxy.rgbtools.org")]
    pub proxy_url: String,

    #[arg(long = "show-output")]
    show_output: bool,

    #[arg(
        long,
        default_value_t = true
        )]
    pub skip_consistency_check: bool,

}

pub fn get_args() -> Args {
    Args::parse()
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

    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub enum DatabaseType {
        Sqlite,
    }

    impl std::fmt::Display for DatabaseType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let s = match self {
                Self::Sqlite => "sqlite",
            };
            s.fmt(f)
        }
    }

    impl std::str::FromStr for DatabaseType {
        type Err = String;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "sqlite" => Ok(Self::Sqlite),
                _ => Err(format!("Unknown database type: {s}")),
            }
        }
    }
}

pub fn get_bitcoin_network() -> rgb_lib::BitcoinNetwork {
    get_wallet_data().bitcoin_network
}

pub fn get_wallet_data() -> rgb_lib::wallet::WalletData {
    let args = Args::parse();

    let bitcoin_network = if args.network_name == parser::BitcoinNetwork::Mainnet {
        rgb_lib::BitcoinNetwork::Mainnet
    } else if args.network_name == parser::BitcoinNetwork::Testnet {
        rgb_lib::BitcoinNetwork::Testnet
    } else if args.network_name == parser::BitcoinNetwork::Regtest {
        rgb_lib::BitcoinNetwork::Regtest
    } else if args.network_name == parser::BitcoinNetwork::Signet {
        rgb_lib::BitcoinNetwork::Signet
    } else {
        panic!("Internal error: an wrong args.network_name should be checked in the parser.")
    };

    let database_type = if args.database_type == parser::DatabaseType::Sqlite {
        rgb_lib::wallet::DatabaseType::Sqlite
    } else {
        panic!("Internal error: an wrong args.database_type should be checked in the parser.")
    };

    WalletData {
        data_dir: args.data_dir,
        bitcoin_network,
        database_type,
        pubkey: "".to_string(),
        mnemonic: None,
    }
}

#[cfg(test)]
mod tests {
    use super::parser::*;

    #[test]
    fn test_database_type() {
        assert_eq!(format!("{}", DatabaseType::Sqlite), "sqlite")
    }

    #[test]
    fn test_bitcoinnetwork_mainnet() {
        assert_eq!(format!("{}", BitcoinNetwork::Mainnet), "mainnet")
    }

    #[test]
    fn test_bitcoinnetwork_testnet() {
        assert_eq!(format!("{}", BitcoinNetwork::Testnet), "testnet")
    }

    #[test]
    fn test_bitcoinnetwork_regtest() {
        assert_eq!(format!("{}", BitcoinNetwork::Regtest), "regtest")
    }

    #[test]
    fn test_bitcoinnetwork_signet() {
        assert_eq!(format!("{}", BitcoinNetwork::Signet), "signet")
    }
}
