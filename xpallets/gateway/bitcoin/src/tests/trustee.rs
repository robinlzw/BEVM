// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use frame_support::assert_noop;
use hex_literal::hex;
use sp_std::convert::TryInto;

use light_bitcoin::{
    crypto::dhash160,
    keys::{Address, AddressTypes, Network, Public, Type},
    mast::Mast,
    script::{Builder, Opcode},
};

use xpallet_gateway_common::traits::TrusteeForChain;

use crate::{
    mock::{ExtBuilder, Test, XGatewayBitcoin, XGatewayBitcoinErr},
    trustee::create_multi_address,
};

#[test]
pub fn test_check_trustee_entity() {
    ExtBuilder::default().build_and_execute(|| {
        let addr_ok_3 = hex!("0311252930af8ba766b9c7a6580d8dc4bbf9b0befd17a8ef7fabac275bba77ae40");
        let public3 = Public::from_slice(&addr_ok_3).unwrap();
        assert_eq!(XGatewayBitcoin::check_trustee_entity(&addr_ok_3).unwrap().0, public3);
        let addr_ok_2 = hex!("0211252930af8ba766b9c7a6580d8dc4bbf9b0befd17a8ef7fabac275bba77ae40");
        let public2 = Public::from_slice(&addr_ok_2).unwrap();
        assert_eq!(XGatewayBitcoin::check_trustee_entity(&addr_ok_2).unwrap().0, public2);

        let addr_too_long = hex!("0311252930af8ba766b9c7a6580d8dc4bbf9b0befd17a8ef7fabac275bba77ae40cc");
        assert_noop!(XGatewayBitcoin::check_trustee_entity(&addr_too_long), XGatewayBitcoinErr::InvalidPublicKey);
        let addr_normal = hex!("0311252930af8ba766b9c7a6580d8dc4bbf9b0befd17a8ef7fabac275bba77ae4011252930af8ba766b9c7a6580d8dc4bbf9b0befd17a8ef7fabac275bba77ae40");
        assert_noop!(XGatewayBitcoin::check_trustee_entity(&addr_normal), XGatewayBitcoinErr::InvalidPublicKey);
        let addr_err_type = hex!("0411252930af8ba766b9c7a6580d8dc4bbf9b0befd17a8ef7fabac275bba77ae40");
        assert_noop!(XGatewayBitcoin::check_trustee_entity(&addr_err_type), XGatewayBitcoinErr::InvalidPublicKey);
        let addr_zero = hex!("020000000000000000000000000000000000000000000000000000000000000000");
        assert_noop!(XGatewayBitcoin::check_trustee_entity(&addr_zero), XGatewayBitcoinErr::InvalidPublicKey);
        let addr_ec_p = hex!("02fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f");
        assert_noop!(XGatewayBitcoin::check_trustee_entity(&addr_ec_p), XGatewayBitcoinErr::InvalidPublicKey);
        let addr_ec_p_2 = hex!("02fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc3f");
        assert_noop!(XGatewayBitcoin::check_trustee_entity(&addr_ec_p_2), XGatewayBitcoinErr::InvalidPublicKey);
    })
}

#[test]
pub fn test_multi_address() {
    let pubkey1_bytes = hex!("0311252930af8ba766b9c7a6580d8dc4bbf9b0befd17a8ef7fabac275bba77ae40");
    let pubkey2_bytes = hex!("02e34d10113f2dd162e8d8614a4afbb8e2eb14eddf4036042b35d12cf5529056a2");
    let pubkey3_bytes = hex!("023e505c48a955e759ce61145dc4a9a7447425290b8483f4e36f05169e7967c86d");

    let script = Builder::default()
        .push_opcode(Opcode::OP_2)
        .push_bytes(&pubkey1_bytes)
        .push_bytes(&pubkey2_bytes)
        .push_bytes(&pubkey3_bytes)
        .push_opcode(Opcode::OP_3)
        .push_opcode(Opcode::OP_CHECKMULTISIG)
        .into_script();
    let multisig_address = Address {
        kind: Type::P2SH,
        network: Network::Testnet,
        hash: AddressTypes::Legacy(dhash160(&script)),
    };
    assert_eq!(
        "2MtAUgQmdobnz2mu8zRXGSTwUv9csWcNwLU",
        multisig_address.to_string()
    );
}

