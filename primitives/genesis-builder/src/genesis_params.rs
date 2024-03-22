use chainx_primitives::Balance;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AllParams<AccountId, TBalance, AssetBalanceOf, StakingBalanceOf> {
    pub balances: BalancesParams<AccountId, TBalance>,
    pub xassets: Vec<FreeBalanceInfo<AccountId, AssetBalanceOf>>,
    pub xstaking: XStakingParams<AccountId, StakingBalanceOf>,
    pub xmining_asset: XMiningAssetParams<AccountId>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FreeBalanceInfo<AccountId, Balance> {
    pub who: AccountId,
    pub free: Balance,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct WellknownAccounts<AccountId> {
    pub legacy_council: AccountId,
    pub legacy_pots: Vec<(AccountId, AccountId)>,
    pub legacy_xbtc_pot: AccountId,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BalancesParams<AccountId, Balance> {
    pub free_balances: Vec<FreeBalanceInfo<AccountId, Balance>>,
    pub wellknown_accounts: WellknownAccounts<AccountId>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo<AccountId, Balance> {
    pub who: AccountId,
    #[serde(with = "xp_rpc::serde_text")]
    pub referral_id: Vec<u8>,
    pub self_bonded: Balance,
    pub total_nomination: Balance,
    #[serde(with = "xp_rpc::serde_num_str")]
    pub total_weight: u128,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Nomination<AccountId, Balance> {
    pub nominee: AccountId,
    pub nomination: Balance,
    #[serde(with = "xp_rpc::serde_num_str")]
    pub weight: u128,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NominatorInfo<AccountId, Balance> {
    pub nominator: AccountId,
    pub nominations: Vec<Nomination<AccountId, Balance>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct XStakingParams<AccountId, Balance> {
    pub validators: Vec<ValidatorInfo<AccountId, Balance>>,
    pub nominators: Vec<NominatorInfo<AccountId, Balance>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct XBtcInfo {
    pub balance: Balance,
    #[serde(with = "xp_rpc::serde_num_str")]
    pub weight: u128,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct XBtcMiner<AccountId> {
    pub who: AccountId,
    #[serde(with = "xp_rpc::serde_num_str")]
    pub weight: u128,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct XMiningAssetParams<AccountId> {
    pub xbtc_miners: Vec<XBtcMiner<AccountId>>,
    pub xbtc_info: XBtcInfo,
}

/*
这段代码定义了ChainX区块链项目中与账户余额,质押(staking)和挖矿资产相关的一系列结构体.
这些结构体用于序列化和反序列化(通过`Serialize`和`Deserialize` trait),以便在网络中传输和存储相关数据.以下是对每个结构体的详细解释:

1. **AllParams结构体**:
   - 描述:包含所有与账户余额,资产余额,质押和挖矿资产相关的参数.
   - 字段:
     - balances:账户余额参数.
     - xassets:资产余额信息列表.
     - xstaking:质押参数.
     - xmining_asset:挖矿资产参数.

2. **FreeBalanceInfo结构体**:
   - 描述:包含账户的自由余额信息.
   - 字段:
     - who:账户ID.
     - free:账户的自由余额.

3. **WellknownAccounts结构体**:
   - 描述:包含一些知名的账户信息,如理事会账户,奖池账户等.
   - 字段:
     - legacy_council:旧理事会账户ID.
     - legacy_pots:旧奖池账户列表,每个奖池由一个账户ID对组成.
     - legacy_xbtc_pot:旧X-BTC奖池账户ID.

4. **BalancesParams结构体**:
   - 描述:包含账户余额参数.
   - 字段:
     - free_balances:自由余额信息列表.
     - wellknown_accounts:知名账户信息.

5. **ValidatorInfo结构体**:
   - 描述:包含验证者的信息,如账户ID,推荐ID,质押金额等.
   - 字段:
     - who:验证者账户ID.
     - referral_id:推荐ID,序列化为文本格式.
     - self_bonded:自我质押金额.
     - total_nomination:总提名金额.
     - total_weight:总权重,序列化为字符串格式.

6. **Nomination结构体**:
   - 描述:包含提名信息,如被提名者,提名金额和权重.
   - 字段:
     - nominee:被提名者账户ID.
     - nomination:提名金额.
     - weight:权重,序列化为字符串格式.

7. **NominatorInfo结构体**:
   - 描述:包含提名者的信息,包括提名者账户ID和提名列表.
   - 字段:
     - nominator:提名者账户ID.
     - nominations:提名列表.

8. **XStakingParams结构体**:
   - 描述:包含质押参数,如验证者信息和提名者信息.
   - 字段:
     - validators:验证者信息列表.
     - nominators:提名者信息列表.

9. **XBtcInfo结构体**:
   - 描述:包含X-BTC相关信息,如余额和权重.
   - 字段:
     - balance:X-BTC余额.
     - weight:X-BTC权重,序列化为字符串格式.

10. **XBtcMiner结构体**:
    - 描述:包含X-BTC矿工的信息,如账户ID和权重.
    - 字段:
      - who:矿工账户ID.
      - weight:权重,序列化为字符串格式.

11. **XMiningAssetParams结构体**:
    - 描述:包含挖矿资产参数,如X-BTC矿工列表和X-BTC信息.
    - 字段:
      - xbtc_miners:X-BTC矿工列表.
      - xbtc_info:X-BTC信息.

这些结构体的设计旨在支持ChainX区块链的多资产和多链特性,允许在区块链上管理和跟踪不同类型的资产和质押状态.
通过这些结构体,ChainX能够实现复杂的资产管理和质押机制,为用户和开发者提供灵活的区块链功能.
*/