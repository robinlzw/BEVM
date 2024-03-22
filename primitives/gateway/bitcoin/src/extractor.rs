// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use crate::types::OpReturnAccount;
use frame_support::log::{debug, error};
use sp_core::crypto::AccountId32;
use sp_std::prelude::Vec;

use chainx_primitives::ReferralId;
use xp_gateway_common::{
    from_ss58_check, transfer_aptos_uncheck, transfer_evm_uncheck, transfer_named_uncheck,
};

pub use xp_gateway_common::AccountExtractor;

/// A helper struct that implements the `AccountExtractor` trait for Bitcoin OP_RETURN data.
///
/// OP_RETURN data format:
/// - `account`, e.g. 5VEW3R1T4LR3kDhYwXeeCnYrHRwRaH7E9V1KprypBe68XmY4
/// - `account@referral`, e.g. 5VEW3R1T4LR3kDhYwXeeCnYrHRwRaH7E9V1KprypBe68XmY4@referral1
#[derive(PartialEq, Eq, Clone)]
pub struct OpReturnExtractor;

impl AccountExtractor<AccountId32, ReferralId> for OpReturnExtractor {
    fn extract_account(data: &[u8]) -> Option<(OpReturnAccount<AccountId32>, Option<ReferralId>)> {
        let account_and_referral = data
            .split(|x| *x == b'@')
            .map(|d| d.to_vec())
            .collect::<Vec<_>>();

        if account_and_referral.is_empty() {
            error!(
                "[extract_account] Can't extract account from data:{:?}",
                hex::encode(data)
            );
            return None;
        }

        let wasm_account = from_ss58_check(account_and_referral[0].as_slice());

        let account = if let Some(v) = wasm_account {
            OpReturnAccount::Wasm(v)
        } else if let Some(v) = transfer_evm_uncheck(account_and_referral[0].as_slice()) {
            OpReturnAccount::Evm(v)
        } else if let Some(v) = transfer_aptos_uncheck(account_and_referral[0].as_slice()) {
            OpReturnAccount::Aptos(v)
        } else {
            let data = transfer_named_uncheck(account_and_referral[0].as_slice())?;
            OpReturnAccount::Named(data.0, data.1)
        };

        let referral = if account_and_referral.len() > 1 {
            Some(account_and_referral[1].to_vec())
        } else {
            None
        };

        debug!(
            "[extract_account] account:{:?}, referral:{:?}",
            account, referral
        );
        Some((account, referral))
    }
}

