// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

pub use super::mock::*;
use super::*;

use frame_support::{assert_noop, assert_ok};
use frame_system::RawOrigin;

#[test]
fn test_normal() {
    ExtBuilder::default().build_and_execute(|| {
        // deposit
        assert_ok!(XGatewayRecords::deposit(&ALICE, X_BTC, 100));
        assert_eq!(XAssets::usable_balance(&ALICE, &X_BTC), 100 + 100);

        // withdraw
        assert_ok!(XGatewayRecords::withdraw(
            &ALICE,
            X_BTC,
            50,
            b"addr".to_vec(),
            b"ext".to_vec().into()
        ));

        let numbers = XGatewayRecords::withdrawals_list_by_chain(Chain::Bitcoin)
            .into_iter()
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        assert_eq!(numbers.len(), 1);

        assert_ok!(XGatewayRecords::process_withdrawals(
            &numbers,
            Chain::Bitcoin
        ));
        for i in numbers {
            assert_ok!(XGatewayRecords::finish_withdrawal(i, None));
        }
        assert_eq!(XAssets::usable_balance(&ALICE, &X_BTC), 50 + 100);
    })
}

#[test]
fn test_normal2() {
    ExtBuilder::default().build_and_execute(|| {
        // deposit
        assert_ok!(XGatewayRecords::deposit(&ALICE, X_BTC, 100));
        assert_eq!(XAssets::usable_balance(&ALICE, &X_BTC), 100 + 100);
        assert_ok!(XGatewayRecords::deposit(&ALICE, X_ETH, 500));
        assert_eq!(XAssets::usable_balance(&ALICE, &X_ETH), 500 + 100);

        // withdraw
        assert_ok!(XGatewayRecords::withdraw(
            &ALICE,
            X_BTC,
            50,
            b"addr".to_vec(),
            b"ext".to_vec().into()
        ));
        // withdrawal twice at once
        assert_ok!(XGatewayRecords::withdraw(
            &ALICE,
            X_ETH,
            100,
            b"addr".to_vec(),
            b"ext".to_vec().into()
        ));
        assert_ok!(XGatewayRecords::withdraw(
            &ALICE,
            X_ETH,
            50,
            b"addr".to_vec(),
            b"ext".to_vec().into()
        ));

        let numbers1 = XGatewayRecords::withdrawals_list_by_chain(Chain::Bitcoin)
            .into_iter()
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        assert_eq!(numbers1.len(), 1);

        let numbers2 = XGatewayRecords::withdrawals_list_by_chain(Chain::Ethereum)
            .into_iter()
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        assert_eq!(numbers2.len(), 2);

        let mut wrong_numbers = numbers1.clone();
        wrong_numbers.extend_from_slice(&numbers2);

        assert_noop!(
            XGatewayRecords::process_withdrawals(&wrong_numbers, Chain::Bitcoin),
            XRecordsErr::UnexpectedChain
        );
        assert_ok!(XGatewayRecords::process_withdrawals(
            &numbers1,
            Chain::Bitcoin
        ));
        assert_ok!(XGatewayRecords::process_withdrawals(
            &numbers2,
            Chain::Ethereum
        ));

        assert_ok!(XGatewayRecords::finish_withdrawals(
            &numbers1,
            Some(Chain::Bitcoin)
        ));
        assert_ok!(XGatewayRecords::finish_withdrawals(
            &numbers2,
            Some(Chain::Ethereum)
        ));

        assert_eq!(XAssets::usable_balance(&ALICE, &X_BTC), 50 + 100);
        assert_eq!(
            XAssets::usable_balance(&ALICE, &X_ETH),
            500 + 100 - 50 - 100
        );
    })
}

#[test]
fn test_withdrawal_more_then_usable() {
    ExtBuilder::default().build_and_execute(|| {
        assert_ok!(XGatewayRecords::deposit(&ALICE, X_BTC, 10));

        assert_noop!(
            XGatewayRecords::withdraw(
                &ALICE,
                X_BTC,
                100 + 50,
                b"addr".to_vec(),
                b"ext".to_vec().into()
            ),
            xpallet_assets::Error::<Test>::InsufficientBalance
        );
    })
}

