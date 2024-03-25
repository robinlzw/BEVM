// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use codec::Encode;
use sp_core::crypto::UncheckedFrom;
use sp_runtime::traits::{Hash, Saturating};

use xp_mining_common::{
    generic_weight_factors, BaseMiningWeight, Claim, ComputeMiningWeight, WeightFactors, WeightType,
};
use xp_mining_staking::MiningPower;

use super::*;

impl<'a, T: Config> BaseMiningWeight<BalanceOf<T>, T::BlockNumber> for AssetLedgerWrapper<'a, T> {
    fn amount(&self) -> BalanceOf<T> {
        xpallet_assets::Pallet::<T>::total_issuance(self.asset_id)
    }

    fn last_acum_weight(&self) -> WeightType {
        self.inner.last_total_mining_weight
    }

    fn set_last_acum_weight(&mut self, latest_mining_weight: WeightType) {
        self.inner.last_total_mining_weight = latest_mining_weight;
    }

    fn last_acum_weight_update(&self) -> T::BlockNumber {
        self.inner.last_total_mining_weight_update
    }

    fn set_last_acum_weight_update(&mut self, current_block: T::BlockNumber) {
        self.inner.last_total_mining_weight_update = current_block;
    }
}

impl<'a, T: Config> BaseMiningWeight<BalanceOf<T>, T::BlockNumber> for MinerLedgerWrapper<'a, T> {
    fn amount(&self) -> BalanceOf<T> {
        xpallet_assets::Pallet::<T>::all_type_asset_balance(self.miner, self.asset_id)
    }

    fn last_acum_weight(&self) -> WeightType {
        self.inner.last_mining_weight
    }

    fn set_last_acum_weight(&mut self, latest_mining_weight: WeightType) {
        self.inner.last_mining_weight = latest_mining_weight;
    }

    fn last_acum_weight_update(&self) -> T::BlockNumber {
        self.inner.last_mining_weight_update
    }

    fn set_last_acum_weight_update(&mut self, current_block: T::BlockNumber) {
        self.inner.last_mining_weight_update = current_block;
    }
}

impl<T: Config> ComputeMiningWeight<T::AccountId, T::BlockNumber> for Pallet<T> {
    type Claimee = AssetId;
    type Error = Error<T>;

    fn claimer_weight_factors(
        who: &T::AccountId,
        target: &Self::Claimee,
        current_block: T::BlockNumber,
    ) -> WeightFactors {
        let mut inner = MinerLedgers::<T>::get(who, target);
        let wrapper = MinerLedgerWrapper::<T>::new(who, target, &mut inner);
        generic_weight_factors::<BalanceOf<T>, T::BlockNumber, _>(wrapper, current_block)
    }

    fn claimee_weight_factors(
        target: &Self::Claimee,
        current_block: T::BlockNumber,
    ) -> WeightFactors {
        let mut inner = AssetLedgers::<T>::get(target);
        let wrapper = AssetLedgerWrapper::<T>::new(target, &mut inner);
        generic_weight_factors::<BalanceOf<T>, T::BlockNumber, _>(wrapper, current_block)
    }
}

// ChainX now uses pallet_balances for native coin PCX, therefore we do not
// have to exclude PCX asset in these OnAssetChanged methods:
//
// * `on_issue_pre`
// * `on_issue_post`
// * `on_move_pre`
//
// ```rust
// if xp_protocol::PCX == *target {
//     return Ok(());
// }
// ```
impl<T: Config> xpallet_assets::OnAssetChanged<T::AccountId, BalanceOf<T>> for Pallet<T> {
    fn on_issue_pre(target: &AssetId, source: &T::AccountId) {
        let current_block = <frame_system::Pallet<T>>::block_number();
        Self::init_receiver_mining_ledger(source, target, current_block);

        Self::update_mining_weights(source, target, current_block);
    }

    fn on_issue_post(
        target: &AssetId,
        source: &T::AccountId,
        _value: BalanceOf<T>,
    ) -> DispatchResult {
        Self::issue_deposit_reward(source, target)
    }

    fn on_move_pre(
        asset_id: &AssetId,
        from: &T::AccountId,
        _: AssetType,
        to: &T::AccountId,
        _: AssetType,
        _: BalanceOf<T>,
    ) {
        if from == to {
            return;
        }

        let current_block = <frame_system::Pallet<T>>::block_number();
        Self::init_receiver_mining_ledger(to, asset_id, current_block);

        Self::update_miner_mining_weight(from, asset_id, current_block);
        Self::update_miner_mining_weight(to, asset_id, current_block);
    }

    fn on_destroy_pre(target: &AssetId, source: &T::AccountId) {
        let current_block = <frame_system::Pallet<T>>::block_number();
        Self::update_mining_weights(source, target, current_block);
    }
}

