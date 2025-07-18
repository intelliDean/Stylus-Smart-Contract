use alloy_sol_types::sol;
use stylus_sdk::prelude::*;

sol! {
    error InsufficientBalance(address from, uint256 have, uint256 want);
    error InsufficientAllowance(address owner, address spender, uint256 have, uint256 want);
    error ONLY_OWNER();
    error ADDRESS_ZERO(address zero);
    error YOU_ARE_NOT_REGISTERED();
    error ALREADY_REGISTERED();
    error OWNER_CANNOT_REGISTER();
    error N0_PLAYERS_TO_REWARD();
    error TRANSFER_FAILED();
    error NOT_FOUND();
    error PLAYER_NOT_SUSPENDED();
    error INSUFFICIENT_BALANCE();

    event PlayerRegisters(address player, bool success);
    event RewardDistributed(uint256 totalRewards, uint256 numberOfPlayers);
    event PlayerP2P(address sender, address recipient, uint256 amount);
    event TokenBurnt(address owner, uint256 _amount);
    event PropCreated(address currentOwner, string _propName, bytes32 propId, uint256 _worth);
    event PropBought(address newOwner, bytes32 _propId, string propName);
}

#[derive(SolidityError)]
pub enum Erc20Error {
    InsufficientBalance(InsufficientBalance),
    InsufficientAllowance(InsufficientAllowance),
    OnlyOwner(ONLY_OWNER),
    AddressZero(ADDRESS_ZERO),
    NotRegistered(YOU_ARE_NOT_REGISTERED),
    CannotRegister(OWNER_CANNOT_REGISTER),
    Registered(ALREADY_REGISTERED),
    NoPlayer(N0_PLAYERS_TO_REWARD),
    TransferFailed(TRANSFER_FAILED),
    NotFound(NOT_FOUND),
    NotSuspended(PLAYER_NOT_SUSPENDED),
    Insufficient(INSUFFICIENT_BALANCE)
}
