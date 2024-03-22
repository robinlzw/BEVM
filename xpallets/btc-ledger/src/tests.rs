// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use crate::mock::*;

use frame_support::{
    assert_err, assert_noop, assert_ok,
    traits::{
        Currency,
        ExistenceRequirement::{AllowDeath, KeepAlive},
        WithdrawReasons,
    },
};
use frame_system::RawOrigin;
use sp_core::crypto::AccountId32;
use sp_runtime::{traits::BadOrigin, ArithmeticError};

#[test]
fn btc_ledger_account_id() {
    let account_id = BtcLedger::account_id();
    let expect = "5EYCAe5iijNKP1cK7TuhPY6Sa5FFnmuyrGtjJmMTQWwJ75Dg";

    assert_eq!(format!("{}", account_id), expect)
}

#[test]
fn account_zero_balance_should_be_not_reaped() {
    new_test_ext().execute_with(|| {
        assert!(frame_system::Account::<Test>::contains_key(
            &AccountId32::from(ALICE)
        ));

        assert_eq!(BtcLedger::free_balance(AccountId32::from(ALICE)), 10);
        assert_ok!(<BtcLedger as Currency<_>>::transfer(
            &ALICE.into(),
            &BOB.into(),
            10,
            AllowDeath
        ));

        // Check that the account is not dead.
        assert!(frame_system::Account::<Test>::contains_key(
            &AccountId32::from(ALICE)
        ));
    });
}

#[test]
fn account_provider_consumer_sufficient() {
    new_test_ext().execute_with(|| {
        // SCENARIO: From existing account to existing account
        assert_eq!(System::providers(&ALICE.into()), 1);
        assert_eq!(System::consumers(&ALICE.into()), 0);
        assert_eq!(System::sufficients(&ALICE.into()), 0);
        assert_eq!(System::providers(&BOB.into()), 1);
        assert_eq!(System::consumers(&BOB.into()), 0);
        assert_eq!(System::sufficients(&BOB.into()), 0);

        assert!(System::account_exists(&ALICE.into()));
        assert!(System::account_exists(&BOB.into()));
        assert_ok!(<BtcLedger as Currency<_>>::transfer(
            &ALICE.into(),
            &BOB.into(),
            5,
            AllowDeath
        ));
        assert!(System::account_exists(&ALICE.into()));
        assert!(System::account_exists(&BOB.into()));
        assert_eq!(BtcLedger::free_balance(AccountId32::from(ALICE)), 5);
        assert_eq!(BtcLedger::free_balance(AccountId32::from(BOB)), 25);

        assert_eq!(System::providers(&BOB.into()), 1);
        assert_eq!(System::consumers(&BOB.into()), 0);
        assert_eq!(System::sufficients(&BOB.into()), 0);

        // SCENARIO: From existing account to nonexistent account
        assert_eq!(System::providers(&ALICE.into()), 1);
        assert_eq!(System::consumers(&ALICE.into()), 0);
        assert_eq!(System::sufficients(&ALICE.into()), 0);
        assert_eq!(System::providers(&CHARLIE.into()), 0);
        assert_eq!(System::consumers(&CHARLIE.into()), 0);
        assert_eq!(System::sufficients(&CHARLIE.into()), 0);

        assert!(!System::account_exists(&CHARLIE.into()));
        assert_ok!(<BtcLedger as Currency<_>>::transfer(
            &ALICE.into(),
            &CHARLIE.into(),
            5,
            AllowDeath
        ));
        assert!(System::account_exists(&CHARLIE.into()));
        assert_eq!(BtcLedger::free_balance(AccountId32::from(ALICE)), 0);
        assert_eq!(BtcLedger::free_balance(AccountId32::from(CHARLIE)), 5);

        assert_eq!(System::providers(&ALICE.into()), 1);
        assert_eq!(System::consumers(&ALICE.into()), 0);
        assert_eq!(System::sufficients(&ALICE.into()), 0);
        assert_eq!(System::providers(&CHARLIE.into()), 0);
        assert_eq!(System::consumers(&CHARLIE.into()), 0);
        assert_eq!(System::sufficients(&CHARLIE.into()), 1);
    });
}

#[test]
fn reward_should_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(BtcLedger::total_balance(&ALICE.into()), 10);
        assert_ok!(BtcLedger::deposit_into_existing(&ALICE.into(), 10).map(drop));
        System::assert_last_event(Event::BtcLedger(crate::Event::Deposit {
            who: ALICE.into(),
            amount: 10,
        }));
        assert_eq!(BtcLedger::total_balance(&ALICE.into()), 20);
        assert_eq!(btc_ledger::TotalInComing::<Test>::get(), 40);
    });
}

