use frame_system::RawOrigin;

use crate::{
    mock::{bob, charlie, dave, ExtBuilder, Test, XAssets, XGatewayCommon, XGatewayRecords},
    Pallet, TrusteeSessionInfoLen, TrusteeSessionInfoOf, TrusteeSigRecord,
};
use frame_support::assert_ok;
use xp_assets_registrar::Chain;
use xp_protocol::X_BTC;

#[test]
fn test_do_trustee_election() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(TrusteeSessionInfoLen::<Test>::get(Chain::Bitcoin), 0);

        assert_eq!(Pallet::<Test>::do_trustee_election(Chain::Bitcoin), Ok(()));

        assert_eq!(TrusteeSessionInfoLen::<Test>::get(Chain::Bitcoin), 1);
    })
}

#[test]
fn test_move_trustee_into_little_black_house() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(XGatewayCommon::do_trustee_election(Chain::Bitcoin), Ok(()));
        assert!(!XGatewayCommon::trustee_transition_status(Chain::Bitcoin));

        TrusteeSigRecord::<Test>::mutate(Chain::Bitcoin, bob(), |record| *record = 10);
        assert_eq!(
            XGatewayCommon::trustee_sig_record(Chain::Bitcoin, bob()),
            10
        );

        assert_ok!(XGatewayCommon::move_trust_into_black_room(
            RawOrigin::Root.into(),
            Chain::Bitcoin,
            Some(vec![bob()]),
        ));

        assert_eq!(
            XGatewayCommon::little_black_house(Chain::Bitcoin),
            vec![bob()]
        );
        assert_eq!(XGatewayCommon::trustee_sig_record(Chain::Bitcoin, bob()), 0);

        assert!(XGatewayCommon::trustee_transition_status(Chain::Bitcoin));
    });
}

#[test]
fn test_claim_not_native_asset_reward() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(XGatewayCommon::do_trustee_election(Chain::Bitcoin), Ok(()));

        TrusteeSigRecord::<Test>::mutate(Chain::Bitcoin, bob(), |record| *record = 9);
        TrusteeSigRecord::<Test>::mutate(Chain::Bitcoin, charlie(), |record| *record = 1);

        assert_eq!(XGatewayCommon::trustee_sig_record(Chain::Bitcoin, bob()), 9);
        assert_eq!(
            XGatewayCommon::trustee_sig_record(Chain::Bitcoin, charlie()),
            1
        );
        assert_eq!(
            XGatewayCommon::trustee_sig_record(Chain::Bitcoin, dave()),
            0
        );

        let multi_address = XGatewayCommon::trustee_multisig_addr(Chain::Bitcoin).unwrap();

        assert_ok!(XGatewayRecords::deposit(&multi_address, X_BTC, 10));

        TrusteeSessionInfoOf::<Test>::mutate(Chain::Bitcoin, 1, |info| {
            if let Some(info) = info {
                info.0.trustee_list.iter_mut().for_each(|trustee| {
                    trustee.1 = XGatewayCommon::trustee_sig_record(Chain::Bitcoin, &trustee.0);
                });
            }
        });

        assert_ok!(XGatewayCommon::apply_claim_trustee_reward(1));

        assert_eq!(XAssets::usable_balance(&bob(), &X_BTC), 9);
        assert_eq!(XAssets::usable_balance(&charlie(), &X_BTC), 1);
    });
}

/*
这段代码是 ChainX 区块链项目的测试模块,用于测试受托人选举,受托人移入小黑屋,以及非原生资产奖励分配等功能.
测试使用了 Substrate 框架的测试工具,包括 `ExtBuilder` 用于构建测试环境,`execute_with` 用于在测试环境中执行操作,并检查预期结果.

### 测试用例:

1. **test_do_trustee_election**:
   - 测试受托人选举功能是否正确.首先检查比特币链上的受托人会话信息长度是否为 0,
   然后执行受托人选举,预期选举成功且会话信息长度增加到 1.

2. **test_move_trustee_into_little_black_house**:
   - 测试将受托人移入小黑屋的功能.首先执行受托人选举,然后检查受托人是否可以被成功移入小黑屋,
   同时检查受托人签名记录是否被重置为 0,以及受托人过渡状态是否被设置为 true.

3. **test_claim_not_native_asset_reward**:
   - 测试非原生资产奖励分配功能.首先设置受托人签名记录,然后模拟向受托人多重签名地址存款,
   接着更新受托人会话信息中的签名记录,并执行奖励分配.最后检查受托人账户的可用余额是否正确反映了他们应得的奖励.

### 测试辅助函数:

- `do_trustee_election`:
  - 执行受托人选举操作,预期返回 Ok(()) 表示成功.

- `move_trust_into_black_room`:
  - 将受托人移入小黑屋,预期返回 Ok(()) 表示成功.

- `trustee_sig_record`:
  - 获取指定受托人在特定链上的签名记录.

- `trustee_multisig_addr`:
  - 获取特定链上的受托人多重签名地址.

- `deposit`:
  - 模拟向指定地址存款操作.

- `apply_claim_trustee_reward`:
  - 执行受托人奖励分配操作.

- `usable_balance`:
  - 获取指定账户对特定资产的可用余额.

这些测试用例确保了 ChainX 区块链项目的关键功能在逻辑上是正确的,并且在实际部署前能够按预期工作.
通过模拟不同的操作和检查预期的结果,测试模块有助于发现和修复潜在的错误,提高系统的稳定性和可靠性.
*/
