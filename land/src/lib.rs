use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::ValidAccountId;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, log, near_bindgen, AccountId, Balance, BorshStorageKey, Gas, PanicOnDefault,
    Promise, PromiseOrValue, Timestamp,
};
use std::convert::TryFrom;

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct NFTLand {
    owner_id: AccountId,
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    cities: UnorderedMap<String, CityMetadata>,
    lands: UnorderedMap<TokenId, LandMetadata>,
}

const MINT_FEE: Balance = 1_000_000_000_000_000_000_000_0;
const PREPARE_GAS: Gas = 1_500_000_000_000_0;
const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 288 288'%3E%3Cg id='l' data-name='l'%3E%3Cpath d='M187.58,79.81l-30.1,44.69a3.2,3.2,0,0,0,4.75,4.2L191.86,103a1.2,1.2,0,0,1,2,.91v80.46a1.2,1.2,0,0,1-2.12.77L102.18,77.93A15.35,15.35,0,0,0,90.47,72.5H87.34A15.34,15.34,0,0,0,72,87.84V201.16A15.34,15.34,0,0,0,87.34,216.5h0a15.35,15.35,0,0,0,13.08-7.31l30.1-44.69a3.2,3.2,0,0,0-4.75-4.2L96.14,186a1.2,1.2,0,0,1-2-.91V104.61a1.2,1.2,0,0,1,2.12-.77l89.55,107.23a15.35,15.35,0,0,0,11.71,5.43h3.13A15.34,15.34,0,0,0,216,201.16V87.84A15.34,15.34,0,0,0,200.66,72.5h0A15.35,15.35,0,0,0,187.58,79.81Z'/%3E%3C/g%3E%3C/svg%3E";

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    CityMetadata,
    LandMetadata,
    // CitiesPerOwner { account_hash: Vec<u8> },
}

