use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeBalanceInfo<AccountId, Balance> {
    pub free: Balance,
    pub who: AccountId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nomination<AccountId, Balance> {
    pub nominee: AccountId,
    pub nomination: Balance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NominatorInfo<AccountId, Balance> {
    pub nominator: AccountId,
    pub nominations: Vec<Nomination<AccountId, Balance>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo<AccountId, Balance> {
    pub who: AccountId,
    #[serde(with = "xp_rpc::serde_text")]
    pub referral_id: Vec<u8>,
    pub total_nomination: Balance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XStakingParams<AccountId, Balance> {
    pub validators: Vec<ValidatorInfo<AccountId, Balance>>,
    pub nominators: Vec<NominatorInfo<AccountId, Balance>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllParams<AccountId, Balance, AssetBalanceOf, StakingBalanceOf> {
    pub balances: Vec<FreeBalanceInfo<AccountId, Balance>>,
    pub xassets: Vec<FreeBalanceInfo<AccountId, AssetBalanceOf>>,
    pub xstaking: XStakingParams<AccountId, StakingBalanceOf>,
}

impl<AccountId, Balance, AssetBalanceOf, StakingBalanceOf> Default
    for AllParams<AccountId, Balance, AssetBalanceOf, StakingBalanceOf>
{
    fn default() -> Self {
        AllParams {
            balances: vec![],
            xassets: vec![],
            xstaking: XStakingParams {
                validators: vec![],
                nominators: vec![],
            },
        }
    }
}

/*
这段代码定义了ChainX区块链项目中与账户余额,提名(nomination)和验证者(validator)信息相关的结构体.
1. **FreeBalanceInfo结构体**:
   - 描述:包含特定账户的自由余额信息.
   - 字段:
     - free:账户的自由余额.
     - who:账户ID.

2. **Nomination结构体**:
   - 描述:包含提名信息,即一个账户对另一个账户的提名.
   - 字段:
     - nominee:被提名的账户ID.
     - nomination:提名金额.

3. **NominatorInfo结构体**:
   - 描述:包含提名者的信息,包括其账户ID和提名列表.
   - 字段:
     - nominator:提名者账户ID.
     - nominations:提名列表,包含多个提名信息.

4. **ValidatorInfo结构体**:
   - 描述:包含验证者的信息,如账户ID和收到的总提名金额.
   - 字段:
     - who:验证者账户ID.
     - referral_id:推荐ID,通常是一个序列号化的文本字符串.
     - total_nomination:收到的总提名金额.

5. **XStakingParams结构体**:
   - 描述:包含质押参数,如验证者信息和提名者信息.
   - 字段:
     - validators:验证者信息列表.
     - nominators:提名者信息列表.

6. **AllParams结构体**:
   - 描述:包含所有相关的参数,包括账户余额,资产余额和质押信息.
   - 字段:
     - balances:账户余额信息列表.
     - xassets:资产余额信息列表.
     - xstaking:质押参数.

7. **Default trait的实现**:
   - 描述:为`AllParams`结构体实现了`Default` trait,提供了一个默认构造函数.
   - 行为:当创建`AllParams`的实例而没有提供参数时,它将创建一个所有字段均为空的实例.

这些结构体的设计旨在支持ChainX区块链的多资产和质押特性,允许在区块链上管理和跟踪不同类型的资产和质押状态.
通过这些结构体,ChainX能够实现复杂的资产管理和质押机制,为用户和开发者提供灵活的区块链功能.


在Layer 2解决方案中,"提名"(Nomination)通常指的是一种质押机制,其中用户或验证者选择支持或委托其质押资产给其他验证者或节点.
这种机制常见于各种Layer 2扩展解决方案,特别是在那些采用权益证明(Proof of Stake, PoS)或
委托权益证明(Delegated Proof of Stake, DPoS)共识算法的系统中.

Layer 2提名的内涵包括以下几个方面:

1. **质押委托**:用户可以将他们的代币或资产委托给其他验证者,而不是自己运行一个节点.
这样,他们可以参与到网络的质押和治理中,同时不需要直接参与验证过程.

2. **治理参与**:提名允许用户通过选择支持哪些验证者来参与到Layer 2网络的治理中.
用户可以根据自己的判断或信任的指标(如验证者的声誉,历史表现,奖励率等)来选择提名对象.

3. **收益分享**:提名者通常会从被提名验证者的奖励中获得一部分作为回报.
这意味着用户可以通过提名获得潜在的质押收益,即使他们没有直接运行验证者节点.

4. **网络安全性**:提名机制有助于分散网络的验证权力,增加网络的去中心化程度.
通过允许用户选择他们信任的验证者,提名机制鼓励了网络中的多样性和竞争,从而提高了整体的网络安全性.

5. **流动性提供**:在某些Layer 2解决方案中,提名还可以作为提供流动性的一种方式,
特别是在去中心化交易所(DEX)或其他流动性池中.用户通过提名可以为市场提供流动性,并从中获得交易费用等回报.

6. **技术进步**:随着Layer 2技术的发展,提名机制也在不断演进,以提供更高效,更安全,更用户友好的质押体验.
例如,一些Layer 2解决方案可能引入了无需信任的提名或跨链提名等创新概念.

总的来说,在Layer 2解决方案中,提名是一种重要的质押和治理机制,它允许用户以灵活的方式参与到网络中,
同时为网络的安全性和去中心化提供支持.随着区块链技术的进步,我们可以预见提名机制将继续发展,以适应不断变化的市场需求和用户期望.
*/