#[test]
fn test_withdrawal_force_set_state() {
    ExtBuilder::default().build_and_execute(|| {
        assert_ok!(XGatewayRecords::deposit(&ALICE, X_BTC, 10));
        // applying
        assert_ok!(XGatewayRecords::withdraw(
            &ALICE,
            X_BTC,
            10,
            b"addr".to_vec(),
            b"ext".to_vec().into()
        ));
        assert_eq!(XAssets::usable_balance(&ALICE, &X_BTC), 100);
        // ignore processing state, force release locked balance
        assert_ok!(XGatewayRecords::set_withdrawal_state(
            RawOrigin::Root.into(),
            0,
            WithdrawalState::RootCancel
        ));
        assert_eq!(XAssets::usable_balance(&ALICE, &X_BTC), 100 + 10);
        // change to processing
        assert_ok!(XGatewayRecords::withdraw(
            &ALICE,
            X_BTC,
            10,
            b"addr".to_vec(),
            b"ext".to_vec().into()
        ));
        assert_ok!(XGatewayRecords::set_withdrawal_state(
            RawOrigin::Root.into(),
            1,
            WithdrawalState::Processing
        ));
        // reject revoke for a processing state
        assert_noop!(
            XGatewayRecords::cancel_withdrawal(1, &ALICE),
            XRecordsErr::NotApplyingState
        );
        // force change to applying
        assert_ok!(XGatewayRecords::set_withdrawal_state(
            RawOrigin::Root.into(),
            1,
            WithdrawalState::Applying
        ));
        assert_eq!(
            XGatewayRecords::state_of(1),
            Some(WithdrawalState::Applying)
        );
    })
}

#[test]
fn test_withdrawal_chainx() {
    ExtBuilder::default().build_and_execute(|| {
        assert_noop!(
            XGatewayRecords::deposit(&ALICE, ChainXAssetId::get(), 10),
            xpallet_assets::Error::<Test>::DenyNativeAsset
        );

        assert_noop!(
            XGatewayRecords::withdraw(
                &ALICE,
                ChainXAssetId::get(),
                50,
                b"addr".to_vec(),
                b"ext".to_vec().into()
            ),
            xpallet_assets::Error::<Test>::DenyNativeAsset
        );
    })
}

/*
这段代码是一个Rust编写的测试套件,用于测试一个名为`XGatewayRecords`的模块或库中的一些功能.
这个模块可能是一个区块链应用程序的一部分,特别是与资产转移和网关记录相关的功能.以下是对代码中各个测试用例的解释:

1. `test_normal` 测试用例:
   - 首先,通过调用`deposit`函数,向网关记录中存入100单位的X_BTC资产.
   - 然后,检查ALICE账户的可用余额是否正确增加了存入的资产数量.
   - 接下来,执行一个提现操作,提现50单位的X_BTC资产到一个比特币地址.
   - 查询比特币链上的所有提现请求,并确保只有一个提现请求被记录.
   - 处理这个提现请求,然后完成提现操作.
   - 最后,验证ALICE账户的可用余额是否正确反映了提现后的数量.

2. `test_normal2` 测试用例:
   - 类似于`test_normal`,但同时涉及X_BTC和X_ETH两种资产的存入和提现操作.
   - 提现操作包括两次不同资产的提现请求.
   - 验证不同链上的提现请求列表,并分别处理每个链的提现.
   - 完成提现后,检查ALICE账户的可用余额是否正确.

3. `test_withdrawal_more_then_usable` 测试用例:
   - 尝试提现超过可用余额的资产数量.
   - 预期结果是操作失败,并返回一个错误,指出余额不足.

4. `test_withdrawal_force_set_state` 测试用例:
   - 存入资产后,执行提现操作.
   - 使用管理员权限强制更改提现状态.
   - 验证强制更改状态后,资产余额是否正确.
   - 尝试取消一个正在处理中的提现请求,并预期操作失败.
   - 强制将提现状态更改为申请中,并验证状态是否正确更新.

5. `test_withdrawal_chainx` 测试用例:
   - 尝试对ChainX的本地资产进行存入和提现操作.
   - 预期结果是操作失败,因为本地资产不允许进行这些操作.

这些测试用例覆盖了正常操作流程,异常情况处理,管理员权限操作以及特定资产规则的测试.通过这些测试,可以确保`XGatewayRecords`模块在不同情况下的行为符合预期.
*/
