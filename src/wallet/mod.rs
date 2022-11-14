use rgb_lib::wallet::{Wallet, WalletData};

#[allow(dead_code)]
pub struct WalletState {
    wallet_data: Option<WalletData>,
}

impl WalletState {
    pub fn new() -> Self {
        WalletState { wallet_data: None }
    }

    #[allow(dead_code)]
    pub fn update(&mut self, pubkey: String, mnemonic: String) -> WalletData {
        let base_data = shiro_backend::opts::get_wallet_data();
        let wallet_data = WalletData {
            data_dir: base_data.data_dir,
            bitcoin_network: base_data.bitcoin_network,
            database_type: base_data.database_type,
            pubkey,
            mnemonic: Some(mnemonic),
        };
        if Wallet::new(wallet_data.clone()).is_ok() {
            self.wallet_data = Some(wallet_data.clone());
        };
        wallet_data
    }

    #[allow(dead_code)]
    pub fn exists(&self) -> bool {
        self.wallet_data.is_some()
    }
}