#[test]
fn test_opreturn_extractor() {
    use sp_core::{
        crypto::{set_default_ss58_version, Ss58AddressFormatRegistry, UncheckedInto},
        H160, H256,
    };

    let addr = "f778a69d4166401048acb0f7b2625e9680609f8859c78e3d28e2549f84f0269a"
        .parse::<H256>()
        .unwrap();
    let mainnet = Ss58AddressFormatRegistry::ChainxAccount.into();
    let testnet = Ss58AddressFormatRegistry::SubstrateAccount.into();

    {
        set_default_ss58_version(mainnet);

        // test for account
        let result = OpReturnExtractor::extract_account(
            "5VEW3R1T4LR3kDhYwXeeCnYrHRwRaH7E9V1KprypBe68XmY4".as_bytes(),
        );
        assert_eq!(
            result,
            Some((OpReturnAccount::Wasm(addr.unchecked_into()), None))
        );

        // test for account and referral
        let result = OpReturnExtractor::extract_account(
            "5VEW3R1T4LR3kDhYwXeeCnYrHRwRaH7E9V1KprypBe68XmY4@referral1".as_bytes(),
        );
        assert_eq!(
            result,
            Some((
                OpReturnAccount::Wasm(addr.unchecked_into()),
                Some(b"referral1".to_vec())
            ))
        );

        let mut key = [0u8; 20];
        key.copy_from_slice(&hex::decode("3800501939F9385CB044F9FB992b97442Cc45e47").unwrap());
        let evm_addr = H160::try_from(key).unwrap();

        let result = OpReturnExtractor::extract_account(
            "0x3800501939F9385CB044F9FB992b97442Cc45e47@referral1".as_bytes(),
        );
        assert_eq!(
            result,
            Some((OpReturnAccount::Evm(evm_addr), Some(b"referral1".to_vec())))
        );

        let result = OpReturnExtractor::extract_account(
            "3800501939F9385CB044F9FB992b97442Cc45e47@referral1".as_bytes(),
        );
        assert_eq!(
            result,
            Some((OpReturnAccount::Evm(evm_addr), Some(b"referral1".to_vec())))
        );

        let mut key = [0u8; 32];
        key.copy_from_slice(
            &hex::decode("eeff357ea5c1a4e7bc11b2b17ff2dc2dcca69750bfef1e1ebcaccf8c8018175b")
                .unwrap(),
        );
        let aptos_addr = H256::try_from(key).unwrap();

        let result = OpReturnExtractor::extract_account(
            "0xeeff357ea5c1a4e7bc11b2b17ff2dc2dcca69750bfef1e1ebcaccf8c8018175b@referral1"
                .as_bytes(),
        );
        assert_eq!(
            result,
            Some((
                OpReturnAccount::Aptos(aptos_addr),
                Some(b"referral1".to_vec())
            ))
        );

        let result = OpReturnExtractor::extract_account(
            "eeff357ea5c1a4e7bc11b2b17ff2dc2dcca69750bfef1e1ebcaccf8c8018175b@referral1".as_bytes(),
        );
        assert_eq!(
            result,
            Some((
                OpReturnAccount::Aptos(aptos_addr),
                Some(b"referral1".to_vec())
            ))
        );

        let name = vec![b's', b'u', b'i'];
        let addr = hex::decode("1dcba11f07596152cf96a9bd358b675d5d5f9506").unwrap();

        let result = OpReturnExtractor::extract_account(
            "sui:0x1dcba11f07596152cf96a9bd358b675d5d5f9506@referral1".as_bytes(),
        );
        assert_eq!(
            result,
            Some((
                OpReturnAccount::Named(name.clone(), addr.clone()),
                Some(b"referral1".to_vec())
            ))
        );

        let result = OpReturnExtractor::extract_account(
            "sui:1dcba11f07596152cf96a9bd358b675d5d5f9506@referral1".as_bytes(),
        );
        assert_eq!(
            result,
            Some((
                OpReturnAccount::Named(name, addr),
                Some(b"referral1".to_vec())
            ))
        );
    }
    {
        set_default_ss58_version(testnet);

        // test for version
        let result = OpReturnExtractor::extract_account(
            "5VEW3R1T4LR3kDhYwXeeCnYrHRwRaH7E9V1KprypBe68XmY4".as_bytes(),
        );
        #[cfg(feature = "ss58check")]
        assert_eq!(
            result,
            Some((OpReturnAccount::Wasm(addr.unchecked_into()), None))
        );
        #[cfg(not(feature = "ss58check"))]
        assert_eq!(
            result,
            Some((OpReturnAccount::Wasm(addr.unchecked_into()), None))
        );
    }
    {
        // test for checksum
        set_default_ss58_version(testnet);

        let addr = "00308187439ac204df9e299e1e54a00000000bf348e03dad679737c91871dc53"
            .parse::<H256>()
            .unwrap();

        // old checksum
        let result = OpReturnExtractor::extract_account(
            "5C4xGQZwoNEM5mdk2U3vJbFZPr6ZKFSiqWnc9JRDcJ3w2x5D".as_bytes(),
        );

        // would check ss58version
        #[cfg(feature = "ss58check")]
        assert_eq!(result, None);
        // would not check ss58 version and hash checksum
        #[cfg(not(feature = "ss58check"))]
        assert_eq!(
            result,
            Some((OpReturnAccount::Wasm(addr.unchecked_into()), None))
        );

        // new checksum
        let result = OpReturnExtractor::extract_account(
            "5C4xGQZwoNEM5mdk2U3vJbFZPr6ZKFSiqWnc9JRDcJ3w334p".as_bytes(),
        );
        assert_eq!(
            result,
            Some((OpReturnAccount::Wasm(addr.unchecked_into()), None))
        );
    }
}

