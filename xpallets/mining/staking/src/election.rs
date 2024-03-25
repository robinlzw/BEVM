// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use super::*;
use frame_support::log;
use sp_std::vec::Vec;

impl<T: Config> Pallet<T> {
    /// Returns a new validator set for the new era.
    pub(crate) fn new_era(start_session_index: SessionIndex) -> Option<Vec<T::AccountId>> {
        // Increment or set current era.
        let current_era = CurrentEra::<T>::mutate(|s| {
            *s = Some(s.map(|s| s + 1).unwrap_or(0));
            s.unwrap()
        });
        ErasStartSessionIndex::<T>::insert(&current_era, &start_session_index);

        // Set staking information for new era.
        let maybe_new_validators = Self::select_and_update_validators(current_era);
        debug!(
            target: "runtime::mining::staking",
            "[new_era] era_index:{}, start_session_index:{}, maybe_new_validators:{:?}",
            current_era, start_session_index, maybe_new_validators
        );

        maybe_new_validators
    }

    /// Returns true if the (potential) validator is able to join in the election.
    ///
    /// Three requirements:
    /// 1. has the desire to win the election.
    /// 2. meets the threshold of a valid candidate.
    /// 3. has set session keys by calling pallet_session set_keys.
    fn is_qualified_candidate(who: &T::AccountId) -> bool {
        Self::is_active(who)
            && Self::meet_candidate_threshold(who)
            && T::ValidatorRegistration::is_registered(who)
    }

    /// Returns true if the candidate meets the minimum candidate threshold.
    ///
    /// Otherwise the candidate will be **forced to be chilled**.
    fn meet_candidate_threshold(who: &T::AccountId) -> bool {
        let BondRequirement { self_bonded, total } = Self::validator_candidate_requirement();
        let threshold_satisfied =
            Self::validator_self_bonded(who) >= self_bonded && Self::total_votes_of(who) >= total;

        if !threshold_satisfied && Self::try_force_chilled(who).is_ok() {
            log::info!(
                target: "runtime::mining::staking",
                "[meet_candidate_threshold] Force {:?} to be inactive since \
                it doesn't meet the minimum bond requirement", who);
        }

        threshold_satisfied
    }

    /// Filters out all the qualified validator candidates, sorted by the total nominations.
    fn filter_out_candidates() -> Vec<(BalanceOf<T>, T::AccountId)> {
        let mut candidates = Self::validator_set()
            .filter(Self::is_qualified_candidate)
            .map(|v| (Self::total_votes_of(&v), v))
            .collect::<Vec<_>>();
        candidates.sort_by(|&(ref b1, _), &(ref b2, _)| b2.cmp(b1));
        candidates
    }

    /// Selects the new validator set at the end of the era.
    ///
    /// Order potential validators by their total nominations and
    /// choose the top-most ValidatorCount::get() of them.
    ///
    /// This should only be called at the end of an era.
    fn select_and_update_validators(_current_era: EraIndex) -> Option<Vec<T::AccountId>> {
        // TODO: might move to offchain worker solution in the future.
        // Currently there is no performance issue practically.
        let candidates = Self::filter_out_candidates();
        debug!(
            target: "runtime::mining::staking",
            "[select_and_update_validators] candidates:{:?}", candidates
        );

        // Avoid reevaluate validator set if it would leave us with fewer than the minimum
        // needed validators.
        if candidates.len() < Self::reasonable_minimum_validator_count() as usize {
            return None;
        }

        let desired_validator_count = ValidatorCount::<T>::get() as usize;

        let validators = candidates
            .into_iter()
            .take(desired_validator_count)
            .map(|(_, v)| v)
            .collect::<Vec<_>>();

        // Always return Some(new_validators).
        Some(validators)
    }
}

/*
这段代码是一个Rust实现的Substrate智能合约框架中的模块代码片段,
用于处理ChainX区块链网络中的质押(Staking)和验证者(Validators)选举逻辑.以下是对代码中各个函数的详细解释:

1. `new_era`: 此函数用于在新的纪元(era)开始时初始化新的验证者集合.它首先更新当前纪元的计数,
并记录新纪元的开始会话索引.然后,它调用`select_and_update_validators`函数来选择新的验证者集合,
并记录这些信息.最后,它记录日志以提供有关新纪元和新验证者集合的信息.

2. `is_qualified_candidate`: 此函数检查一个潜在的验证者是否有资格参加选举.要成为一个合格的候选人,
必须满足三个条件:有意赢得选举,满足有效候选人的门槛,并且已经通过`pallet_session set_keys`调用设置了会话密钥.

3. `meet_candidate_threshold`: 此函数检查候选人是否满足最低候选门槛.如果不满足,候选人将被强制不活跃(chilled).
这有助于确保网络的安全性,通过确保只有那些有足够的自我质押和总投票数的候选人才能成为验证者.

4. `filter_out_candidates`: 此函数过滤并返回所有合格的验证者候选人,按照他们收到的总提名(votes)进行排序.
这个列表将用于最终选择新的验证者集合.

5. `select_and_update_validators`: 此函数在纪元结束时被调用,用于选择新的验证者集合.它首先过滤出所有合格的候选人,
然后根据他们的总提名数量选择前`ValidatorCount::get()`个候选人作为新的验证者集合.这个函数确保即使在新的纪元中,也不会因为验证者数量不足而影响网络的运行.

整体而言,这段代码是ChainX区块链网络中处理质押和验证者选举的核心逻辑,它确保了网络的去中心化和安全性,
同时提供了一种机制来激励持币者参与网络的维护.通过这种方式,ChainX能够保持其区块链网络的健康和稳定运行.
*/
