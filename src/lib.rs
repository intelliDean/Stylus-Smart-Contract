// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

pub mod ERC20;
pub mod utility;

use crate::utility::Erc20Error::*;
use crate::utility::*;
use crate::ERC20::{Token, Immutables};
use alloy_sol_types::SolValue;
use stylus_sdk::prelude::*;
use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, U256, U64},
    crypto::keccak,
    prelude::*,
};

/// Immutable definitions
pub struct DegenParams;
impl Immutables for DegenParams {
    const NAME: &'static str = "DegenToken";
    const SYMBOL: &'static str = "DGT";
    const DECIMALS: u8 = 18;
}

sol_storage! {
    #[entrypoint]
   pub struct Degen {
        address owner;
        Player[] all_players;

        mapping(address => Player) players;
        mapping(bytes32 => GameProp) game_props;
        mapping(address => mapping(bytes32 => GameProp)) player_props;

         #[borrow]
        Token<DegenParams> erc20;
    }

    struct Player {
        address player_id;
        string player_nick;
        uint256 register_at;
        uint64 score;
        bool is_registered;
    }

     struct GameProp {
        address current_owner;
        bytes32 prop_id;
        string prop_name;
        uint256 worth;
    }
}

#[public]
impl Degen {
    #[constructor]
    // #[payable]
    pub fn constructor(&mut self) {
        let owner = self.vm().tx_origin();

        self.owner.set(owner);
        self.erc20.set_owner(owner);
    }

    fn player_register(&mut self, username: String) -> Result<(), Erc20Error> {
        self.address_zero_check()?;

        let caller = self.vm().msg_sender();

        if self.players.setter(caller).is_registered.get() {
            return Err(Registered(ALREADY_REGISTERED {}));
        }

        if caller != self.owner.get() {
            return Err(CannotRegister(OWNER_CANNOT_REGISTER {}));
        }

        let time = self.vm().block_timestamp();

        let mut player = self.players.setter(caller);

        player.player_id.set(caller);
        player.player_nick.set_str(username);
        player.register_at.set(U256::from(time));
        player.score.set(U64::from(0));
        player.is_registered.set(true);

        stylus_sdk::evm::log(PlayerRegisters {
            player: caller,
            success: true,
        });

        Ok(())
    }

    fn play_game(&mut self) -> Result<(), Erc20Error> {
        let current_score = self.players.setter(self.vm().msg_sender()).score.get();

        self.players
            .setter(self.vm().msg_sender())
            .score
            .set(current_score + U64::from(1));

        Ok(())
    }

    fn address_zero_check(&self) -> Result<(), Erc20Error> {
        let caller = self.vm().msg_sender();
        if caller.is_zero() {
            return Err(AddressZero(ADDRESS_ZERO { zero: caller }));
        }
        Ok(())
    }

    fn reg_check(&mut self) -> Result<(), Erc20Error> {
        if !self
            .players
            .setter(self.vm().msg_sender())
            .is_registered
            .get()
        {
            return Err(NotRegistered(YOU_ARE_NOT_REGISTERED {}));
        }
        Ok(())
    }

    fn only_owner(&self) -> Result<(), Erc20Error> {
        if self.vm().msg_sender() != self.owner.get() {
            return Err(OnlyOwner(ONLY_OWNER {}));
        }

        Ok(())
    }



    fn distribute_reward_to_players(&mut self) -> Result<(), Erc20Error> {
        self.only_owner()?;

        let num_of_players = self.all_players.len();

        if num_of_players < 1 {
            return Err(NoPlayer(N0_PLAYERS_TO_REWARD {}));
        }

        let mut total_rewards = U64::from(0);
        let mut amount = U64::from(0);

        for i in 0..num_of_players {
            let player = self
                .players
                .setter(self.all_players.get(i).unwrap().player_id.get());

            let current_score = player.score.get();
            if current_score <= U64::from(10) {
                amount = current_score * U64::from(3);
                self.erc20
                    .mint(player.player_id.get(), U256::from(amount))?;
                total_rewards += amount;
            } else if current_score > U64::from(10) && current_score <= U64::from(50) {
                amount = current_score * U64::from(5);
                self.erc20
                    .mint(player.player_id.get(), U256::from(amount))?;
                total_rewards += amount;
            } else {
                amount = current_score * U64::from(10);
                self.erc20
                    .mint(player.player_id.get(), U256::from(amount))?;
                total_rewards += amount;
            }
        }

        stylus_sdk::evm::log(RewardDistributed {
            totalRewards: U256::from(total_rewards),
            numberOfPlayers: U256::from(num_of_players),
        });

        Ok(())
    }

