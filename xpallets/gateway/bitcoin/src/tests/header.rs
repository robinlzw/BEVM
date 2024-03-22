// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use frame_support::{assert_noop, assert_ok};

use light_bitcoin::{
    chain::{h256, BlockHeader},
    keys::Network,
    serialization,
};

use crate::mock::alice;
use crate::{
    mock::{
        generate_blocks_478557_478563, generate_blocks_63290_63310, ExtBuilder, XGatewayBitcoin,
        XGatewayBitcoinErr,
    },
    types::BtcHeaderIndex,
};

#[test]
fn test_genesis() {
    ExtBuilder::default().build_and_execute(|| {
        let (header, num) = XGatewayBitcoin::genesis_info();
        assert_eq!(
            header.hash(),
            h256("0x0e0afd82419f6fa40fcb1a77550dbb22e567f7ae6b4a95b77a00d30425010000")
        );
        assert_eq!(num, 63290);

        let index = XGatewayBitcoin::best_index();
        assert_eq!(
            index,
            BtcHeaderIndex {
                hash: header.hash(),
                height: 63290
            }
        );
    })
}

#[test]
fn test_insert_headers() {
    let (base_height, c1, _) = generate_blocks_478557_478563();
    ExtBuilder::default()
        .build_mock((*c1.get(0).unwrap(), base_height), Network::Mainnet)
        .execute_with(|| {
            assert_noop!(
                XGatewayBitcoin::apply_push_header(*c1.get(0).unwrap()),
                XGatewayBitcoinErr::ExistingHeader
            );

            assert_ok!(XGatewayBitcoin::apply_push_header(*c1.get(1).unwrap()));
            assert_ok!(XGatewayBitcoin::apply_push_header(*c1.get(2).unwrap()));
            assert_ok!(XGatewayBitcoin::apply_push_header(*c1.get(3).unwrap()));
            assert_ok!(XGatewayBitcoin::apply_push_header(*c1.get(4).unwrap()));
            assert_ok!(XGatewayBitcoin::apply_push_header(*c1.get(5).unwrap()));

            let best_index = XGatewayBitcoin::best_index();
            assert_eq!(best_index.hash, c1.get(5).unwrap().hash());

            should_in_mainchain(&c1, true);
        })
}

fn should_in_mainchain(headers: &[BlockHeader], expect: bool) {
    for header in headers.iter() {
        assert_eq!(XGatewayBitcoin::main_chain(&header.hash()), expect);
    }
}

