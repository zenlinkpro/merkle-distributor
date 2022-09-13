# Overview
 Merkle-Distributor is used to send rewards to users in batches. We use user data as the leaves of the merkle tree.  
 Anyone can use the data of a certain user and its path in the merkle tree to calculate a merkle tree root.  
 This is to prove the legitimacy of this user and the reward he deserves.
    
# Steps to use
1. Prepare the json file of the airdrop information, refer to the format merkle-distributor/scripts/scripts/example.json  
json is a KV array. k: address of the airdrop; v: number of airdrops in hexadecimal, and no 0x prefix.
2. Generate the merkle tree  
Run ts-node scripts/generate-merkle-root.ts --input scripts/example.json  
You get the following json  
    merkleRoot: the root of the merkle tree   
    tokenTotal: the total number of airdrops  
    claims: information about the user who received the airdrop.
```
{
    "merkleRoot": "0xc5a4b4dbe724bfb5aac5879fa145e98686e3e77aacacfc7e6dbea5daa587af3f",
    "tokenTotal": "0x71ac",
    "claim":{
        "bixYxFRJkFMwMRnSCH9GQbYsurmL6n88eKnh6ron8ZvgwTY":{
            "index":0,
            "amount": "0x0123".
            "proof":[
                "0x532271c512903744ec774b8fdeceabbd06afaf2e1b0db9e342656ee31fd7ab3b",
                "0xdcf8b07aa4773863e0002159afe7ddc583a18f83848ebe93788ee03d664fe949",
                "0x4dc2a86348200da3469bbf96d46ab71d528b8604c031e0e2cd44955e849b59a3",
                "0x1f2f61c2ab4cb554103ea4b0e65b1d2401b24f781adbc882091a6a428218e68c",
                "0x1d880e0b5495f6ee079b10338cd765ce98cbaa6677f20ea226a78bbb9580d232",
                "0xd00c62f1eed832299152913784b9505f8ae30e67ac07b25f63074c1e0977e430",
                "0x86ceded7066a10bdb521f5d2d28c037565f85e17e411019e88f4337140ac3b97"
            ]
        },
        "bmtJRHykbp8zN33Hd58poQjMrPXaUiJTkY4XYJvxuz3yTQV": {
            "index":1,
            "amount": "0x0123",
            "proof":[
                "0xfb4c1fdb961b33fe34628c4a3a99f05d26c06f053000f0eab04ddd2b7857b29d",
                "0xdb9586d9476f100d3d63c9fd04925abe451eee1416358de45576cedce9c7b197",
                "0x0564e3219c5663052dbc56d34a194628e134eb3852025202acacfa5be20995a2",
                "0x246dcb49ecfe475d689d26a428d7904a28689c72fb35229ac5484ea9b08baefb"
            ]
        },
        ...
    }
}
```
3. Create merkle distributors

- Permission Management  
Currently we have changed the permissions for creating merekle Distributor. 
Only users who are in the whitelist can create them. 
You can join the whitelist by calling the following interface via democratic referendum.
````
        Public add_to_create_whitelist(
            Origin. OriginFor<T>,
            Account. AccountIdOf<T>,
        )
```
- Create

    ```
        /// `create_merkle_distributor` will create a merkle distributor.
        /// Allows the specified user to claim assets.
        ///
        /// The dispatch source for this call must be the root ``Signed''.
        ///
        /// - `merkle_root`: The merekle root in the above json file.
        /// - `description`: A short description of this json file, no more than 20 characters.
        /// - `distribute_currency`: the merekle token type, .
        /// - `distribute_amount`: the total amount of the airdrop.
        #[pallet::weight(T::WeightInfo::create_merkle_distributor()])
        pub fn create_merkle_distributor(
            origin: OriginFor<T>,
            merkle_root: H256,
            Description. vec<u8>.
            distribution_currency: T::CurrencyId,
            distribution_amount: T::balance.
        ) 
    ```

4. Charge  
    Query the ID of the created merkleDistributor in the chain state merkleDistributor::merkleDistributorMetadata  
    Call the following interface with an account that has enough assets
```
        /// Collects currency from the merkle distributor's account.
        ///
        /// `merkle_distributor_id`: The ID of the Merkle distributor.
        #[pallet::weight(T::WeightInfo::charge()]]
        #[transactional]
        pub fn charge(
            origin: OriginFor<T>,
            merkle_distributor_id: T::MerkleDistributorId,
        )
```
5. Claim airdrops  
    The following parameters correspond to the information in the claims section of the json file. An account can claim airdrops for other accounts.

        /// ``claim`` Claims the reward by user information and Merkle proof.
        ////
        //// - `merkle_distributor_id`: ID of the merkle distributor.
        /// - `index`: index of the merkle tree leaf.
        /// - `account`: The account of the owner of the merkle proof. It accepts rewards.
        /// - `merkle_proof`: hash with merkle tree leaves to get the merkle tree root.
        #[pallet::weight(T::WeightInfo::claim()]
        #[transactional]
        pub fn claim(
            origin: OriginFor<T>,
            merkle_distributor_id: T::MerkleDistributorId,
            index: u32,
            Account. <T::Lookup as StaticLookup>::Source,
            amount: u128,
            merkle_proof. vec<H256>.
        )
```

6. Query claim status
```
    index is the index in the claims corresponding to each user in the generated json file
    claimed_word_index: u32 = index / 32;
    claimed_bit_index = index % 32

    Check the chain status 
    claimed_word = merkleDistributor::claimedBitmap(merkle_distributor_id, claimed_word_index);

    mask: u32 = 1 << claimed_bit_index;
    
    whether to claim: claimed_word & mask == mask
```