#[test]
fn test_create_multi_address() {
    let mut hot_keys = Vec::new();
    let pubkey1_bytes = hex!("03f72c448a0e59f48d4adef86cba7b278214cece8e56ef32ba1d179e0a8129bdba");
    let pubkey2_bytes = hex!("0306117a360e5dbe10e1938a047949c25a86c0b0e08a0a7c1e611b97de6b2917dd");
    let pubkey3_bytes = hex!("0311252930af8ba766b9c7a6580d8dc4bbf9b0befd17a8ef7fabac275bba77ae40");
    let pubkey4_bytes = hex!("0227e54b65612152485a812b8856e92f41f64788858466cc4d8df674939a5538c3");
    hot_keys.push(Public::from_slice(&pubkey1_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey2_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey3_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey4_bytes).unwrap());

    let mut cold_keys = Vec::new();
    let pubkey5_bytes = hex!("02a79800dfed17ad4c78c52797aa3449925692bc8c83de469421080f42d27790ee");
    let pubkey6_bytes = hex!("03ece1a20b5468b12fd7beda3e62ef6b2f6ad9774489e9aff1c8bc684d87d70780");
    let pubkey7_bytes = hex!("02e34d10113f2dd162e8d8614a4afbb8e2eb14eddf4036042b35d12cf5529056a2");
    let pubkey8_bytes = hex!("020699bf931859cafdacd8ac4d3e055eae7551427487e281e3efba618bdd395f2f");
    cold_keys.push(Public::from_slice(&pubkey5_bytes).unwrap());
    cold_keys.push(Public::from_slice(&pubkey6_bytes).unwrap());
    cold_keys.push(Public::from_slice(&pubkey7_bytes).unwrap());
    cold_keys.push(Public::from_slice(&pubkey8_bytes).unwrap());

    ExtBuilder::default().build_and_execute(|| {
        let hot_info = create_multi_address::<Test>(&hot_keys, 3).unwrap();
        let cold_info = create_multi_address::<Test>(&cold_keys, 3).unwrap();
        let real_hot_addr = b"2N6mJFLkjN9muneSeHCsMCxWXVZ4ruLKfFo".to_vec();
        let real_cold_addr = b"2NEf17iYn2Lj2AdnAd1C7A9i8a5PpfPjaFk".to_vec();
        assert_eq!(hot_info.addr, real_hot_addr);
        assert_eq!(cold_info.addr, real_cold_addr);

        let pks = [
            169, 20, 148, 73, 171, 119, 43, 210, 235, 251, 157, 63, 210, 43, 149, 44, 127, 46, 221,
            9, 204, 107, 135,
        ];
        let addr: Address = String::from_utf8_lossy(&hot_info.addr).parse().unwrap();
        let pk = match addr.hash {
            AddressTypes::Legacy(h) => h.as_bytes().to_vec(),
            AddressTypes::WitnessV0ScriptHash(_) => todo!(),
            AddressTypes::WitnessV0KeyHash(_) => todo!(),
            AddressTypes::WitnessV1Taproot(_) => todo!(),
        };
        let mut pubkeys = vec![Opcode::OP_HASH160 as u8, Opcode::OP_PUSHBYTES_20 as u8];
        pubkeys.extend_from_slice(&pk);
        pubkeys.push(Opcode::OP_EQUAL as u8);
        assert_eq!(pubkeys, pks);
    });
}
#[test]
fn test_create_taproot_address() {
    let mut hot_keys = Vec::new();
    let pubkey1_bytes = hex!("0283f579dd2380bd31355d066086e1b4d46b518987c1f8a64d4c0101560280eae2");
    let pubkey2_bytes = hex!("027a0868a14bd18e2e45ff3ad960f892df8d0edd1a5685f0a1dc63c7986d4ad55d");
    let pubkey3_bytes = hex!("02c9929543dfa1e0bb84891acd47bfa6546b05e26b7a04af8eb6765fcc969d565f");
    let pubkey4_bytes = hex!("0237322a5008a1b26ac72778167e770e1fa2272cfd9f9fe0f2c20bd41fe051da6c");
    let pubkey5_bytes = hex!("03227368d7168173229f1898b8448dc5c0640ce35eb574639e42ec515b73d5f2d3");
    let pubkey6_bytes = hex!("027196048a63ec7a3b9cb5a23a51952503e7fca8de2ec42388952e067d39fc83ff");
    let pubkey7_bytes = hex!("025b9cd3170511ced44caf8067b1a759dfd7b2f2d52352c4a788b15a1cbc3afa56");
    let pubkey8_bytes = hex!("030709034ebd0964796a11fab08fce940524bd7dfdbd99b8a7a7618b71754e7584");
    let pubkey9_bytes = hex!("02f8c11e20a30a0683539ea579725eced3e055a4dcbef88f69162805f3dc609760");
    let pubkey10_bytes = hex!("0379f12ca4c0fb587616aba27ab7f66245120b4e1f83a413769a5779af48146e87");
    hot_keys.push(Public::from_slice(&pubkey1_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey2_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey3_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey4_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey5_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey6_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey7_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey8_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey9_bytes).unwrap());
    hot_keys.push(Public::from_slice(&pubkey10_bytes).unwrap());
    ExtBuilder::default().build_and_execute(|| {
        let pks = hot_keys
            .into_iter()
            .map(|k| k.try_into().unwrap())
            .collect::<Vec<_>>();
        let mast = Mast::new(pks, 7_u32).unwrap();
        let threshold_addr: Address = mast
            .generate_address(&crate::Pallet::<Test>::network_id().to_string())
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(mast.pubkeys.len(), 120);
        assert_eq!(
            threshold_addr.to_string(),
            "tb1psaktm6w6nrh5xs8umla9qaw6zjarr4yuqk3m4x8pzc6ekve93v7ss20kuq"
        )
    })
}

