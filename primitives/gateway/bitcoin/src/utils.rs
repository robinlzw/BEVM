// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use frame_support::log::{error, warn};
use sp_std::{cmp::Ordering, prelude::Vec};

use light_bitcoin::{
    chain::{Transaction, TransactionOutput},
    keys::{Address, Network},
    script::{Opcode, Script, ScriptType},
};

/// Extract address from a transaction output specified by outpoint_index.
pub fn extract_addr_from_transaction(
    tx: &Transaction,
    outpoint_index: usize,
    network: Network,
) -> Option<Address> {
    tx.outputs
        .get(outpoint_index)
        .and_then(|output| extract_output_addr(output, network))
}

/// Extract address from a transaction output script.
/// only support `p2pk`, `p2pkh` and `p2sh` output script
pub fn extract_output_addr(output: &TransactionOutput, network: Network) -> Option<Address> {
    let script = Script::new(output.script_pubkey.clone());

    // only support `p2pk`, `p2pkh` and `p2sh` script
    let script_type = script.script_type();
    let script_addresses = script
        .extract_destinations()
        .map_err(|err| {
            error!(
                        "[extract_output_addr] Can't extract destinations of btc script err:{}, type:{:?}, script:{}",
                        err, script_type, script
                    );
        }).unwrap_or_default();
    if script_addresses.len() != 1 {
        warn!(
            "[extract_output_addr] Can't extract address of btc script, type:{:?}, address:{:?}, script:{}",
            script_addresses, script_type, script
        );
        return None;
    }
    let address = &script_addresses[0];
    match script_type {
        ScriptType::PubKey
        | ScriptType::PubKeyHash
        | ScriptType::ScriptHash
        | ScriptType::WitnessV0Keyhash
        | ScriptType::WitnessV0Scripthash
        | ScriptType::WitnessV1Taproot => {
            // find address in this transaction
            Some(Address {
                network,
                kind: address.kind,
                hash: address.hash,
            })
        }
        _ => None,
    }
}

/// Check if the `addr` is hot trustee address or cold trustee address.
pub fn is_trustee_addr(addr: Address, trustee_pair: (Address, Address)) -> bool {
    let (hot_addr, cold_addr) = trustee_pair;
    addr.hash == hot_addr.hash || addr.hash == cold_addr.hash
}

/// Extract the opreturn data from btc null data script.
/// OP_RETURN format:
/// - op_return + op_push(<0x4c) + data (op_push == data.len())
/// - op_return + op_push(=0x4c) + data.len() + data
pub fn extract_opreturn_data(script: &Script) -> Option<Vec<u8>> {
    if !script.is_null_data_script() {
        return None;
    }

    // jump `OP_RETURN`, after checking `is_null_data_script`
    // subscript = `op_push + data` or `op_push + data.len() + data`
    let subscript = script.subscript(1);
    if subscript.is_empty() {
        error!("[parse_opreturn] Nothing after `OP_RETURN`, valid in rule but invalid for public consensus");
        return None;
    }

    // parse op_push and data.
    let op_push = subscript[0];
    match op_push.cmp(&(Opcode::OP_PUSHDATA1 as u8)) {
        Ordering::Less => {
            // OP_RETURN format: op_return + op_push(<0x4c) + data (op_push == data.len())
            if subscript.len() < 2 {
                error!(
                    "[parse_opreturn] Nothing after `OP_PUSHDATA1`, invalid opreturn script:{:?}",
                    script
                );
                return None;
            }
            let data = &subscript[1..];
            if op_push as usize == data.len() {
                Some(data.to_vec())
            } else {
                error!("[parse_opreturn] Unexpected opreturn source error, expected data len:{}, actual data:{:?}", op_push, data);
                None
            }
        }
        Ordering::Equal => {
            // OP_RETURN format: op_return + op_push(=0x4c) + data.len() + data
            //
            // if op_push == `OP_PUSHDATA1`, we must have extra byte for the length of data,
            // otherwise it's an invalid data.
            if subscript.len() < 3 {
                error!(
                    "[parse_opreturn] Nothing after `OP_PUSHDATA1`, invalid opreturn script: {:?}",
                    script
                );
                return None;
            }
            let data_len = subscript[1];
            let data = &subscript[2..];
            if data_len as usize == data.len() {
                Some(data.to_vec())
            } else {
                error!("[parse_opreturn] Unexpected opreturn source error, expected data len:{}, actual data:{:?}", data_len, data);
                None
            }
        }
        Ordering::Greater => {
            error!(
                "[parse_opreturn] Unexpected opreturn source error, \
                opreturn format should be `op_return+op_push+data` or `op_return+op_push+data_len+data`, \
                op_push: {:?}", op_push
            );
            None
        }
    }
}