#[test]
fn test_insert_forked_headers_from_genesis_height() {
    // e.g.
    // b1
    // b --- b --- b --- b
    // |---- b --- b
    let (base_height, c1, forked) = generate_blocks_478557_478563();
    ExtBuilder::default()
        .build_mock((*c1.get(1).unwrap(), base_height + 1), Network::Mainnet)
        .execute_with(|| {
            // note: confirm block is 4
            assert_noop!(
                XGatewayBitcoin::apply_push_header(*c1.get(1).unwrap()),
                XGatewayBitcoinErr::ExistingHeader
            );

            // insert first
            assert_ok!(XGatewayBitcoin::apply_push_header(*c1.get(2).unwrap()));
            // best index is normal block 2
            let best_index = XGatewayBitcoin::best_index();
            assert_eq!(best_index.hash, c1.get(2).unwrap().hash());

            // insert forked first
            assert_ok!(XGatewayBitcoin::apply_push_header(*forked.get(2).unwrap()));
            // contains two block at height 2, but best index still normal
            let best_index = XGatewayBitcoin::best_index();
            assert_eq!(best_index.hash, c1.get(2).unwrap().hash());
            should_in_mainchain(&c1[1..3], true);

            assert_ok!(XGatewayBitcoin::apply_push_header(*forked.get(3).unwrap()));
            // forked block overtake than normal, change current best to forked 3
            let best_index = XGatewayBitcoin::best_index();
            assert_eq!(best_index.hash, forked.get(3).unwrap().hash());
            assert_eq!(XGatewayBitcoin::confirmed_index(), None);
            should_in_mainchain(&c1[2..3], false);
            should_in_mainchain(&forked[1..4], true);

            // start insert normal
            assert_ok!(XGatewayBitcoin::apply_push_header(*c1.get(3).unwrap()));
            // because forked 3 insert before, so that even receive normal 3, best still forked 3
            let best_index = XGatewayBitcoin::best_index();
            assert_eq!(best_index.hash, forked.get(3).unwrap().hash());

            // switch forked to normal chain
            assert_ok!(XGatewayBitcoin::apply_push_header(*c1.get(4).unwrap()));
            let best_index = XGatewayBitcoin::best_index();
            assert_eq!(best_index.hash, c1.get(4).unwrap().hash());
            // confirm set to 1
            let confirmed_index = XGatewayBitcoin::confirmed_index().unwrap();
            assert_eq!(confirmed_index.hash, c1.get(1).unwrap().hash());

            // move confirmed in normal chain
            assert_ok!(XGatewayBitcoin::apply_push_header(*c1.get(5).unwrap()));
            let best_index = XGatewayBitcoin::best_index();
            assert_eq!(best_index.hash, c1.get(5).unwrap().hash());
            // confirm set to 2
            let confirmed_index = XGatewayBitcoin::confirmed_index().unwrap();
            assert_eq!(confirmed_index.hash, c1.get(2).unwrap().hash());

            should_in_mainchain(&c1[1..6], true);
            should_in_mainchain(&forked[2..4], false);
            println!("current confirmed height:{:?}", confirmed_index.height);

            // add a forked block exceed confirmed height, this forked block would mark as AncientFork,
            // but now this forked block is less then best, so do not do confirmed check
            // and on the other hand, confirmed block for this forked block is before current normal
            assert_ok!(XGatewayBitcoin::apply_push_header(*forked.get(4).unwrap()));
            // but when add a more forked block, would try to move confirmed
            assert_noop!(
                XGatewayBitcoin::apply_push_header(*forked.get(5).unwrap()),
                XGatewayBitcoinErr::AncientFork,
            );
        })
}

#[test]
fn test_insert_forked_headers() {
    // e.g.
    // b0
    // b --- b --- b --- b --- b
    //       |---- b --- b
    let (base_height, c1, forked) = generate_blocks_478557_478563();
    ExtBuilder::default()
        .build_mock((*c1.get(0).unwrap(), base_height), Network::Mainnet)
        .execute_with(|| {
            assert_ok!(XGatewayBitcoin::apply_push_header(*c1.get(1).unwrap()));
            assert_ok!(XGatewayBitcoin::apply_push_header(*c1.get(2).unwrap()));
            assert_ok!(XGatewayBitcoin::apply_push_header(*c1.get(3).unwrap()));
            should_in_mainchain(&c1[0..4], true);
            // now confirmed would set
            let confirmed_index = XGatewayBitcoin::confirmed_index().unwrap();
            assert_eq!(confirmed_index.hash, c1.get(0).unwrap().hash());

            // insert forked
            assert_ok!(XGatewayBitcoin::apply_push_header(*forked.get(2).unwrap()));
            assert_ok!(XGatewayBitcoin::apply_push_header(*forked.get(3).unwrap()));
            let best_index = XGatewayBitcoin::best_index();
            assert_eq!(best_index.hash, c1.get(3).unwrap().hash());
            let confirmed_index = XGatewayBitcoin::confirmed_index().unwrap();
            assert_eq!(confirmed_index.hash, c1.get(0).unwrap().hash());
            should_in_mainchain(&c1[0..4], true);
            should_in_mainchain(&forked[2..4], false);

            // insert forked, switch chain, but confirm b1, b1 is also the parent for normal chain
            assert_ok!(XGatewayBitcoin::apply_push_header(*forked.get(4).unwrap()));
            let best_index = XGatewayBitcoin::best_index();
            assert_eq!(best_index.hash, forked.get(4).unwrap().hash());
            let confirmed_index = XGatewayBitcoin::confirmed_index().unwrap();
            assert_eq!(confirmed_index.hash, c1.get(1).unwrap().hash());
            should_in_mainchain(&c1[2..4], false);
            should_in_mainchain(&forked[1..5], true);

            // b1 still on normal chain, so we could switch to normal and reset main chain
            assert_ok!(XGatewayBitcoin::apply_push_header(*c1.get(4).unwrap()));
            assert_ok!(XGatewayBitcoin::apply_push_header(*c1.get(5).unwrap()));
            let best_index = XGatewayBitcoin::best_index();
            assert_eq!(best_index.hash, c1.get(5).unwrap().hash());
            let confirmed_index = XGatewayBitcoin::confirmed_index().unwrap();
            assert_eq!(confirmed_index.hash, c1.get(2).unwrap().hash());
            should_in_mainchain(&c1[0..6], true);
            should_in_mainchain(&forked[2..5], false);
        });
}

