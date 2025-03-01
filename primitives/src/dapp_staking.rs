// This file is part of Astar.

// Copyright (C) 2019-2023 Stake Technologies Pte.Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later

// Astar is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Astar is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Astar. If not, see <http://www.gnu.org/licenses/>.

use super::{Balance, BlockNumber};

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};

use frame_support::pallet_prelude::{RuntimeDebug, Weight};
use sp_core::H160;
use sp_std::hash::Hash;

/// Era number type
pub type EraNumber = u32;
/// Period number type
pub type PeriodNumber = u32;
/// Dapp Id type
pub type DAppId = u16;
/// Tier Id type
pub type TierId = u8;

/// Configuration for cycles, periods, subperiods & eras.
///
/// * `cycle` - Time unit similar to 'year' in the real world. Consists of one or more periods. At the beginning of each cycle, inflation is recalculated.
/// * `period` - Period consists of two distinct subperiods: `Voting` & `Build&Earn`. They are integral parts of dApp staking.
///              Length is expressed in standard eras or just _eras_.
/// * `era` - Era is the basic time unit in the dApp staking protocol. At the end of each era, reward pools for stakers & dApps are calculated.
///           Era length is expressed in blocks.
pub trait CycleConfiguration {
    /// How many different periods are there in a cycle (a 'year').
    ///
    /// This value has to be at least 1.
    fn periods_per_cycle() -> PeriodNumber;

    /// For how many standard era lengths does the voting subperiod last.
    ///
    /// This value has to be at least 1.
    fn eras_per_voting_subperiod() -> EraNumber;

    /// How many standard eras are there in the build&earn subperiod.
    ///
    /// This value has to be at least 1.
    fn eras_per_build_and_earn_subperiod() -> EraNumber;

    /// How many blocks are there per standard era.
    ///
    /// This value has to be at least 1.
    fn blocks_per_era() -> BlockNumber;

    /// For how many standard era lengths does the period last.
    fn period_in_era_lengths() -> EraNumber {
        Self::eras_per_voting_subperiod().saturating_add(Self::eras_per_build_and_earn_subperiod())
    }

    /// For how many standard era lengths does the cycle (a 'year') last.
    fn cycle_in_era_lengths() -> EraNumber {
        Self::period_in_era_lengths().saturating_mul(Self::periods_per_cycle())
    }

    /// How many blocks are there per cycle (a 'year').
    fn blocks_per_cycle() -> BlockNumber {
        Self::blocks_per_era().saturating_mul(Self::cycle_in_era_lengths())
    }

    /// For how many standard era lengths do all the build&earn subperiods in a cycle last.
    fn build_and_earn_eras_per_cycle() -> EraNumber {
        Self::eras_per_build_and_earn_subperiod().saturating_mul(Self::periods_per_cycle())
    }

    /// How many distinct eras are there in a single period.
    fn eras_per_period() -> EraNumber {
        Self::eras_per_build_and_earn_subperiod().saturating_add(1)
    }

    /// How many distinct eras are there in a cycle.
    fn eras_per_cycle() -> EraNumber {
        Self::eras_per_period().saturating_mul(Self::periods_per_cycle())
    }
}

/// Trait for observers (listeners) of various events related to dApp staking protocol.
pub trait Observer {
    /// Called in the block right before the next era starts.
    ///
    /// Returns the weight consumed by the call.
    ///
    /// # Arguments
    /// * `next_era` - Era number of the next era.
    fn block_before_new_era(_next_era: EraNumber) -> Weight {
        Weight::zero()
    }
}

impl Observer for () {}

/// Interface for staking reward handler.
///
/// Provides reward pool values for stakers - normal & bonus rewards, as well as dApp reward pool.
/// Also provides a safe function for paying out rewards.
pub trait StakingRewardHandler<AccountId> {
    /// Returns the staker reward pool & dApp reward pool for an era.
    ///
    /// The total staker reward pool is dynamic and depends on the total value staked.
    fn staker_and_dapp_reward_pools(total_value_staked: Balance) -> (Balance, Balance);

    /// Returns the bonus reward pool for a period.
    fn bonus_reward_pool() -> Balance;

    /// Attempts to pay out the rewards to the beneficiary.
    fn payout_reward(beneficiary: &AccountId, reward: Balance) -> Result<(), ()>;
}

/// Trait defining the interface for dApp staking `smart contract types` handler.
///
/// It can be used to create a representation of the specified smart contract instance type.
pub trait SmartContractHandle<AccountId> {
    /// Create a new smart contract representation for the specified EVM address.
    fn evm(address: H160) -> Self;
    /// Create a new smart contract representation for the specified Wasm address.
    fn wasm(address: AccountId) -> Self;
}

/// Multi-VM pointer to smart contract instance.
#[derive(
    PartialEq,
    Eq,
    Copy,
    Clone,
    Encode,
    Decode,
    RuntimeDebug,
    MaxEncodedLen,
    Hash,
    scale_info::TypeInfo,
)]
pub enum SmartContract<AccountId> {
    /// EVM smart contract instance.
    Evm(H160),
    /// Wasm smart contract instance.
    Wasm(AccountId),
}

// TODO: remove this once dApps staking v2 has been removed.
impl<AccountId> Default for SmartContract<AccountId> {
    fn default() -> Self {
        Self::evm([0x01; 20].into())
    }
}

impl<AccountId> SmartContractHandle<AccountId> for SmartContract<AccountId> {
    fn evm(address: H160) -> Self {
        Self::Evm(address)
    }

    fn wasm(address: AccountId) -> Self {
        Self::Wasm(address)
    }
}

/// Used to check whether an account is allowed to participate in dApp staking or not.
pub trait AccountCheck<AccountId> {
    /// `true` if the account is allowed to stake, `false` otherwise.
    fn allowed_to_stake(account: &AccountId) -> bool;
}

impl<AccountId> AccountCheck<AccountId> for () {
    fn allowed_to_stake(_account: &AccountId) -> bool {
        true
    }
}
