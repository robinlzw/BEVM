// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

//! RPC interface for the transaction payment module.
#![allow(clippy::type_complexity)]
use std::collections::btree_map::BTreeMap;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::Arc;

use codec::Codec;
use jsonrpc_derive::rpc;

use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

use xp_rpc::{runtime_error_into_rpc_err, Result, RpcBalance, RpcVoteWeight};

use xpallet_mining_staking_rpc_runtime_api::{
    NominatorInfo, NominatorLedger, Unbonded, ValidatorInfo, ValidatorLedger,
    XStakingApi as XStakingRuntimeApi,
};

/// XStaking RPC methods.
#[rpc]
pub trait XStakingApi<BlockHash, AccountId, Balance, VoteWeight, BlockNumber>
where
    AccountId: Ord,
    Balance: Display + FromStr,
    VoteWeight: Display + FromStr,
{
    /// Get overall information about all potential validators
    #[rpc(name = "xstaking_getValidators")]
    fn validators(
        &self,
        at: Option<BlockHash>,
    ) -> Result<
        Vec<ValidatorInfo<AccountId, RpcBalance<Balance>, RpcVoteWeight<VoteWeight>, BlockNumber>>,
    >;

    /// Get overall information given the validator AccountId.
    #[rpc(name = "xstaking_getValidatorByAccount")]
    fn validator_info_of(
        &self,
        who: AccountId,
        at: Option<BlockHash>,
    ) -> Result<ValidatorInfo<AccountId, RpcBalance<Balance>, RpcVoteWeight<VoteWeight>, BlockNumber>>;

    /// Get the staking dividends info given the staker AccountId.
    #[rpc(name = "xstaking_getDividendByAccount")]
    fn staking_dividend_of(
        &self,
        who: AccountId,
        at: Option<BlockHash>,
    ) -> Result<BTreeMap<AccountId, RpcBalance<Balance>>>;

    /// Get the nomination details given the staker AccountId.
    #[rpc(name = "xstaking_getNominationByAccount")]
    fn nomination_details_of(
        &self,
        who: AccountId,
        at: Option<BlockHash>,
    ) -> Result<
        BTreeMap<
            AccountId,
            NominatorLedger<RpcBalance<Balance>, RpcVoteWeight<VoteWeight>, BlockNumber>,
        >,
    >;

    /// Get individual nominator information given the nominator AccountId.
    #[rpc(name = "xstaking_getNominatorByAccount")]
    fn nominator_info_of(
        &self,
        who: AccountId,
        at: Option<BlockHash>,
    ) -> Result<NominatorInfo<BlockNumber>>;
}