#[near_bindgen]
impl NFTLand {
    #[init]
    pub fn new_default_meta(owner_id: String) -> Self {
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
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
    pub fn new(owner_id: String, metadata: NFTContractMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        Self {
            owner_id: owner_id.clone(),
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                ValidAccountId::try_from(env::current_account_id()).unwrap(),
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            cities: UnorderedMap::new(StorageKey::CityMetadata),
            lands: UnorderedMap::new(StorageKey::LandMetadata),
        }
    }

    pub fn open_area(
        &mut self,
        // owner_id: AccountId,
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

        let name_hash = env::sha256(name.as_bytes());
        let name_hash_str = hex::encode(&name_hash);

        // let mut city_ids = self.cities_per_owner.get(&owner_id).unwrap_or_else(|| {
        //     UnorderedSet::new(StorageKey::CitiesPerOwner {
        //         account_hash: env::sha256(owner_id.as_bytes()),
        //     })
        // });

        // city_ids.insert(&name_hash_str);

        // self.cities_per_owner.insert(&owner_id, &city_ids);

        self.cities.insert(
            &name_hash_str.clone(),
            &CityMetadata {
                city_id: name_hash_str,
                city_name: name,
                limit: limit,
                land_sold: 0u64, // required
                land_price: price.parse().unwrap(),
                open_time: open_time,
                close_time: close_time,
            },
        );
    }

    pub fn get_area(&self, name: String) -> Option<CityMetadata> {
        let name_hash = env::sha256(name.as_bytes());
        let name_hash_str = hex::encode(&name_hash);

        self.cities.get(&name_hash_str)
    }

    #[payable]
    pub fn buy_land(&mut self, name: String) {
        let city_raw = self.get_area(name.clone());

        assert!(city_raw != None, "City no exist.");
        let mut city = city_raw.unwrap();

        assert!(
            env::block_timestamp() > city.open_time,
            "This city has not started selling lands yet"
        );
        log!(
            "{}",
            format!(
                "close time: {}, time block: {}",
                city.close_time,
                env::block_timestamp()
            )
        );
        // assert!(
        //     env::block_timestamp() < city.close_time,
        //     "This city has ended lands sales"
        // );
        assert!(city.land_sold < city.limit, "All lands are sold out");
        assert!(
            env::attached_deposit() == city.land_price + MINT_FEE,
            "Please deposit exactly price of land"
        );

        let new_name = name.clone() + "#" + &city.land_sold.to_string();

        let token_id = env::sha256(new_name.as_bytes());
        let new_token_id = hex::encode(&token_id);
        ex_self::nft_private_mint(
            new_token_id.clone(),
            ValidAccountId::try_from(env::predecessor_account_id()).unwrap(),
            &env::current_account_id(),
            MINT_FEE,
            PREPARE_GAS,
        );
        let name_hash = env::sha256(name.as_bytes());
        let name_hash_str = hex::encode(&name_hash);

        city.land_sold += 1;
        self.cities.insert(&name_hash_str, &city);

        let land: LandMetadata = LandMetadata {
            city_id: name_hash_str,
            land_id: new_token_id.clone(),
            name: new_name,
            image: "https://res.cloudinary.com/dcrbaasbt/image/upload/v1637838225/257513804_224195603181581_4280639743210185776_n_nwqzoz.png".to_string()
        };

        self.lands.insert(&new_token_id, &land);
    }

    #[payable]
    #[private]
    pub fn nft_private_mint(&mut self, token_id: TokenId, receiver_id: ValidAccountId) -> Token {
        self.tokens.mint(
            token_id,
            receiver_id,
            Some(TokenMetadata {
                title: None,
                description: None,
                media: None,
                media_hash: None,
                copies: None,
                issued_at: Some(env::block_timestamp().to_string()),
                expires_at: None, // ISO 8601 datetime when token expires
                starts_at: None,  // ISO 8601 datetime when token starts being valid
                updated_at: None, // ISO 8601 datetime when token was last updated
                extra: None,
                reference: None,
                reference_hash: None,
            }),
        )
    }

    pub fn get_land(&self, name: String) -> Option<LandMetadata> {
        let hash = env::sha256(name.as_bytes());
        let hash_str = hex::encode(&hash);

        self.lands.get(&hash_str)
    }

    pub fn get_lands_by_owner(&self, owner_id: AccountId) -> Vec<LandMetadata> {
        let token_ids = self
            .tokens
            .tokens_per_owner
            .as_ref()
            .unwrap()
            .get(&owner_id)
            .unwrap_or_else(|| UnorderedSet::new(b"".to_vec()));
        token_ids
            .iter()
            .map(|token_id| self.lands.get(&token_id).unwrap())
            .collect()
    }

    // pub fn get_cites_by_owner(&self, owner_id: AccountId) -> Vec<CityMetadata> {
    //     let token_ids = self
    //         .cities_per_owner
    //         .get(&owner_id)
    //         .unwrap_or_else(|| UnorderedSet::new(b"c".to_vec()));
    //     token_ids
    //         .iter()
    //         .map(|token_id| self.cities.get(&token_id).unwrap())
    //         .collect()
    // }

    pub fn get_all_areas(&self) -> Vec<CityMetadata> {
        self.cities.values().collect()
    }

    pub fn get_active_area(&self) -> Vec<CityMetadata> {
        self.cities
            .values()
            .filter_map(|city| {
                if city.open_time < env::block_timestamp()
                    && city.close_time > env::block_timestamp()
                {
                    Some(city)
                } else {
                    None
                }
            })
            .collect()
    }
}

near_contract_standards::impl_non_fungible_token_core!(NFTLand, tokens);
near_contract_standards::impl_non_fungible_token_approval!(NFTLand, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(NFTLand, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for NFTLand {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct LandMetadata {
    pub land_id: String, // required
    pub city_id: String, // required,
    pub name: String,    // required
    pub image: String,   // required,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct CityMetadata {
    pub city_id: String,
    pub city_name: String,
    pub limit: u64,     // required, type ticket => amount
    pub land_sold: u64, // required
    pub land_price: Balance,
    pub open_time: Timestamp,  // required
    pub close_time: Timestamp, // required
}

#[ext_contract(ex_self)]
trait LandContract {
    fn nft_private_mint(&mut self, token_id: TokenId, receiver_id: ValidAccountId) -> Token;
}
