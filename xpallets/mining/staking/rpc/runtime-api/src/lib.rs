// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! Runtime API definition required by ChainX RPC extensions.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments, clippy::unnecessary_mut_passed)]

use sp_std::{collections::btree_map::BTreeMap, prelude::*};

use codec::Codec;

pub use xpallet_mining_staking::{
    NominatorInfo, NominatorLedger, Unbonded, ValidatorInfo, ValidatorLedger, VoteWeight,
};

sp_api::decl_runtime_apis! {
    /// The API to query Staking info.
    pub trait XStakingApi<AccountId, Balance, VoteWeight, BlockNumber>
    where
        AccountId: Codec + Ord,
        Balance: Codec,
        VoteWeight: Codec,
        BlockNumber: Codec,
    {
        /// Get overall information about all potential validators.
        fn validators() -> Vec<ValidatorInfo<AccountId, Balance, VoteWeight, BlockNumber>>;

        /// Get overall information given the validator AccountId.
        fn validator_info_of(who: AccountId) -> ValidatorInfo<AccountId, Balance, VoteWeight, BlockNumber>;

        /// Get the staking dividends info given the staker AccountId.
        fn staking_dividend_of(who: AccountId) -> BTreeMap<AccountId, Balance>;

        /// Get the nomination details given the staker AccountId.
        fn nomination_details_of(who: AccountId) -> BTreeMap<AccountId, NominatorLedger<Balance, VoteWeight, BlockNumber>>;

        /// Get individual nominator information given the nominator AccountId.
        fn nominator_info_of(who: AccountId) -> NominatorInfo<BlockNumber>;
    }
}

/*
这段代码定义了ChainX区块链项目所需的运行时API接口,特别是与质押(Staking)相关的RPC扩展.
这些API接口允许外部客户端查询有关质押状态和奖励的信息.以下是代码的主要组成部分和它们的功能:

1. **`#![cfg_attr(not(feature = "std"), no_std)]`**:
   - 这是一个条件编译属性,指定当不启用`std`特性时,代码应该在`no_std`环境中编译.这通常适用于在WebAssembly (Wasm) 环境中运行的代码.

2. **`#![allow(clippy::too_many_arguments, clippy::unnecessary_mut_passed)]`**:
   - 这是一个编译警告抑制属性,用于忽略`clippy`工具报告的特定警告.在这里,它抑制了因为函数参数过多或传递了不必要的可变引用而产生的警告.

3. **依赖项**:
   - 引入了必要的外部依赖项,包括`sp_std`和`codec`库中的类型和特质.

4. **`sp_api::decl_runtime_apis!`宏**:
   - 这个宏定义了一个运行时API接口,它包含了一组可以在ChainX区块链上查询质押信息的方法.

5. **`XStakingApi` trait**:
   - 这是一个特质(trait),定义了`XStakingApi`接口.它包含以下方法:
     - `validators()`: 返回所有潜在验证者的综合信息.
     - `validator_info_of(who: AccountId)`: 根据验证者的账户ID,返回验证者的综合信息.
     - `staking_dividend_of(who: AccountId)`: 根据质押者的账户ID,返回质押分红信息.
     - `nomination_details_of(who: AccountId)`: 根据质押者的账户ID,返回提名详情.
     - `nominator_info_of(who: AccountId)`: 根据提名者的账户ID,返回提名者信息.

这个API为ChainX区块链的用户提供了一个标准化的方式来查询质押相关的数据,这对于开发基于ChainX的去中心化应用(DApps)和工具非常重要.
通过这些API,用户可以获取到验证者的状态,质押分红信息,提名详情和提名者信息,从而更好地理解和参与到ChainX的质押生态系统中.
*/