/// A struct that implements the [`XStakingApi`].
pub struct XStaking<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> XStaking<C, B> {
    /// Create new `Contracts` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId, Balance, VoteWeight, BlockNumber>
    XStakingApi<<Block as BlockT>::Hash, AccountId, Balance, VoteWeight, BlockNumber>
    for XStaking<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
    C::Api: XStakingRuntimeApi<Block, AccountId, Balance, VoteWeight, BlockNumber>,
    AccountId: Codec + Ord,
    Balance: Codec + Display + FromStr,
    VoteWeight: Codec + Display + FromStr,
    BlockNumber: Codec,
{
    fn validators(
        &self,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<
        Vec<ValidatorInfo<AccountId, RpcBalance<Balance>, RpcVoteWeight<VoteWeight>, BlockNumber>>,
    > {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.validators(&at)
            .map(|validators| {
                validators
                    .into_iter()
                    .map(|validator| ValidatorInfo {
                        account: validator.account,
                        profile: validator.profile,
                        ledger: ValidatorLedger {
                            total_nomination: validator.ledger.total_nomination.into(),
                            last_total_vote_weight: validator.ledger.last_total_vote_weight.into(),
                            last_total_vote_weight_update: validator
                                .ledger
                                .last_total_vote_weight_update,
                        },
                        is_validating: validator.is_validating,
                        self_bonded: validator.self_bonded.into(),
                        reward_pot_account: validator.reward_pot_account,
                        reward_pot_balance: validator.reward_pot_balance.into(),
                    })
                    .collect::<Vec<_>>()
            })
            .map_err(runtime_error_into_rpc_err)
    }

    fn validator_info_of(
        &self,
        who: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<ValidatorInfo<AccountId, RpcBalance<Balance>, RpcVoteWeight<VoteWeight>, BlockNumber>>
    {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.validator_info_of(&at, who)
            .map(|validator| ValidatorInfo {
                account: validator.account,
                profile: validator.profile,
                ledger: ValidatorLedger {
                    total_nomination: validator.ledger.total_nomination.into(),
                    last_total_vote_weight: validator.ledger.last_total_vote_weight.into(),
                    last_total_vote_weight_update: validator.ledger.last_total_vote_weight_update,
                },
                is_validating: validator.is_validating,
                self_bonded: validator.self_bonded.into(),
                reward_pot_account: validator.reward_pot_account,
                reward_pot_balance: validator.reward_pot_balance.into(),
            })
            .map_err(runtime_error_into_rpc_err)
    }

    fn staking_dividend_of(
        &self,
        who: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<BTreeMap<AccountId, RpcBalance<Balance>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.staking_dividend_of(&at, who)
            .map(|staking_dividend| {
                staking_dividend
                    .into_iter()
                    .map(|(account, balance)| (account, balance.into()))
                    .collect()
            })
            .map_err(runtime_error_into_rpc_err)
    }

    fn nomination_details_of(
        &self,
        who: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<
        BTreeMap<
            AccountId,
            NominatorLedger<RpcBalance<Balance>, RpcVoteWeight<VoteWeight>, BlockNumber>,
        >,
    > {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.nomination_details_of(&at, who)
            .map(|nomination_details| {
                nomination_details
                    .into_iter()
                    .map(|(account, nominator_ledger)| {
                        (
                            account,
                            NominatorLedger {
                                nomination: nominator_ledger.nomination.into(),
                                last_vote_weight: nominator_ledger.last_vote_weight.into(),
                                last_vote_weight_update: nominator_ledger.last_vote_weight_update,
                                unbonded_chunks: nominator_ledger
                                    .unbonded_chunks
                                    .into_iter()
                                    .map(|unbonded| Unbonded {
                                        value: unbonded.value.into(),
                                        locked_until: unbonded.locked_until,
                                    })
                                    .collect(),
                            },
                        )
                    })
                    .collect()
            })
            .map_err(runtime_error_into_rpc_err)
    }

    fn nominator_info_of(
        &self,
        who: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<NominatorInfo<BlockNumber>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        api.nominator_info_of(&at, who)
            .map_err(runtime_error_into_rpc_err)
    }
}

/*
这段代码定义了ChainX区块链项目的质押(Staking)模块的RPC(远程过程调用)接口.这个接口允许外部客户端查询有关
质押状态,验证者信息,质押分红和提名详情的数据.以下是代码的主要组成部分和它们的功能:

1. **`#![allow(clippy::type_complexity)]`**:
   - 这是一个编译警告抑制属性,用于忽略`clippy`工具报告的复杂类型警告.

2. **依赖项**:
   - 引入了必要的外部依赖项,包括`std`库中的类型,`codec`库用于序列化和反序列化,`jsonrpc_derive`库用于
   自动生成RPC方法,以及其他Substrate和ChainX项目中的类型和特质.

3. **`XStakingApi` trait**:
   - 定义了质押相关的RPC方法.这些方法允许客户端查询:
     - `validators`: 获取所有潜在验证者的综合信息.
     - `validator_info_of`: 根据验证者的账户ID,获取验证者的综合信息.
     - `staking_dividend_of`: 根据质押者的账户ID,获取质押分红信息.
     - `nomination_details_of`: 根据质押者的账户ID,获取提名详情.
     - `nominator_info_of`: 根据提名者的账户ID,获取提名者信息.

4. **`XStaking` struct**:
   - 实现了`XStakingApi` trait,提供了与ChainX区块链节点进行交互的实际RPC方法.它持有对客户端的引用,
   并使用`PhantomData`来指定它服务的区块链块类型.

5. **实现`XStakingApi` trait**:
   - 为`XStaking` struct提供了实际的RPC方法实现.这些方法使用客户端的运行时API来获取所需的信息,并将其转换为RPC结果类型.
   - 使用了`map_err`来将可能发生的运行时错误转换为RPC错误.

整体来看,这段代码为ChainX区块链提供了一个RPC接口,使得外部客户端可以查询质押相关的信息.这对于开发者来说是一个强大的工具,
因为它允许他们构建应用程序和服务,这些应用程序和服务可以基于区块链上的质押数据进行决策和交互.
*/