#[test]
fn balance_works() {
    new_test_ext().execute_with(|| {
        let _ = BtcLedger::deposit_creating(&ALICE.into(), 30);
        System::assert_has_event(Event::BtcLedger(crate::Event::Deposit {
            who: ALICE.into(),
            amount: 30,
        }));
        assert_eq!(BtcLedger::free_balance(&AccountId32::from(ALICE)), 40);
        assert_eq!(BtcLedger::total_balance(&ALICE.into()), 40);
        assert_eq!(BtcLedger::free_balance(AccountId32::from(BOB)), 20);
        assert_eq!(BtcLedger::total_balance(&BOB.into()), 20);
    });
}

#[test]
fn balance_transfer_works() {
    new_test_ext().execute_with(|| {
        let _ = BtcLedger::deposit_creating(&ALICE.into(), 40);
        assert_ok!(BtcLedger::transfer(
            Some(ALICE.into()).into(),
            BOB.into(),
            20
        ));
        assert_eq!(BtcLedger::total_balance(&ALICE.into()), 30);
        assert_eq!(BtcLedger::total_balance(&BOB.into()), 40);
    });
}

#[test]
fn force_transfer_works() {
    new_test_ext().execute_with(|| {
        let _ = BtcLedger::deposit_creating(&ALICE.into(), 50);
        assert_noop!(
            BtcLedger::force_transfer(Some(BOB.into()).into(), ALICE.into(), BOB.into(), 50),
            BadOrigin,
        );

        assert_ok!(BtcLedger::force_transfer(
            RawOrigin::Root.into(),
            ALICE.into(),
            BOB.into(),
            50
        ));
        assert_eq!(BtcLedger::total_balance(&ALICE.into()), 10);
        assert_eq!(BtcLedger::total_balance(&BOB.into()), 70);
    });
}

#[test]
fn withdrawing_balance_should_work() {
    new_test_ext().execute_with(|| {
        let _ = BtcLedger::deposit_creating(&BOB.into(), 100);
        let _ = BtcLedger::withdraw(&BOB.into(), 20, WithdrawReasons::TRANSFER, AllowDeath);

        System::assert_last_event(Event::BtcLedger(crate::Event::Withdraw {
            who: BOB.into(),
            amount: 20,
        }));

        assert_eq!(BtcLedger::free_balance(AccountId32::from(BOB)), 100);
        assert_eq!(btc_ledger::TotalInComing::<Test>::get(), 110);

        let _ = BtcLedger::withdraw(&ALICE.into(), 10, WithdrawReasons::TRANSFER, KeepAlive);

        System::assert_last_event(Event::BtcLedger(crate::Event::Withdraw {
            who: ALICE.into(),
            amount: 10,
        }));

        assert_eq!(BtcLedger::free_balance(AccountId32::from(BOB)), 100);
        assert_eq!(btc_ledger::TotalInComing::<Test>::get(), 100);
    });
}

#[test]
fn transferring_too_high_value_should_not_panic() {
    new_test_ext().execute_with(|| {
        BtcLedger::make_free_balance_be(&ALICE.into(), u128::MAX);
        BtcLedger::make_free_balance_be(&BOB.into(), 1);

        assert_err!(
            BtcLedger::transfer(Some(ALICE.into()).into(), BOB.into(), u128::MAX),
            ArithmeticError::Overflow,
        );

        assert_eq!(BtcLedger::free_balance(AccountId32::from(ALICE)), u128::MAX);
        assert_eq!(BtcLedger::free_balance(AccountId32::from(BOB)), 1);
    });
}

#[test]
fn burn_must_work() {
    new_test_ext().execute_with(|| {
        let init_total_issuance = BtcLedger::total_issuance();
        let imbalance = BtcLedger::burn(10);
        assert_eq!(BtcLedger::total_issuance(), init_total_issuance - 10);
        drop(imbalance);
        assert_eq!(BtcLedger::total_issuance(), init_total_issuance);
    });
}

#[test]
#[should_panic = "duplicate balances in genesis."]
fn cannot_set_genesis_value_twice() {
    let mut t = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();
    let _ = btc_ledger::GenesisConfig::<Test> {
        balances: vec![(ALICE.into(), 10), (BOB.into(), 20), (ALICE.into(), 15)],
    }
    .assimilate_storage(&mut t)
    .unwrap();
}

#[test]
fn transfer_all_free_succeed() {
    new_test_ext().execute_with(|| {
        assert_ok!(BtcLedger::set_balance(Origin::root(), ALICE.into(), 100));
        assert_ok!(BtcLedger::transfer(
            Some(ALICE.into()).into(),
            BOB.into(),
            100
        ));
        assert_eq!(BtcLedger::total_balance(&ALICE.into()), 0);
        assert_eq!(BtcLedger::total_balance(&BOB.into()), 120);
    });
}

