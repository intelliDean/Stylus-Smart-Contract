// Copyright 2025, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

// Imported packages

use alloc::string::String;
use crate::utility::{Erc20Error, Erc20Error::*, InsufficientAllowance, InsufficientBalance, ADDRESS_ZERO, ONLY_OWNER};
use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use core::marker::PhantomData;
use stylus_sdk::prelude::*;

pub trait Erc20Params {
    const NAME: &'static str;
    const SYMBOL: &'static str;
    const DECIMALS: u8;
}

sol! {
    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);
}

sol_storage! {
    pub struct Erc20<T> {

        address owner;

        mapping(address => uint256) balances;
        
        mapping(address => mapping(address => uint256)) allowances;
        
        uint256 total_supply;
        
        /// Used to allow [`Erc20Params`]
        PhantomData<T> phantom;
    }
}


#[public]
impl<T: Erc20Params> Erc20<T> {
    pub fn name() -> String {

        T::NAME.into()
    }
    pub fn symbol() -> String {
        T::SYMBOL.into()
    }

    pub fn decimals() -> u8 {
        T::DECIMALS
    }

    pub fn total_supply(&self) -> U256 {
        self.total_supply.get()
    }

    pub fn balance_of(&self, owner: Address) -> U256 {
        self.balances.get(owner)
    }

    pub fn mint(&mut self, address: Address, value: U256) -> Result<(), Erc20Error> {

        self.only_owner()?;
        Erc20::<T>::address_zero_check(&address)?;
        
        let mut balance = self.balances.setter(address);
        let new_balance = balance.get() + value;
        balance.set(new_balance);

        
        self.total_supply.set(self.total_supply.get() + value);

        stylus_sdk::evm::log(
            Transfer {
                from: Address::ZERO,
                to: address,
                value,
            },
        );

        Ok(())
    }

    pub fn transfer(&mut self, to: Address, value: U256) -> Result<bool, Erc20Error> {
        self._transfer(self.vm().msg_sender(), to, value)?;
        Ok(true)
    }
    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Erc20Error> {
        
        let msg_sender = self.vm().msg_sender();

        Erc20::<T>::address_zero_check(&msg_sender)?;
        Erc20::<T>::address_zero_check(&to)?;

        let mut sender_allowances = self.allowances.setter(from);

        let mut allowance = sender_allowances.setter(msg_sender);
        
        let old_allowance = allowance.get();
        if old_allowance < value {
            return Err(InsufficientAllowance(InsufficientAllowance {
                owner: from,
                spender: msg_sender,
                have: old_allowance,
                want: value,
            }));
        }

        allowance.set(old_allowance - value);

        self._transfer(from, to, value)?;

        Ok(true)
    }
    pub fn approve(&mut self, spender: Address, value: U256) -> bool {

        Erc20::<T>::address_zero_check(&spender);

        let msg_sender = self.vm().msg_sender();

        self.allowances.setter(msg_sender).insert(spender, value);
        
        stylus_sdk::evm::log(
            Approval {
                owner: msg_sender,
                spender,
                value,
            },
        );
        true
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.allowances.getter(owner).get(spender)
    }

    pub fn burn(&mut self, value: U256) -> Result<(), Erc20Error> {

        let owner = self.vm().msg_sender();
        
        let mut balance = self.balances.setter(owner);
        let old_balance = balance.get();
        if old_balance < value {
            return Err(InsufficientBalance(InsufficientBalance {
                from: owner,
                have: old_balance,
                want: value,
            }));
        }
        balance.set(old_balance - value);

        self.total_supply.set(self.total_supply.get() - value);

        stylus_sdk::evm::log(
            Transfer {
                from: owner,
                to: Address::ZERO,
                value,
            },
        );

        Ok(())
    }
}


impl<T: Erc20Params> Erc20<T> {

    pub fn set_owner(&mut self, _owner: Address) {
        self.owner.set(_owner);
    }

    fn only_owner(&self) -> Result<(), Erc20Error> {
        if self.vm().msg_sender() != self.owner.get() {
            return Err(OnlyOwner(ONLY_OWNER {}));
        }
        Ok(())
    }

    fn address_zero_check(addr: &Address) -> Result<(), Erc20Error> {
        if addr.is_zero() {
            return Err(AddressZero(ADDRESS_ZERO { zero: *addr }));
        }
        Ok(())
    }
    pub fn _transfer(&mut self, from: Address, to: Address, value: U256) -> Result<(), Erc20Error> {

        Erc20::<T>::address_zero_check(&self.vm().msg_sender())?;
        Erc20::<T>::address_zero_check(&to)?;

        let mut sender_balance = self.balances.setter(from);

        let old_sender_balance = sender_balance.get();
        if old_sender_balance < value {
            return Err(InsufficientBalance(InsufficientBalance {
                from,
                have: old_sender_balance,
                want: value,
            }));
        }
        sender_balance.set(old_sender_balance - value);

        let mut to_balance = self.balances.setter(to);
        let new_to_balance = to_balance.get() + value;
        to_balance.set(new_to_balance);

        stylus_sdk::evm::log( Transfer { from, to, value });
        Ok(())
    }
}