impl<T: Config> Pallet<T> {
    /// Returns the tuple of (dividend, source_weight, target_weight, reward_pot_account).
    pub fn calculate_dividend_on_claim(
        claimer: &T::AccountId,
        claimee: &AssetId,
        block_number: T::BlockNumber,
    ) -> Result<(BalanceOf<T>, WeightType, WeightType, T::AccountId), Error<T>> {
        let reward_pot = T::DetermineRewardPotAccount::reward_pot_account_for(claimee);
        let reward_pot_balance = Self::free_balance(&reward_pot);

        let (dividend, source_weight, target_weight) =
            <Self as ComputeMiningWeight<T::AccountId, T::BlockNumber>>::compute_dividend(
                claimer,
                claimee,
                block_number,
                reward_pot_balance,
            )?;

        Ok((dividend, source_weight, target_weight, reward_pot))
    }

    /// Returns the dividend of `claimer` to `claimee` at `block_number`.
    pub fn compute_dividend_at(
        claimer: &T::AccountId,
        claimee: &AssetId,
        block_number: T::BlockNumber,
    ) -> Result<BalanceOf<T>, Error<T>> {
        Self::calculate_dividend_on_claim(claimer, claimee, block_number)
            .map(|(dividend, _, _, _)| dividend)
    }

    /// Allocates the dividend to claimer and referral(treasury) accordingly.
    ///
    /// Each asset miner can have a referral, which splits the 10% of
    /// of total asset mining dividend. The 10% split will be transferred
    /// to the treasury account if the claimer does not have a referral.
    ///
    /// total_asset_miner_dividend
    ///   ├──> referral(treasury) 10%
    ///   └──> claimer            90%
    fn allocate_dividend(
        claimee_reward_pot: &T::AccountId,
        claimer: &T::AccountId,
        claimee: &AssetId,
        dividend: BalanceOf<T>,
    ) -> Result<(), Error<T>> {
        let to_referral_or_treasury = dividend / 10u32.saturated_into();
        let reward_splitter =
            T::GatewayInterface::referral_of(claimer, *claimee).unwrap_or_else(|| {
                <T as Config>::TreasuryAccount::treasury_account()
                    .expect("TreasuryAccount is some; qed")
            });
        Self::transfer(
            claimee_reward_pot,
            &reward_splitter,
            to_referral_or_treasury,
        )?;

        let to_claimer = dividend - to_referral_or_treasury;
        Self::transfer(claimee_reward_pot, claimer, to_claimer)?;

        Ok(())
    }
}

impl<T: Config> Claim<T::AccountId> for Pallet<T> {
    type Claimee = AssetId;
    type Error = Error<T>;

    fn claim(claimer: &T::AccountId, claimee: &Self::Claimee) -> Result<(), Error<T>> {
        let current_block = <frame_system::Pallet<T>>::block_number();

        let ClaimRestriction {
            staking_requirement,
            frequency_limit,
        } = ClaimRestrictionOf::<T>::get(claimee);

        Self::passed_enough_interval(claimer, claimee, frequency_limit, current_block)?;

        let (dividend, source_weight, target_weight, claimee_reward_pot) =
            Self::calculate_dividend_on_claim(claimer, claimee, current_block)?;

        Self::has_enough_staking(claimer, dividend, staking_requirement)?;

        Self::allocate_dividend(&claimee_reward_pot, claimer, claimee, dividend)?;

        Self::apply_update_miner_mining_weight(claimer, claimee, 0, current_block);
        Self::apply_update_asset_mining_weight(
            claimee,
            target_weight - source_weight,
            current_block,
        );

        MinerLedgers::<T>::mutate(claimer, claimee, |miner_ledger| {
            miner_ledger.last_claim = Some(current_block);
        });

        Self::deposit_event(Event::<T>::Claimed(claimer.clone(), *claimee, dividend));

        Ok(())
    }
}

impl<T: Config> xpallet_assets_registrar::RegistrarHandler for Pallet<T> {
    fn on_register(asset_id: &AssetId, has_mining_rights: bool) -> DispatchResult {
        if !has_mining_rights {
            return Ok(());
        }
        MiningPrevilegedAssets::<T>::mutate(|i| i.push(*asset_id));
        AssetLedgers::<T>::insert(
            asset_id,
            AssetLedger {
                last_total_mining_weight_update: <frame_system::Pallet<T>>::block_number(),
                ..Default::default()
            },
        );
        Ok(())
    }

    fn on_deregister(asset_id: &AssetId) -> DispatchResult {
        MiningPrevilegedAssets::<T>::mutate(|v| {
            v.retain(|i| i != asset_id);
        });
        Ok(())
    }
}