#[test]
fn test_extract_opreturn_data() {
    // tx: 6b2bea220fdecf30ae3d0e0fa6770f06f281999f81d485ebfc15bdf375268c59
    // null data script: 6a 30 35524745397a4a79667834367934467948444a65317976394e44725946435446746e6e6d714e445077506a6877753871
    let script = "6a3035524745397a4a79667834367934467948444a65317976394e44725946435446746e6e6d714e445077506a6877753871".parse::<Script>().unwrap();
    let data = extract_opreturn_data(&script).unwrap();
    assert_eq!(
        data,
        b"5RGE9zJyfx46y4FyHDJe1yv9NDrYFCTFtnnmqNDPwPjhwu8q".to_vec()
    );

    // tx: 003e7e005b172fe0046fd06a83679fbcdc5e3dd64c8ef9295662a463dea486aa
    // null data script: 6a 38 35515a5947565655507370376362714755634873524a555a726e6d547545796836534c48366a6470667346786770524b404c616f63697573
    let script = "6a3835515a5947565655507370376362714755634873524a555a726e6d547545796836534c48366a6470667346786770524b404c616f63697573".parse::<Script>().unwrap();
    let data = extract_opreturn_data(&script).unwrap();
    assert_eq!(
        data,
        b"5QZYGVVUPsp7cbqGUcHsRJUZrnmTuEyh6SLH6jdpfsFxgpRK@Laocius".to_vec()
    );
}

/*
这段代码是ChainX项目中用于处理比特币交易脚本和提取相关信息的一组函数.下面是对每个函数的详细解释:

1. **extract_addr_from_transaction**:
   - 功能:从比特币交易中提取指定输出索引处的地址.
   - 参数:交易对象,输出索引,网络类型(如主网或测试网).
   - 返回值:一个可选的地址,如果输出存在且能够提取地址则为`Some(Address)`,否则为`None`.

2. **extract_output_addr**:
   - 功能:从比特币交易输出脚本中提取地址.
   - 参数:交易输出对象,网络类型.
   - 返回值:一个可选的地址,仅当脚本类型为`p2pk`,`p2pkh`,`p2sh`时有效,否则为`None`.
   - 注意:此函数只支持特定的输出脚本类型,并且会记录错误和警告信息.

3. **is_trustee_addr**:
   - 功能:检查给定的地址是否是热钱包或冷钱包受托人地址.
   - 参数:待检查的地址,受托人地址对.
   - 返回值:如果地址匹配热钱包或冷钱包地址,则为`true`,否则为`false`.

4. **extract_opreturn_data**:
   - 功能:从比特币的OP_RETURN null数据脚本中提取数据.
   - 参数:脚本对象.
   - 返回值:如果脚本是有效的OP_RETURN脚本,则返回包含数据的`Vec<u8>`,否则为`None`.
   - 注意:此函数检查脚本是否符合OP_RETURN格式,并解析出数据.

5. **test_extract_opreturn_data**:
   - 功能:测试`extract_opreturn_data`函数是否能正确提取OP_RETURN脚本中的数据.
   - 参数:无,使用硬编码的脚本字符串进行测试.
   - 行为:对两个不同的OP_RETURN脚本进行解析,并断言提取的数据与预期相符.

这些函数对于ChainX项目中的比特币网关功能至关重要,因为它们用于解析比特币交易,
识别交易类型(如存款或提款),并从中提取用户账户信息.这有助于ChainX在处理跨链交易时保持准确性和安全性.

*/