#[test]
fn transfer_all_works() {
    new_test_ext().execute_with(|| {
        // setup
        assert_ok!(BtcLedger::set_balance(Origin::root(), ALICE.into(), 200));
        assert_ok!(BtcLedger::set_balance(Origin::root(), BOB.into(), 0));
        // transfer all and allow death
        assert_ok!(BtcLedger::transfer(
            Some(ALICE.into()).into(),
            BOB.into(),
            200
        ));
        assert_eq!(BtcLedger::total_balance(&ALICE.into()), 0);
        assert_eq!(BtcLedger::total_balance(&BOB.into()), 200);

        // setup
        assert_ok!(BtcLedger::set_balance(Origin::root(), ALICE.into(), 200));
        assert_ok!(BtcLedger::set_balance(Origin::root(), BOB.into(), 0));
        // transfer all and keep alive
        assert_ok!(BtcLedger::transfer(
            Some(ALICE.into()).into(),
            BOB.into(),
            200
        ));
        assert_eq!(BtcLedger::total_balance(&ALICE.into()), 0);
        assert_eq!(BtcLedger::total_balance(&BOB.into()), 200);
    });
}

#[test]
fn set_balance_handles_total_issuance() {
    new_test_ext().execute_with(|| {
        let old_total_issuance = BtcLedger::total_issuance();
        assert_ok!(BtcLedger::set_balance(Origin::root(), CHARLIE.into(), 69));
        assert_eq!(BtcLedger::total_issuance(), old_total_issuance + 69);
        assert_eq!(BtcLedger::total_balance(&CHARLIE.into()), 69);
        assert_eq!(BtcLedger::free_balance(&CHARLIE.into()), 69);
    });
}

/*

这段代码是一系列用于测试 `btc_ledger` 模块的单元测试.这些测试使用 Substrate 的测试框架来模拟区块链运行时环境,
并验证 `btc_ledger` 模块的各种功能是否按预期工作.以下是对每个测试的详细解释:

### 测试用例

1. **btc_ledger_account_id**: 验证 `btc_ledger` 模块的账户 ID 是否正确.

2. **account_zero_balance_should_be_not_reaped**: 确保账户余额为零时不会被删除(reaped).

3. **account_provider_consumer_sufficient**: 测试转账操作对账户提供者(providers),消费者(consumers)和足够余额(sufficients)计数器的影响.

4. **reward_should_work**: 测试奖励(deposit)功能是否正确增加了账户余额和总发行量.

5. **balance_works**: 测试创建新账户并存入余额是否成功.

6. **balance_transfer_works**: 测试转账功能是否正确处理账户余额和总余额.

7. **force_transfer_works**: 测试强制转账功能是否按预期工作,即使是非根账户发起的请求也应该失败.

8. **withdrawing_balance_should_work**: 测试取款功能是否正确处理账户余额和总发行量.

9. **transferring_too_high_value_should_not_panic**: 确保转账操作在处理过高的值时不会发生 panic.

10. **burn_must_work**: 测试销毁(burn)功能是否正确减少了总发行量.

11. **cannot_set_genesis_value_twice**: 确保在创世配置中不能为同一账户设置重复的余额,这应该会触发 panic.

12. **transfer_all_free_succeed**: 测试当账户中有足够的余额时,转账操作是否成功.

13. **transfer_all_works**: 类似于 `transfer_all_free_succeed`,但测试在不同情况下转账所有余额的行为.

14. **set_balance_handles_total_issuance**: 测试设置账户余额是否正确更新了总发行量.

### 测试辅助函数

- `new_test_ext`: 创建一个新的测试外部环境,包括初始化存储和执行一些初始操作.

这些测试用例覆盖了 `btc_ledger` 模块的大部分功能,确保了模块在各种情况下的行为都是正确的.

---------------------------------------------------------------------------------------------------------
创世配置(Genesis Configuration)是指在区块链网络启动时用于初始化区块链状态的一组参数和设置.
这些配置通常在区块链的创世区块(Genesis Block)中定义,它们为区块链的运行设定了初始条件,包括但不限于:

1. **账户余额**:定义了区块链上各个账户的初始余额.

2. **系统参数**:设置了区块链的一些基础参数,如区块奖励,交易费用,共识机制参数等.

3. **代码和合约**:对于智能合约平台,创世配置可能包括预部署的智能合约或链上运行的代码.

4. **权限和身份**:定义了哪些账户具有特殊的权限,例如挖矿,治理或系统管理权限.

5. **链上资产**:对于跨链系统,可能包括初始映射的外部资产或代币.

在 Substrate 框架中,创世配置是通过 `GenesisConfig` 结构体来定义的,它允许开发者在链启动之前指定所有必要的初始状态.
例如,在上面的代码中,`btc_ledger::GenesisConfig` 用于初始化 `btc_ledger` 模块的账户余额.

创世配置是区块链网络安全,去中心化和透明运行的基础,因为一旦区块链启动并开始处理交易,这些初始状态就变得不可更改(除非通过硬分叉等机制).
*/
