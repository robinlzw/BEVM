// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

use super::*;
use crate::Pallet as XGatewayRecords;

const ASSET_ID: AssetId = xp_protocol::X_BTC;

fn deposit<T: Config>(who: T::AccountId, amount: BalanceOf<T>) {
    let receiver_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(who);
    // root_deposit
    let _ = XGatewayRecords::<T>::root_deposit(
        RawOrigin::Root.into(),
        receiver_lookup,
        ASSET_ID,
        amount,
    );
}

fn deposit_and_withdraw<T: Config>(who: T::AccountId, amount: BalanceOf<T>) {
    deposit::<T>(who.clone(), amount);
    let withdrawal = amount - 500u32.into();
    let addr = b"3LFSUKkP26hun42J1Dy6RATsbgmBJb27NF".to_vec();
    let memo = b"memo".to_vec().into();
    let receiver_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(who);
    XGatewayRecords::<T>::root_withdraw(
        RawOrigin::Root.into(),
        receiver_lookup,
        ASSET_ID,
        withdrawal,
        addr,
        memo,
    )
    .unwrap();
    assert_eq!(
        XGatewayRecords::<T>::state_of(0),
        Some(WithdrawalState::Applying)
    );
}

benchmarks! {
    root_deposit {
        let receiver: T::AccountId = whitelisted_caller();
        let receiver_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(receiver.clone());
        let amount: BalanceOf<T> = 1000u32.into();
    }: _(RawOrigin::Root, receiver_lookup, ASSET_ID, amount)
    verify {
        assert_eq!(xpallet_assets::Pallet::<T>::usable_balance(&receiver, &ASSET_ID), amount);
    }

    root_withdraw {
        let receiver: T::AccountId = whitelisted_caller();
        let receiver_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(receiver.clone());
        let amount: BalanceOf<T> = 1000u32.into();
        deposit::<T>(receiver, amount);
        let withdrawal = amount - 500u32.into();
        let addr = b"3LFSUKkP26hun42J1Dy6RATsbgmBJb27NF".to_vec();
        let memo = b"memo".to_vec().into();
    }: _(RawOrigin::Root, receiver_lookup, ASSET_ID, withdrawal, addr, memo)
    verify {
        assert!(XGatewayRecords::<T>::pending_withdrawals(0).is_some());
        assert_eq!(XGatewayRecords::<T>::state_of(0), Some(WithdrawalState::Applying));
    }

    set_withdrawal_state {
        let receiver: T::AccountId = whitelisted_caller();
        let receiver_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(receiver.clone());
        let amount: BalanceOf<T> = 1000u32.into();
        deposit_and_withdraw::<T>(receiver, amount);
        let state = WithdrawalState::RootFinish;
    }: _(RawOrigin::Root, 0, state)
    verify {
        assert_eq!(XGatewayRecords::<T>::state_of(0), None);
    }

    set_withdrawal_state_list {
        let u in 1 .. 64 => ();

        let receiver: T::AccountId = whitelisted_caller();
        let receiver_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(receiver.clone());
        let amount: BalanceOf<T> = 1000u32.into();
        deposit_and_withdraw::<T>(receiver, amount);
        let state = WithdrawalState::RootFinish;
    }: _(RawOrigin::Root, vec![(0, state)])
    verify {
        assert_eq!(XGatewayRecords::<T>::state_of(0), None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{ExtBuilder, Test};
    use frame_support::assert_ok;

    #[test]
    fn test_benchmarks() {
        ExtBuilder::default().build().execute_with(|| {
            assert_ok!(Pallet::<Test>::test_benchmark_root_deposit());
            assert_ok!(Pallet::<Test>::test_benchmark_root_withdraw());
            assert_ok!(Pallet::<Test>::test_benchmark_set_withdrawal_state());
        });
    }
}

/*
这段代码是 ChainX 项目中用于执行基准测试的套件,特别是针对 `XGatewayRecords` 模块的存款和取款操作.
基准测试是用来评估和测量系统性能的一种方法,可以帮助开发者识别瓶颈和优化性能.

### 基准测试函数

- **root_deposit**: 模拟根账户(即系统管理员)向 `XGatewayRecords` 模块存入资产的操作.

- **root_withdraw**: 模拟根账户从 `XGatewayRecords` 模块取款的操作,并验证取款状态是否正确设置为 `Applying`(申请中).

- **set_withdrawal_state**: 模拟根账户设置特定取款记录的状态.

- **set_withdrawal_state_list**: 模拟根账户批量设置多个取款记录的状态.

### 辅助函数

- **deposit**: 一个辅助函数,用于向 `XGatewayRecords` 模块存入指定账户的资产.

- **deposit_and_withdraw**: 一个辅助函数,首先存入资产,然后执行取款操作.

### 测试模块

- **tests**: 包含一个测试函数 `test_benchmarks`,它构建了一个测试外部环境并执行所有基准测试,确保它们都能成功运行.

### 总结

这些基准测试对于评估 ChainX 项目中 `XGatewayRecords` 模块的性能至关重要.通过模拟不同的操作,如存款,取款和状态设置,
开发者可以了解模块在不同负载下的表现,从而进行优化和调整,确保系统的稳定性和效率.这些测试也有助于确保系统的安全性和可靠性,特别是在处理大量的跨链交易时.
*/
