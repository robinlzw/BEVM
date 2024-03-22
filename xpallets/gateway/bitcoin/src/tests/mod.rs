// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

mod header;
mod trustee;
mod tx;

use sp_core::crypto::{set_default_ss58_version, Ss58AddressFormatRegistry};

use xp_gateway_common::AccountExtractor;

use light_bitcoin::script::Script;

use crate::{
    mock::{Test, XGatewayBitcoin},
    Config,
};

#[test]
pub fn test_verify_btc_address() {
    let address = b"mqVznxoxdeSNYgDCg6ZVE5pc6476BY6zHK".to_vec();
    assert!(XGatewayBitcoin::verify_btc_address(&address).is_ok());
}

#[test]
fn test_account_ss58_version() {
    set_default_ss58_version(Ss58AddressFormatRegistry::ChainxAccount.into());
    let script = Script::from(
        "5Uj3ehamDZWPfgA8iAZenhcAmPDakjf4aMbkBB4dXVvjoW6x@33"
            .as_bytes()
            .to_vec(),
    );
    let data = script.to_bytes();
    assert!(<Test as Config>::AccountExtractor::extract_account(&data).is_some());
}

/*
这两个测试用例是 ChainX 项目中的单元测试,用于验证比特币网关模块(`XGatewayBitcoin`)的地址验证和 SS58 地址格式转换功能.

### 测试用例解释

1. **`test_verify_btc_address`**: 此测试用例用于验证 `XGatewayBitcoin` 模块是否能够正确地验证比特币地址.
测试通过提供一个比特币地址的字节表示,并调用 `verify_btc_address` 函数来检查该地址是否有效.如果地址格式正确,测试将通过.

2. **`test_account_ss58_version`**: 此测试用例用于验证 SS58 地址格式转换功能.SS58 是 Substrate 地址格式的标准,
它允许将不同类型的地址(如比特币地址)转换为 Substrate 兼容的地址格式.测试首先设置默认的 SS58 地址版本为 ChainX 账户地址格式,
然后创建一个 `light_bitcoin` 库中的 `Script` 对象,该对象包含了一个比特币支付地址.接着,
测试使用 `AccountExtractor` trait 提供的 `extract_account` 方法来从 `Script` 对象中提取账户信息.如果能够成功提取,说明地址转换功能正常工作.

### 辅助函数

- **`set_default_ss58_version`**: 这个函数用于设置默认的 SS58 地址格式版本.在 Substrate 框架中,SS58 地址可以有不同的版本,对应不同的区块链或网络.

- **`AccountExtractor`**: 这是 `xp_gateway_common` 模块中的一个 trait,它定义了如何从一个给定的数据源(如比特币脚本)中提取账户信息.

### 总结

这些测试用例确保了 ChainX 项目中的比特币网关模块能够正确处理比特币地址,并且能够将比特币地址转换为 Substrate 兼容的 SS58 地址格式.
这对于跨链资产转移和用户账户管理至关重要,因为它确保了在 ChainX 链上可以正确识别和处理来自比特币网络的地址.
*/
