use crate::{Config, GenesisConfig};

pub(crate) mod balances {
    use frame_support::traits::StoredMap;
    use pallet_balances::AccountData;
    use xp_genesis_builder::FreeBalanceInfo;

    use crate::Config;

    // Set PCX free balance.
    pub fn initialize<T: Config>(free_balances: &[FreeBalanceInfo<T::AccountId, T::Balance>]) {
        let set_free_balance = |who: &T::AccountId, free: &T::Balance| {
            T::AccountStore::insert(
                who,
                AccountData {
                    free: *free,
                    ..Default::default()
                },
            )
        };

        let mut total_issuance = T::Balance::default();

        for FreeBalanceInfo { who, free } in free_balances {
            let _ = set_free_balance(who, free);
            total_issuance += *free;
        }

        pallet_balances::TotalIssuance::<T>::mutate(|v| *v = total_issuance);
    }
}

pub(crate) mod xassets {
    // Set XBTC free balance.
    use xp_genesis_builder::FreeBalanceInfo;
    use xp_protocol::X_BTC;

    use super::*;
    use crate::AssetBalanceOf;

    pub fn initialize<T: Config>(xbtc_assets: &[FreeBalanceInfo<T::AccountId, AssetBalanceOf<T>>]) {
        for FreeBalanceInfo { who, free } in xbtc_assets {
            xpallet_assets::Pallet::<T>::force_set_free_balance(&X_BTC, who, *free);
        }
    }
}

pub(crate) mod xstaking {
    use xp_genesis_builder::{Nomination, NominatorInfo, XStakingParams};

    use super::*;
    use crate::StakingBalanceOf;

    // Simulate the bond operation.
    pub fn initialize<T: Config>(
        params: &XStakingParams<T::AccountId, StakingBalanceOf<T>>,
        initial_authorities: &[Vec<u8>],
    ) {
        let XStakingParams {
            validators,
            nominators,
        } = params;

        // Firstly register the genesis validators.
        xpallet_mining_staking::Pallet::<T>::initialize_validators(validators, initial_authorities)
            .expect("Failed to initialize genesis staking validators");

        // Then mock the validator bond themselves and set the vote weights.
        for NominatorInfo {
            nominator,
            nominations,
        } in nominators
        {
            for Nomination {
                nominee,
                nomination,
            } in nominations
            {
                xpallet_mining_staking::Pallet::<T>::force_set_nominator_vote_weight(
                    nominator,
                    nominee,
                    Default::default(),
                );
                xpallet_mining_staking::Pallet::<T>::force_bond(nominator, nominee, *nomination)
                    .expect("force validator self-bond can not fail; qed");
            }
        }
    }
}

pub(crate) mod xmining_asset {
    use xp_genesis_builder::FreeBalanceInfo;
    use xp_protocol::X_BTC;

    use super::*;
    use crate::AssetBalanceOf;

    // Set the weight related to zero.
    pub fn initialize<T: Config>(xbtc_assets: &[FreeBalanceInfo<T::AccountId, AssetBalanceOf<T>>]) {
        let current_block = frame_system::Pallet::<T>::block_number();

        for FreeBalanceInfo { who, .. } in xbtc_assets {
            xpallet_mining_asset::Pallet::<T>::force_set_miner_mining_weight(
                who,
                &X_BTC,
                Default::default(),
                current_block,
            );
        }

        xpallet_mining_asset::Pallet::<T>::force_set_asset_mining_weight(
            &X_BTC,
            Default::default(),
            current_block,
        );
    }
}

pub(crate) fn initialize<T: Config>(config: &GenesisConfig<T>) {
    let now = std::time::Instant::now();

    balances::initialize::<T>(&config.params.balances);
    xassets::initialize::<T>(&config.params.xassets);
    xstaking::initialize::<T>(&config.params.xstaking, &config.initial_authorities);
    xmining_asset::initialize::<T>(&config.params.xassets);

    frame_support::log::info!(
        "Took {:?}ms to orchestrate the regenesis state",
        now.elapsed().as_millis()
    );
}

/*
这段代码是一个Rust库的一部分,它提供了在Substrate框架中初始化创世状态(regenesis)的功能.
这个库允许ChainX区块链在升级或重置时设置创世状态的各种参数,包括账户余额,资产余额,质押状态和挖矿权重等.

代码的主要组成部分如下:

1. **`balances`模块**:
   - 这个模块负责初始化账户的自由余额(free balance).它使用`frame_support::traits::StoredMap`和
   `pallet_balances::AccountData`来存储和更新账户余额信息.
   - `initialize`函数接受一个`FreeBalanceInfo`数组,该数组包含了账户ID和对应的余额.它遍历这个数组,并为每个账户设置余额.

2. **`xassets`模块**:
   - 这个模块负责初始化X资产(如X-BTC)的自由余额.它使用`xp_genesis_builder`和`xp_protocol`中的定义来设置特定资产的余额.
   - `initialize`函数接受一个`FreeBalanceInfo`数组,该数组包含了账户ID和对应的X资产余额,并为每个账户设置X资产余额.

3. **`xstaking`模块**:
   - 这个模块负责模拟质押(staking)操作,包括初始化验证者(validators)和提名者(nominators)的信息.
   - `initialize`函数接受质押参数和初始验证者列表.它首先初始化验证者,然后为每个提名者设置投票权重,并模拟自我质押操作.

4. **`xmining_asset`模块**:
   - 这个模块负责设置与挖矿相关的权重.它使用`xp_genesis_builder`和`xp_protocol`中的定义来设置矿工的挖矿权重和资产的挖矿权重.
   - `initialize`函数接受一个`FreeBalanceInfo`数组,并为每个账户设置挖矿权重,同时设置整个资产的挖矿权重.

5. **`initialize`函数**:
   - 这是一个公共函数,它协调上述所有模块的初始化过程.它接受一个`GenesisConfig`对象,该对象包含了所有必要的创世参数.
   - 该函数依次调用其他模块的`initialize`函数,以设置余额,资产,质押和挖矿权重.
   - 函数执行完毕后,它会记录初始化所花费的时间.

整体来看,这段代码为ChainX区块链提供了一个灵活的创世状态初始化工具,允许在不同的区块链版本之间进行平滑的过渡.
*/