/*
这段代码是ChainX项目的一部分,它定义了一个名为`OpReturnExtractor`的结构体,该结构体实现了`AccountExtractor` trait,
用于从比特币的OP_RETURN数据中提取账户信息.OP_RETURN是比特币脚本中的一种输出类型,
它允许在交易中嵌入额外的数据,这些数据不会消耗比特币但可以用于存储信息,例如跨链交易中的账户信息.

以下是代码中各个部分的详细解释:

1. **导入依赖**:代码开始部分导入了所需的模块,包括日志记录,加密,地址格式和字节串处理等.

2. **OpReturnExtractor结构体**:这是一个辅助结构体,用于处理比特币OP_RETURN数据.
它实现了`AccountExtractor` trait,该trait定义了如何从数据中提取账户信息.

3. **OP_RETURN数据格式**:OP_RETURN数据可以包含账户信息和推荐ID(referral ID).
账户信息可以是ChainX的SS58地址,EVM地址,Aptos地址或其他命名账户.

4. **提取账户信息的方法**:`extract_account`方法接受一个包含OP_RETURN数据的字节切片,
并尝试从中提取账户信息.如果数据包含`@`符号,则认为它还包含一个推荐ID.
该方法会尝试解析SS58检查地址,EVM地址,Aptos地址和命名账户,并将结果作为`OpReturnAccount`枚举类型返回.

5. **日志记录**:在提取过程中,如果无法从数据中提取账户信息,会记录一个错误日志.成功提取信息时,会记录一个调试日志.

6. **测试函数**:`test_opreturn_extractor`函数包含了多个测试案例,
用于验证`OpReturnExtractor`是否能够正确处理不同类型的OP_RETURN数据.
测试案例包括了对ChainX账户,EVM账户,Aptos账户和命名账户的提取,以及带有推荐ID的情况.

7. **测试配置**:测试函数中使用了`set_default_ss58_version`来设置默认的SS58地址格式,
这影响了账户信息提取的方式.测试还考虑了是否启用了SS58检查功能,以及如何处理带有校验和的地址.

整体而言,这段代码为ChainX区块链提供了一种机制,使其能够有效地从比特币交易中的OP_RETURN数据提取账户信息,
这对于跨链资产转移和用户资金管理至关重要.通过这种方式,ChainX能够确保资产的安全转移,并为用户提供准确的账户信息.

这段代码是一个测试模块,用于验证`OpReturnExtractor`结构体是否能够正确地从不同类型的OP_RETURN数据中提取账户信息和推荐ID(referral ID).以下是带有注释的代码:

```rust
// 定义一个H256类型的地址,用于测试
let addr = "f778a69d4166401048acb0f7b2625e9680609f8859c78e3d28e2549f84f0269a"
    .parse::<H256>() // 将十六进制字符串解析为H256类型
    .unwrap();
// 定义ChainX主网和测试网的SS58地址格式
let mainnet = Ss58AddressFormatRegistry::ChainxAccount.into();
let testnet = Ss58AddressFormatRegistry::SubstrateAccount.into();

{
    // 设置默认的SS58地址格式为ChainX主网
    set_default_ss58_version(mainnet);

    // 测试提取ChainX账户
    let result = OpReturnExtractor::extract_account(
        "5VEW3R1T4LR3kDhYwXeeCnYrHRwRaH7E9V1KprypBe68XmY4".as_bytes(),
    );
    // 断言提取的结果是否符合预期
    assert_eq!(
        result,
        Some((OpReturnAccount::Wasm(addr.unchecked_into()), None))
    );

    // 测试提取ChainX账户和推荐ID
    let result = OpReturnExtractor::extract_account(
        "5VEW3R1T4LR3kDhYwXeeCnYrHRwRaH7E9V1KprypBe68XmY4@referral1".as_bytes(),
    );
    assert_eq!(
        result,
        Some((
            OpReturnAccount::Wasm(addr.unchecked_into()),
            Some(b"referral1".to_vec())
        ))
    );

    // 定义一个EVM地址
    let mut key = [0u8; 20];
    key.copy_from_slice(&hex::decode("3800501939F9385CB044F9FB992b97442Cc45e47").unwrap());
    let evm_addr = H160::try_from(key).unwrap();

    // 测试提取EVM账户和推荐ID
    let result = OpReturnExtractor::extract_account(
        "0x3800501939F9385CB044F9FB992b97442Cc45e47@referral1".as_bytes(),
    );
    assert_eq!(
        result,
        Some((OpReturnAccount::Evm(evm_addr), Some(b"referral1".to_vec())))
    );

    // 测试提取EVM账户和推荐ID(不包含前缀0x)
    let result = OpReturnExtractor::extract_account(
        "3800501939F9385CB044F9FB992b97442Cc45e47@referral1".as_bytes(),
    );
    assert_eq!(
        result,
        Some((OpReturnAccount::Evm(evm_addr), Some(b"referral1".to_vec())))
    );

    // 定义一个Aptos地址
    let mut key = [0u8; 32];
    key.copy_from_slice(
        &hex::decode("eeff357ea5c1a4e7bc11b2b17ff2dc2dcca69750bfef1e1ebcaccf8c8018175b")
            .unwrap(),
    );
    let aptos_addr = H256::try_from(key).unwrap();

    // 测试提取Aptos账户和推荐ID
    let result = OpReturnExtractor::extract_account(
        "0xeeff357ea5c1a4e7bc11b2b17ff2dc2dcca69750bfef1e1ebcaccf8c8018175b@referral1"
            .as_bytes(),
    );
    assert_eq!(
        result,
        Some((
            OpReturnAccount::Aptos(aptos_addr),
            Some(b"referral1".to_vec())
        ))
    );

    // 测试提取Aptos账户和推荐ID(不包含前缀0x)
    let result = OpReturnExtractor::extract_account(
        "eeff357ea5c1a4e7bc11b2b17ff2dc2dcca69750bfef1e1ebcaccf8c8018175b@referral1".as_bytes(),
    );
    assert_eq!(
        result,
        Some((
            OpReturnAccount::Aptos(aptos_addr),
            Some(b"referral1".to_vec())
        ))
    );

    // 定义一个命名账户和地址
    let name = vec![b's', b'u', b'i'];
    let addr = hex::decode("1dcba11f07596152cf96a9bd358b675d5d5f9506").unwrap();

    // 测试提取命名账户和推荐ID
    let result = OpReturnExtractor::extract_account(
        "sui:0x1dcba11f07596152cf96a9bd358b675d5d5f9506@referral1".as_bytes(),
    );
    assert_eq!(
        result,
        Some((
            OpReturnAccount::Named(name.clone(), addr.clone()),
            Some(b"referral1".to_vec())
        ))
    );

    // 测试提取命名账户和推荐ID(不包含前缀sui:)
    let result = OpReturnExtractor::extract_account(
        "sui:1dcba11f07596152cf96a9bd358b675d5d5f9506@referral1".as_bytes(),
    );
    assert_eq!(
        result,
        Some((
            OpReturnAccount::Named(name, addr),
            Some(b"referral1".to_vec())
        ))
    );
}
```

这个测试模块通过一系列的断言来验证`OpReturnExtractor`的行为是否符合预期.
它涵盖了不同类型的账户(ChainX,EVM,Aptos和命名账户)以及带有推荐ID的情况.
这些测试确保了`OpReturnExtractor`能够在处理OP_RETURN数据时正确地提取相关信息.

*/