    fn player_p2p_transfer(&mut self, recipient: Address, value: U256) -> Result<(), Erc20Error> {
        self.address_zero_check()?;
        self.reg_check()?;

        if self.erc20.transfer(recipient, value)? {
            stylus_sdk::evm::log(PlayerP2P {
                sender: self.vm().msg_sender(),
                recipient,
                amount: value,
            });
        } else {
            return Err(TransferFailed(TRANSFER_FAILED {}));
        }

        Ok(())
    }

    fn suspend_player(&mut self, address: Address) -> Result<(), Erc20Error> {
        self.only_owner()?;

        let mut player = self.players.setter(address);

        if player.player_id.get().is_zero() || !player.is_registered.get() {
            return Err(NotFound(NOT_FOUND {}));
        }

        player.is_registered.set(false);

        Ok(())
    }
    fn reinstate_player(&mut self, address: Address) -> Result<(), Erc20Error> {
        self.only_owner()?;

        let mut player = self.players.setter(address);

        if player.player_id.get().is_zero() {
            return Err(NotFound(NOT_FOUND {}));
        } else if player.is_registered.get() {
            return Err(NotSuspended(PLAYER_NOT_SUSPENDED {}));
        } else {
            player.is_registered.set(true);
        }

        Ok(())
    }

    fn player_check_balance(&self) -> U256 {
        self.erc20.balance_of(self.vm().msg_sender())
    }

    fn player_burn_token(&mut self, value: U256) -> Result<(), Erc20Error> {
        self.reg_check()?;

        self.erc20.burn(value)?;

        Ok(())
    }

    fn add_game_prop(&mut self, name: String, value: U256) -> Result<(), Erc20Error> {
        self.only_owner()?;

        let contract_addr = self.vm().contract_address();

        type TxIdHashType = (String, U256);
        let tx_hash_data = (name.clone(), value);
        let tx_hash_data_encode_packed = TxIdHashType::abi_encode_packed(&tx_hash_data);

        let prop_id: FixedBytes<32> = keccak(tx_hash_data_encode_packed).into();

        let mut game_prop = self.game_props.setter(prop_id);

        game_prop.prop_name.set_str(name);
        game_prop.prop_id.set(prop_id);
        game_prop.worth.set(value);
        game_prop.current_owner.set(contract_addr);

        Ok(())
    }
    fn player_buys_from_game_store(&mut self, prop_id: FixedBytes<32>) -> Result<(), Erc20Error> {
        self.reg_check()?;

        let contract_addr = self.vm().contract_address();

        let caller = self.vm().msg_sender();

        let mut game_prop = self.game_props.setter(prop_id);

        if game_prop.current_owner.get().is_zero() {
            return Err(NotFound(NOT_FOUND {}));
        }

        let prop_value = game_prop.worth.get();

        if self.erc20.balance_of(caller) < prop_value {
            return Err(Insufficient(INSUFFICIENT_BALANCE {}));
        }

        if self.erc20.transfer(contract_addr, prop_value)? {
            game_prop.current_owner.set(caller);

            let mut player_prop_map = self.player_props.setter(caller);
            let mut prop = player_prop_map.setter(prop_id);
            prop.current_owner.set(caller);
            prop.prop_id.set(prop_id);

            let prop_name = game_prop.prop_name.get_string();

            prop.prop_name.set_str(prop_name.clone());
            prop.worth.set(game_prop.worth.get());

            stylus_sdk::evm::log(PropBought {
                newOwner: caller,
                _propId: prop_id,
                propName: prop_name,
            });
        }

        Ok(())
    }
}


//contract assress: 0xa6febd4225232e71a6a46209adb46fd3de1f5bda