#[test]
fn test_change_difficulty() {
    ExtBuilder::default().build_and_execute(|| {
        let headers = generate_blocks_63290_63310();
        let to_height = 63290 + 20;
        let current_difficulty = headers[&63291].bits;
        let new_difficulty = headers[&to_height].bits;
        println!(
            "current_difficulty: bit:{:?}|new_difficulty: bit:{:?}",
            current_difficulty, new_difficulty
        );
        for i in 63291..to_height {
            assert_ok!(XGatewayBitcoin::apply_push_header(headers[&i]));
        }
    })
}

#[test]
fn test_call() {
    ExtBuilder::default().build_and_execute(|| {
        let headers = generate_blocks_63290_63310();
        let origin = frame_system::RawOrigin::Signed(alice()).into();
        let v = serialization::serialize(&headers[&(63290 + 1)]);
        let v = v.take();
        assert_ok!(XGatewayBitcoin::push_header(origin, v));
    })
}

/*
这些测试用例是 ChainX 项目中用于验证比特币网关模块(`XGatewayBitcoin`)功能的单元测试.
它们使用 Substrate 框架的测试基础设施来模拟区块链的运行环境,并验证比特币区块头的处理逻辑.以下是对这些测试用例的解释:

### 测试用例解释

1. **`test_genesis`**: 验证创世区块的区块头和高度是否正确存储.这个测试检查了 ChainX 链的创世区块信息,并确保它与预期的哈希值和高度匹配.

2. **`test_insert_headers`**: 测试连续插入比特币区块头到 ChainX 链的情况.
这个测试模拟了接收一系列连续的区块头,并将它们推送到 ChainX 链上,同时检查最佳区块索引和主链状态是否正确更新.

3. **`test_insert_forked_headers_from_genesis_height`**: 测试从创世高度开始插入分叉区块头的情况.
这个测试模拟了一个分叉链的情况,其中分叉链的区块头被插入到 ChainX 链上,并验证了在分叉和回归到主链时 ChainX 链的状态是否正确更新.

4. **`test_insert_forked_headers`**: 类似于上一个测试,但这次是从非创世高度开始插入分叉区块头.这个测试进一步验证了 ChainX 链如何处理分叉和回归到主链的情况.

5. **`test_change_difficulty`**: 测试比特币网络难度变化的情况.这个测试通过插入一系列区块头来模拟比特币网络的难度变化,并验证 ChainX 链是否正确处理了这些变化.

6. **`test_call`**: 测试通过 RPC 调用插入比特币区块头的情况.这个测试模拟了通过外部调用将比特币区块头推送到 ChainX 链上的过程.

### 辅助函数

- **`should_in_mainchain`**: 一个辅助函数,用于检查给定的区块头列表是否都在 ChainX 链的主链中.

### 总结

这些测试用例确保了 ChainX 项目中的比特币网关模块能够正确地处理比特币区块头,无论是在正常操作还是在网络分叉的情况下.
通过这些测试,ChainX 项目可以确保其跨链桥接功能的安全性和可靠性,从而维护比特币资产在 ChainX 链上的正确表示.
*/
