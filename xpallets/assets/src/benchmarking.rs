// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use super::*;

use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

use crate::Pallet as XAssets;

const ASSET_ID: AssetId = xp_protocol::X_BTC;
const SEED: u32 = 0;

benchmarks! {
    transfer {
        let caller = whitelisted_caller();
        let transfer_amount: BalanceOf<T> = (100000000 * 10_u32).into(); // e.g. 10 btc
        XAssets::<T>::issue(&ASSET_ID, &caller, transfer_amount, true).unwrap();

        let recipient: T::AccountId = account("recipient", 0, SEED);
        let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient.clone());
    }: _(RawOrigin::Signed(caller.clone()), recipient_lookup, ASSET_ID, transfer_amount)
    verify {
        assert_eq!(XAssets::<T>::usable_balance(&caller, &ASSET_ID), Zero::zero());
        assert_eq!(XAssets::<T>::usable_balance(&recipient, &ASSET_ID), transfer_amount);
    }

    force_transfer {
        let caller = whitelisted_caller();
        let transfer_amount: BalanceOf<T> = (100000000 * 10_u32).into(); // e.g. 10 btc
        XAssets::<T>::issue(&ASSET_ID, &caller, transfer_amount, true).unwrap();

        let caller_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(caller.clone());
        let recipient: T::AccountId = account("recipient", 0, SEED);
        let recipient_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(recipient.clone());
    }: _(RawOrigin::Root, caller_lookup, recipient_lookup, ASSET_ID, transfer_amount)
    verify {
        assert_eq!(XAssets::<T>::usable_balance(&caller, &ASSET_ID), Zero::zero());
        assert_eq!(XAssets::<T>::usable_balance(&recipient, &ASSET_ID), transfer_amount);
    }

    set_balance {
        let n in 1 .. AssetType::iter().collect::<Vec<_>>().len() as u32;

        let user: T::AccountId = account("user", 0, SEED);
        let user_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(user.clone());
        let mut balances = BTreeMap::new();
        balances.insert(AssetType::Locked, 1000u32.into());
        balances.insert(AssetType::Locked, 1000u32.into());
        balances.insert(AssetType::Reserved, 1000u32.into());
        balances.insert(AssetType::ReservedWithdrawal, 1000u32.into());
        balances.insert(AssetType::ReservedDexSpot, 1000u32.into());
    }: set_balance(RawOrigin::Root, user_lookup, ASSET_ID, balances.clone())
    verify {
        assert_eq!(XAssets::<T>::asset_balance(&user, &ASSET_ID), balances);
    }

    set_asset_limit {
        let res = AssetRestrictions::DEPOSIT | AssetRestrictions::DESTROY_USABLE;
    }: set_asset_limit(RawOrigin::Root, ASSET_ID, res)
    verify {
        assert_eq!(XAssets::<T>::asset_restrictions_of(&ASSET_ID), res);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{ExtBuilder, Test};
    use frame_support::assert_ok;

    #[test]
    fn test_benchmarks() {
        ExtBuilder::default().build_default().execute_with(|| {
            assert_ok!(Pallet::<Test>::test_benchmark_transfer());
            assert_ok!(Pallet::<Test>::test_benchmark_force_transfer());
            assert_ok!(Pallet::<Test>::test_benchmark_set_balance());
            assert_ok!(Pallet::<Test>::test_benchmark_set_asset_limit());
        });
    }
}

/*
这段代码是ChainX项目的基准测试(benchmarking)模块,用于评估和测试`XAssets` pallet的性能.
基准测试是区块链开发中的一个重要组成部分,因为它们可以帮助开发者了解系统在不同操作下的性能表现,从而进行优化.

以下是代码的详细解释:

1. **模块和特性导入**:代码开始部分导入了所需的模块,包括`frame_benchmarking`用于基准测试框架,
`frame_system`用于Substrate的系统模块,以及项目内部的`crate`模块.

2. **基准测试定义**:使用`benchmarks!`宏定义了一系列基准测试,每个测试都是对`XAssets` pallet的一个操作,
例如`transfer`(转移资产),`force_transfer`(强制转移资产),`set_balance`(设置账户余额),`set_asset_limit`(设置资产限制).

3. **测试设置**:每个基准测试都有一个设置阶段,用于创建测试环境,例如发行资产,创建账户等.
使用`whitelisted_caller`创建一个有权限的调用者,使用`account`创建测试账户.

4. **测试操作**:每个基准测试都有一个操作阶段,使用`_`语法定义了执行的操作和其参数.例如,
在`transfer`测试中,调用`XAssets::<T>::issue`发行资产,然后使用`transfer`操作将资产转移到另一个账户.

5. **验证**:每个基准测试都有一个验证阶段,使用`verify`块来检查操作的结果是否符合预期.例如,检查发送者和接收者的余额是否正确更新.

6. **测试模块**:在`#[cfg(test)]`模块中定义了单元测试,使用`ExtBuilder`和`Test`来构建测试环境.
`test_benchmark_*`函数调用基准测试,`assert_ok!`宏用于断言测试是否成功执行.

整体来看,这段代码为`XAssets` pallet提供了一组基准测试,用于评估资产转移,余额设置和资产限制设置等操作的性能.
通过这些测试,开发者可以确保pallet的性能符合预期,并在必要时进行优化.
*/
