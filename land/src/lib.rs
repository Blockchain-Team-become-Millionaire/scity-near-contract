use near_sdk::Timestamp;
use std::cmp::min;
use std::collections::HashMap;
use std::convert::TryFrom;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, ValidAccountId, U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, AccountId, Balance, CryptoHash, PanicOnDefault, Promise, StorageUsage,
};

pub use crate::enumerable::*;
use crate::internal::*;
pub use crate::metadata::*;
pub use crate::mint::*;
pub use crate::nft_core::*;
pub use crate::token::*;

mod enumerable;
mod internal;
mod metadata;
mod mint;
mod nft_core;
mod token;

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,

    pub tokens_by_id: LookupMap<TokenId, Token>,

    pub token_metadata_by_id: UnorderedMap<TokenId, TokenMetadata>,

    pub owner_id: AccountId,

    /// The storage size in bytes for one account.
    pub extra_storage_in_bytes_per_token: StorageUsage,

    pub metadata: LazyOption<NFTMetadata>,

    pub area_metadata_by_id: UnorderedMap<String, AreaMetadata>,
}

/// Helper structure to for keys of the persistent collections.
#[derive(BorshSerialize)]
pub enum StorageKey {
    TokensPerOwner,
    TokenPerOwnerInner { account_id_hash: CryptoHash },
    TokensById,
    TokenMetadataById,
    AreaMetadataById,
    NftMetadata,
    TokensPerType,
    TokensPerTypeInner { token_type_hash: CryptoHash },
    TokenTypesLocked,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new_default_meta(owner_id: ValidAccountId) -> Self {
        Self::new(
            owner_id,
            NFTMetadata {
                spec: "nft-1.0.0".to_string(),
                name: "The metaverse".to_string(),
                symbol: "Land".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        )
    }
    #[init]
    pub fn new(owner_id: ValidAccountId, metadata: NFTMetadata) -> Self {
        let mut this = Self {
            tokens_per_owner: LookupMap::new(StorageKey::TokensPerOwner.try_to_vec().unwrap()),
            tokens_by_id: LookupMap::new(StorageKey::TokensById.try_to_vec().unwrap()),
            token_metadata_by_id: UnorderedMap::new(
                StorageKey::TokenMetadataById.try_to_vec().unwrap(),
            ),
            owner_id: owner_id.into(),
            extra_storage_in_bytes_per_token: 0,
            metadata: LazyOption::new(
                StorageKey::NftMetadata.try_to_vec().unwrap(),
                Some(&metadata),
            ),
            area_metadata_by_id: UnorderedMap::new(
                StorageKey::AreaMetadataById.try_to_vec().unwrap(),
            ),
        };

        this.measure_min_token_storage_cost();

        this
    }

    fn measure_min_token_storage_cost(&mut self) {
        let initial_storage_usage = env::storage_usage();
        let tmp_account_id = "a".repeat(64);
        let u = UnorderedSet::new(
            StorageKey::TokenPerOwnerInner {
                account_id_hash: hash_account_id(&tmp_account_id),
            }
            .try_to_vec()
            .unwrap(),
        );
        self.tokens_per_owner.insert(&tmp_account_id, &u);

        let tokens_per_owner_entry_in_bytes = env::storage_usage() - initial_storage_usage;
        let owner_id_extra_cost_in_bytes = (tmp_account_id.len() - self.owner_id.len()) as u64;

        self.extra_storage_in_bytes_per_token =
            tokens_per_owner_entry_in_bytes + owner_id_extra_cost_in_bytes;

        self.tokens_per_owner.remove(&tmp_account_id);
    }

    // View method
    pub fn get_area(&self, name: String) -> Option<AreaMetadata> {
        let hash = hex::encode(&env::sha256(name.as_bytes()));
        self.area_metadata_by_id.get(&hash)
    }

    // Call method
    pub fn open_area(
        &mut self,
        name: String,
        limit: u64,
        price: String,
        open_time: Timestamp,
        close_time: Timestamp,
    ) {
        assert!(
            env::predecessor_account_id() == self.owner_id,
            "Caller is not owner."
        );

        let hash = hex::encode(&env::sha256(name.as_bytes()));
        self.area_metadata_by_id.insert(
            &hash.clone(),
            &AreaMetadata {
                name: name,
                limit: limit,
                land_sold: 0u64,
                land_price: price.parse().unwrap(),
                open_time: open_time,
                close_time: close_time,
            },
        );
    }

    #[payable]
    pub fn buy_land(&mut self, name: String) {
        let area_data = self.get_area(name.clone());

        assert!(area_data != None, "Area no exist.");
        let mut area = area_data.unwrap();

        assert!(
            env::block_timestamp() > area.open_time,
            "This area has not started selling lands yet"
        );
        log!(
            "{}",
            format!(
                "close time: {}, time block: {}",
                area.close_time,
                env::block_timestamp()
            )
        );
        // assert!(
        //     env::block_timestamp() < area.close_time,
        //     "This area has ended lands sales"
        // );
        // assert!(area.land_sold < area.limit, "All lands are sold out");
        // assert!(
        //     env::attached_deposit() == area.land_price,
        //     "Please deposit exactly price of land"
        // );

        let new_name = name.clone() + "#" + &area.land_sold.to_string();
        let token_id = hex::encode(&env::sha256(new_name.as_bytes()));

        let area_hash = hex::encode(&env::sha256(name.as_bytes()));

        area.land_sold += 1;
        self.area_metadata_by_id.insert(&area_hash, &area);

        let token: TokenMetadata = TokenMetadata {
            title: Some(String::from(new_name.clone())),
            description: Some(String::from(new_name)),
            media: Some(String::from("https://res.cloudinary.com/dcrbaasbt/image/upload/v1637838225/257513804_224195603181581_4280639743210185776_n_nwqzoz.png")),
            media_hash: None,
            copies: Some(1),
            issued_at: Some(env::block_timestamp()),
            city: Some(name),
            location: Some(String::from("10, 20")),
            rare: Some(String::from("R")),
            mining_efficiency: Some(1),
            mining_power: Some(10),
        };

        self.nft_mint(
            Some(token_id),
            token,
            Some(HashMap::new()),
            Some(ValidAccountId::try_from(env::predecessor_account_id()).unwrap()),
        )
    }

    pub fn get_land(&self, name: String) -> Option<TokenMetadata> {
        let hash = hex::encode(&env::sha256(name.as_bytes()));
        self.token_metadata_by_id.get(&hash)
    }

    pub fn get_lands_by_owner(&self, owner_id: AccountId) -> Vec<TokenMetadata> {
        let token_ids = self
            .tokens_per_owner
            .get(&owner_id)
            .unwrap_or_else(|| UnorderedSet::new(b"".to_vec()));
        token_ids
            .iter()
            .map(|token_id| self.token_metadata_by_id.get(&token_id).unwrap())
            .collect()
    }

    pub fn get_all_areas(&self) -> Vec<AreaMetadata> {
        self.area_metadata_by_id.values().collect()
    }
}