/*
这些测试用例是 ChainX 项目中用于验证比特币网关模块(`XGatewayBitcoin`)的地址验证和多签名地址创建功能的单元测试.

### 测试用例解释

1. **`test_check_trustee_entity`**: 此测试用例用于验证 `XGatewayBitcoin` 模块是否能够正确地检查比特币地址是否有效,
并且能否从地址中提取公钥.测试包括了多种情况,包括有效地址,过长地址,普通地址,错误类型地址,全零地址,以及无限大的椭圆曲线点地址.

2. **`test_multi_address`**: 此测试用例用于验证能否正确创建一个多签名(P2SH)比特币地址.它通过构建一个包含三个公钥的脚本,
并将其哈希转换为 P2SH 地址,然后与预期的地址字符串进行比较.

3. **`test_create_multi_address`**: 此测试用例用于验证 `create_multi_address` 函数是否能够正确创建多签名地址.
它提供了一组热钱包和冷钱包的公钥,并创建了相应的多签名地址.然后,测试将生成的地址与预期的地址进行比较,以确保它们匹配.

4. **`test_create_taproot_address`**: 此测试用例用于验证能否正确创建一个 Taproot 地址.它使用一组公钥创建了一个 `Mast` 实例,
并生成了一个 Taproot 地址.然后,测试验证了公钥的数量,并检查生成的地址是否与预期的地址字符串匹配.

### 辅助函数

- **`hex!`**: 宏用于将十六进制字符串转换为字节数组.

- **`Public::from_slice`**: 函数用于从字节切片中创建一个 `Public` 类型的实例.

- **`Address`**: 结构体用于表示比特币地址,包括地址类型,网络和哈希.

- **`Mast`**: 类型用于表示一个多签名地址结构,它可以生成 Taproot 地址.

### 总结

这些测试用例确保了 ChainX 项目中的比特币网关模块能够正确处理比特币地址,包括验证地址的有效性,创建多签名地址以及生成 Taproot 地址.
这对于跨链资产管理和交易至关重要,因为它确保了在 ChainX 链上可以正确识别和处理来自比特币网络的地址.
*/