/// Simple Asset reward pot account determiner.
///
/// Formula: `blake2_256(blake2_256(asset_id) + blake2_256(registered_block_number))`
pub struct SimpleAssetRewardPotAccountDeterminer<T: Config>(sp_std::marker::PhantomData<T>);

impl<T: Config> xp_mining_common::RewardPotAccountFor<T::AccountId, AssetId>
    for SimpleAssetRewardPotAccountDeterminer<T>
where
    T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
{
    fn reward_pot_account_for(asset_id: &AssetId) -> T::AccountId {
        let id_hash = T::Hashing::hash(&asset_id.to_le_bytes()[..]);
        let registered_block = <xpallet_assets_registrar::Pallet<T>>::registered_at(asset_id);
        let registered_block_hash =
            <T as frame_system::Config>::Hashing::hash(registered_block.encode().as_ref());

        let id_slice = id_hash.as_ref();
        let registered_slice = registered_block_hash.as_ref();

        let mut buf = Vec::with_capacity(id_slice.len() + registered_slice.len());
        buf.extend_from_slice(id_slice);
        buf.extend_from_slice(registered_slice);

        UncheckedFrom::unchecked_from(T::Hashing::hash(&buf[..]))
    }
}

impl<T: Config> xp_mining_staking::AssetMining<BalanceOf<T>> for Pallet<T> {
    /// Collects the mining power of all mining assets.
    fn asset_mining_power() -> Vec<(AssetId, MiningPower)> {
        // Currently only X-BTC asset.
        FixedAssetPowerOf::<T>::iter()
            .map(|(asset_id, fixed_power)| {
                let total_issuance = <xpallet_assets::Pallet<T>>::total_issuance(&asset_id);
                (
                    asset_id,
                    total_issuance
                        .saturating_mul(fixed_power.saturated_into())
                        .saturated_into::<MiningPower>(),
                )
            })
            .collect()
    }

    /// Issues reward to the reward pot of an Asset.
    fn reward(asset_id: AssetId, value: BalanceOf<T>) {
        let reward_pot = T::DetermineRewardPotAccount::reward_pot_account_for(&asset_id);
        <T as xpallet_assets::Config>::Currency::deposit_creating(&reward_pot, value);
        Self::deposit_event(Event::<T>::Minted(reward_pot, value));
    }
}

/*
这段代码是ChainX区块链项目的一部分,它实现了挖矿权重计算和资产管理的逻辑.代码中定义了几个trait的实现,
用于计算挖矿权重,处理资产变动事件,分配挖矿奖励等.以下是代码的主要组成部分和它们的功能:

1. **`BaseMiningWeight` trait的实现**:
   - 为`AssetLedgerWrapper`和`MinerLedgerWrapper`实现了`BaseMiningWeight` trait,
   这允许它们作为挖矿权重的基础数据结构.这些实现提供了访问和修改挖矿权重相关字段的方法.

2. **`ComputeMiningWeight` trait的实现**:
   - 为`Pallet`实现了`ComputeMiningWeight` trait,这允许计算特定矿工或资产的挖矿权重.这包括计算权重因子和处理挖矿奖励的逻辑.

3. **`xpallet_assets::OnAssetChanged` trait的实现**:
   - 为`Pallet`实现了`OnAssetChanged` trait,这允许在资产发行,转移或销毁之前和之后执行特定的逻辑.这包括初始化矿工的挖矿账本,更新挖矿权重等.

4. **`Claim` trait的实现**:
   - 为`Pallet`实现了`Claim` trait,这允许矿工认领他们的挖矿奖励.这包括验证认领条件,计算奖励,分配奖励给矿工和推荐人(国库)等.

5. **`RegistrarHandler` trait的实现**:
   - 为`Pallet`实现了`RegistrarHandler` trait,这允许在资产注册或注销时执行特定的逻辑.这包括管理有挖矿权的资产列表和初始化资产的挖矿账本.

6. **`AssetMining` trait的实现**:
   - 为`Pallet`实现了`AssetMining` trait,这允许收集所有挖矿资产的挖矿功率,并发行奖励到资产的奖励池账户.

7. **`SimpleAssetRewardPotAccountDeterminer` struct**:
   - 定义了一个简单的资产奖励池账户确定器,它使用资产ID和注册区块号的哈希来确定奖励池账户.

整体来看,这段代码为ChainX区块链提供了一套完整的挖矿奖励和权重管理机制.它确保了挖矿奖励的分配是公平和透明的,
并且与矿工的挖矿权重成正比.这对于维护区块链的安全性和激励矿工参与挖矿至关重要.
*/
