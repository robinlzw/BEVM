// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

pub trait MultisigAddressFor<AccountId> {
    fn calc_multisig(accounts: &[AccountId], threshold: u16) -> AccountId;
}

impl<AccountId: Default> MultisigAddressFor<AccountId> for () {
    fn calc_multisig(_: &[AccountId], _: u16) -> AccountId {
        Default::default()
    }
}

pub trait MultiSig<AccountId: PartialEq> {
    fn multisig() -> AccountId;
}

pub trait Validator<AccountId> {
    fn is_validator(who: &AccountId) -> bool;

    fn validator_for(name: &[u8]) -> Option<AccountId>;
}

impl<AccountId> Validator<AccountId> for () {
    fn is_validator(_: &AccountId) -> bool {
        false
    }

    fn validator_for(_: &[u8]) -> Option<AccountId> {
        None
    }
}

/// This trait provides a simple way to get the treasury account.
pub trait TreasuryAccount<AccountId> {
    fn treasury_account() -> Option<AccountId>;
}

impl<AccountId> TreasuryAccount<AccountId> for () {
    fn treasury_account() -> Option<AccountId> {
        None
    }
}

/*
这段代码定义了几个用于区块链系统中的多签名地址计算,多签名验证和国库账户检索的trait(特质),以及它们的默认实现.
这些trait和实现通常用于区块链项目的模块开发中,以便在不依赖特定运行时环境的情况下提供通用的功能.
下面是对每个trait及其默认实现的详细解释:

1. **MultisigAddressFor**:
   - 这个trait定义了一个名为`calc_multisig`的函数,它接受一个账户ID的切片和一个阈值作为参数,
   并返回一个多签名地址(`AccountId`类型).这个地址是通过某种算法(未在代码中指定)从提供的账户ID和阈值计算得出的.
   - 默认实现简单地返回一个默认的`AccountId`.在实际的区块链项目中,这个函数通常会根据特定的多签名方案(如ECDSA,Schnorr等)和账户ID集合来计算多签名地址.

2. **MultiSig**:
   - 这个trait定义了一个名为`multisig`的函数,它返回一个多签名地址(`AccountId`类型).这个地址通常代表一个多签名钱包或合约.
   - 默认实现返回`None`,意味着没有定义多签名地址.在实际的项目中,这个函数会被用来检索或生成多签名地址.

3. **Validator**:
   - 这个trait定义了两个函数:`is_validator`用于检查给定的账户ID是否是验证者,`validator_for`用于根据名称检索验证者的账户ID.
   - 默认实现中,`is_validator`总是返回`false`,表示没有任何账户是验证者;`validator_for`总是返回`None`,
   表示没有账户与给定的名称关联.在实际的区块链项目中,这些函数会根据共识机制和验证者集合来实现具体的逻辑.

4. **TreasuryAccount**:
   - 这个trait定义了一个名为`treasury_account`的函数,它返回国库账户的`AccountId`.国库账户通常用于存储区块链系统的财政收入,如交易费,奖励等.
   - 默认实现返回`None`,表示没有定义国库账户.在实际的区块链项目中,这个函数会被用来检索或生成国库账户.

这些trait和它们的默认实现提供了一个框架,允许开发者在不同的区块链项目中重用和定制这些功能.通过为这些trait提供具体的实现,
开发者可以确保他们的模块能够与项目的其他部分(如多签名方案,验证者集合和国库管理)无缝集成.
*/
