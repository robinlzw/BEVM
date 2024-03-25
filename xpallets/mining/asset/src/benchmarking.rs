// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

pub use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;
use xp_protocol::{X_BTC, X_DOT};

use super::*;

const SEED: u32 = 0;

benchmarks! {
    claim {
        xpallet_assets_registrar::Pallet::<T>::register(
            frame_system::RawOrigin::Root.into(),
            X_DOT,
            xpallet_assets_registrar::AssetInfo::new::<T>(
                b"X-DOT".to_vec(),
                b"Polkadot".to_vec(),
                xpallet_assets_registrar::Chain::Polkadot,
                10,
                b"Polkadot".to_vec(),
            ).unwrap(),
            true,
            true,
        ).unwrap();

        FixedAssetPowerOf::<T>::insert(X_DOT, 100);

        let miner = account("miner", 0, SEED);
        xpallet_assets::Pallet::<T>::issue(&X_DOT, &miner, 1000u32.into(), true)?;

        let reward_pot = T::DetermineRewardPotAccount::reward_pot_account_for(&X_DOT);
        <T as xpallet_assets::Config>::Currency::make_free_balance_be(&reward_pot, 100u32.into());
        <T as xpallet_assets::Config>::Currency::issue(100u32.into());

        Pallet::<T>::set_claim_staking_requirement(RawOrigin::Root.into(), X_DOT, 0)?;

        let block_number: T::BlockNumber = frame_system::Pallet::<T>::block_number();
        frame_system::Pallet::<T>::set_block_number(block_number + 100u32.into());

    }: _(RawOrigin::Signed(miner.clone()), X_DOT)
    verify {
        // 10% belongs to the referral/treasury, 90% is the miner's reward.
        assert!(Pallet::<T>::free_balance(&miner) == 90u32.into());
    }

    set_claim_staking_requirement {
        let c = 1000;
    }: _(RawOrigin::Root, X_BTC, c)
    verify {
        assert_eq!(ClaimRestrictionOf::<T>::get(X_BTC).staking_requirement, c);
    }

    set_claim_frequency_limit {
        let c = 1000u32;
    }: _(RawOrigin::Root, X_BTC, c.into())
    verify {
        assert_eq!(ClaimRestrictionOf::<T>::get(X_BTC).frequency_limit, c.into());
    }

    set_asset_power {
        let c = 1000;
    }: _(RawOrigin::Root, X_BTC, c)
    verify {
        assert_eq!(FixedAssetPowerOf::<T>::get(X_BTC), c);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{ExtBuilder, Test};
    use frame_support::assert_ok;

    #[test]
    fn test_benchmarks() {
        ExtBuilder::default().build().execute_with(|| {
            assert_ok!(crate::tests::t_register_xbtc());
            assert_ok!(Pallet::<Test>::test_benchmark_claim());
            assert_ok!(Pallet::<Test>::test_benchmark_set_claim_staking_requirement());
            assert_ok!(Pallet::<Test>::test_benchmark_set_claim_frequency_limit());
            assert_ok!(Pallet::<Test>::test_benchmark_set_asset_power());
        });
    }
}

/*
这段代码是ChainX区块链项目的一部分,它定义了一系列基准测试(benchmarks)来评估链上某些操作的性能.
这些基准测试是使用Substrate框架的`frame_benchmarking`库编写的,旨在确保随着区块链状态的增长,系统的关键功能仍然能够高效运行.

基准测试的主要内容包括:

1. **`claim`基准测试**:
   - 这个测试模拟了一个矿工对其挖矿奖励进行认领的过程.它首先注册了一个名为`X_DOT`的资产,并设置了其相关信息,包括资产名称,描述,链类型,小数位数等.
   - 然后,它为`X_DOT`资产设置了固定的挖矿权重,并为一个矿工账户发行了1000个`X_DOT`代币.
   - 接着,它为`X_DOT`资产的奖励池账户注入了100个代币,并设置了认领质押要求为0.
   - 最后,它模拟了一个区块时间的推进,并执行了一个认领操作.验证步骤确保矿工账户的余额正确反映了90%的奖励(假设10%归属于推荐/国库).

2. **`set_claim_staking_requirement`基准测试**:
   - 这个测试设置了对于`X_BTC`资产的认领质押要求,并验证设置是否成功.

3. **`set_claim_frequency_limit`基准测试**:
   - 这个测试设置了对于`X_BTC`资产的认领频率限制,并验证设置是否成功.

4. **`set_asset_power`基准测试**:
   - 这个测试设置了对于`X_BTC`资产的固定挖矿权重,并验证设置是否成功.

此外,代码中还包含了一个`tests`模块,它使用`#[cfg(test)]`属性来确保只有在编译测试时才会包含这些代码.
这个模块中有一个测试函数`test_benchmarks`,它构建了一个测试环境并执行了所有的基准测试,确保它们都能够成功运行.

整体来看,这段代码为ChainX区块链提供了一套全面的基准测试,用于确保系统的关键功能在不同负载下的性能.
这对于维护区块链的稳定性和可靠性至关重要,尤其是在面对大量交易和状态变化时.
*/
