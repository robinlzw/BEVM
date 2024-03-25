// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use frame_support::traits::LockIdentifier;

pub const STAKING_ID: LockIdentifier = *b"staking ";

/// Session reward of the first 210_000 sessions.
///
/// ChainX uses a Bitcoin like issuance model, the initial reward is 50 PCX.
pub const INITIAL_REWARD: u64 = 5_000_000_000;

/// ChainX uses a Bitcoin like issuance model, issuing a fixed total of 21 million.
pub const FIXED_TOTAL: u64 = 2_100_000_000_000_000;

/// The maximum number of Staking validators.
///
/// Currently the election will perform a naive sort on the all candidates,
/// so we don't want the candidate list too huge.
pub const DEFAULT_MAXIMUM_VALIDATOR_COUNT: u32 = 1000;

/// The maximum number of ongoing unbonded operations in parallel.
pub const DEFAULT_MAXIMUM_UNBONDED_CHUNK_SIZE: u32 = 10;

/// ChainX 2.0's block time is targeted at 6s, i.e., 5 minutes per session.
///
/// ChainX 1.0 is 2s/block, 150 blocks/session, the duration of each session is also
/// 5 minutes, therefore the issuance rate stays the same in terms of the time dimension,
/// the daily Staking earnings does not change.
pub const DEFAULT_BLOCKS_PER_SESSION: u64 = 50;

/// The default bonding duration for regular staker is 3 days.
///
/// The staker can unbond the staked balances, but these balances will be free immediately,
/// they have to wait for 3 days to withdraw them into the free balances.
pub const DEFAULT_BONDING_DURATION: u64 = DEFAULT_BLOCKS_PER_SESSION * 12 * 24 * 3;

/// The default bonding duration for validator is 3 * 10 days.
pub const DEFAULT_VALIDATOR_BONDING_DURATION: u64 = DEFAULT_BONDING_DURATION * 10;

/*
这段代码是使用Rust语言编写的,它定义了一些与ChainX项目相关的常量和配置.
1. `STAKEING_ID`: 这是一个`LockIdentifier`类型的常量,用于标识Staking(质押)操作.
在区块链中,质押是一种常见的机制,允许持币者锁定他们的代币以支持网络的安全性,并可能获得奖励.

2. `INITIAL_REWARD`: 这是ChainX网络初始的区块奖励,设置为50 PCX(ChainX的本地代币).

3. `FIXED_TOTAL`: 这是ChainX网络发行的总代币量上限,固定为2100万PCX.这与比特币的总量上限类似,旨在控制通货膨胀.

4. `DEFAULT_MAXIMUM_VALIDATOR_COUNT`: 这是ChainX网络中选举产生的验证者(Validators)的最大数量.
验证者负责创建新区块并维护网络的安全性.这个常量设置为1000,意味着最多有1000个验证者.

5. `DEFAULT_MAXIMUM_UNBONDED_CHUNK_SIZE`: 这是并行进行的未绑定(Unbonded)操作的最大数量.
未绑定操作是指用户从质押状态中解除代币的操作.

6. `DEFAULT_BLOCKS_PER_SESSION`: 这是ChainX 2.0版本中每个会话(Session)的区块数量,
目标是每5分钟一个会话,与ChainX 1.0的区块时间相同,保持发行速率不变.

7. `DEFAULT_BONDING_DURATION`: 这是普通质押者(非验证者)的默认绑定(Bonding)持续时间,设置为3天.
在这个时间内,质押者可以解除质押,但是需要等待3天后才能将代币提取到可用余额中.

8. `DEFAULT_VALIDATOR_BONDING_DURATION`: 这是验证者的默认绑定持续时间,是普通质押者的10倍,即30天.

这些常量和配置对于ChainX网络的运行至关重要,它们定义了网络的货币政策,质押机制和验证者选举等核心特性.
*/
