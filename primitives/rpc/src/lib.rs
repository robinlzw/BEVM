// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use std::{
    fmt::{Debug, Display},
    result::Result as StdResult,
    str::FromStr,
};

pub use jsonrpc_core::{Error, ErrorCode, Result};
use serde::{de, ser, Deserialize, Serialize};

/// The call to runtime failed.
pub const RUNTIME_ERROR: i64 = 1;

/// The call related to trustee to runtime failed.
const RUNTIME_TRUSTEE_ERROR: i64 = RUNTIME_ERROR + 100;

/// Decode the generic trustee info failed.
///
/// TODO: these pallet-specific errors should be moved to its own rpc module
/// when there are many of them.
pub const RUNTIME_TRUSTEE_DECODE_ERROR: i64 = RUNTIME_TRUSTEE_ERROR + 1;

/// The trustees are inexistent.
pub const RUNTIME_TRUSTEE_INEXISTENT_ERROR: i64 = RUNTIME_TRUSTEE_ERROR + 2;

/// The transaction was not decodable.
pub const DECODE_ERROR: i64 = 10000;

/// The bytes failed to be decoded as hex.
pub const DECODE_HEX_ERROR: i64 = DECODE_ERROR + 1;

/// Converts a runtime trap into an RPC error.
pub fn runtime_error_into_rpc_err(err: impl Debug) -> Error {
    Error {
        code: ErrorCode::ServerError(RUNTIME_ERROR),
        message: "Runtime trapped".into(),
        data: Some(format!("{:?}", err).into()),
    }
}

/// Converts a trustee runtime trap into an RPC error.
pub fn trustee_decode_error_into_rpc_err(err: impl Debug) -> Error {
    Error {
        code: ErrorCode::ServerError(RUNTIME_TRUSTEE_DECODE_ERROR),
        message: "Can not decode generic trustee session info".into(),
        data: Some(format!("{:?}", err).into()),
    }
}

/// Converts a trustee runtime trap into an RPC error.
pub fn trustee_inexistent_rpc_err() -> Error {
    Error {
        code: ErrorCode::ServerError(RUNTIME_TRUSTEE_INEXISTENT_ERROR),
        message: "Trustee does not exist".into(),
        data: None,
    }
}

/// Converts a hex decode error into an RPC error.
pub fn hex_decode_error_into_rpc_err(err: impl Debug) -> Error {
    Error {
        code: ErrorCode::ServerError(DECODE_HEX_ERROR),
        message: "Failed to decode hex".into(),
        data: Some(format!("{:?}", err).into()),
    }
}

/// Balance type when interacting with RPC.
pub type RpcBalance<Balance> = RpcU128<Balance>;

/// Price type of order when interacting with RPC.
pub type RpcPrice<Price> = RpcU128<Price>;

/// Weight type of mining when interacting with RPC.
pub type RpcMiningWeight<Weight> = RpcU128<Weight>;

/// Weight type of staking when interacting with RPC.
pub type RpcVoteWeight<Weight> = RpcU128<Weight>;

/// A helper struct for handling u128 serialization/deserialization of RPC.
/// See https://github.com/polkadot-js/api/issues/2464 for details (shit!).
#[derive(Eq, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct RpcU128<T: Display + FromStr>(#[serde(with = "self::serde_num_str")] T);

impl<T: Display + FromStr> From<T> for RpcU128<T> {
    fn from(value: T) -> Self {
        RpcU128(value)
    }
}

/// Number string serialization/deserialization
pub mod serde_num_str {
    use super::*;

    /// A serializer that encodes the number as a string
    pub fn serialize<S, T>(value: &T, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: ser::Serializer,
        T: Display,
    {
        serializer.serialize_str(&value.to_string())
    }

    /// A deserializer that decodes a string to the number.
    pub fn deserialize<'de, D, T>(deserializer: D) -> StdResult<T, D::Error>
    where
        D: de::Deserializer<'de>,
        T: FromStr,
    {
        let data = String::deserialize(deserializer)?;
        data.parse::<T>()
            .map_err(|_| de::Error::custom("Parse from string failed"))
    }
}

/// Hex serialization/deserialization
pub mod serde_hex {
    use super::*;

    /// A serializer that encodes the bytes as a hex-string
    pub fn serialize<T, S>(value: &T, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: ser::Serializer,
        T: AsRef<[u8]>,
    {
        serializer.serialize_str(&format!("0x{}", hex::encode(value)))
    }

    /// A deserializer that decodes the hex-string to bytes (Vec<u8>)
    pub fn deserialize<'de, D>(deserializer: D) -> StdResult<Vec<u8>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let data = String::deserialize(deserializer)?;
        let data = if let Some(stripped) = data.strip_prefix("0x") {
            stripped
        } else {
            &data[..]
        };
        let hex = hex::decode(data).map_err(de::Error::custom)?;
        Ok(hex)
    }
}

/// Text serialization/deserialization
pub mod serde_text {
    use super::*;

    /// A serializer that encodes the bytes as a string
    pub fn serialize<T, S>(value: &T, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: ser::Serializer,
        T: AsRef<[u8]>,
    {
        let output = String::from_utf8_lossy(value.as_ref());
        serializer.serialize_str(&output)
    }

    /// A deserializer that decodes the string to the bytes (Vec<u8>)
    pub fn deserialize<'de, D>(deserializer: D) -> StdResult<Vec<u8>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let data = String::deserialize(deserializer)?;
        Ok(data.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_serde_num_str_attr() {
        use super::RpcU128;

        let test = RpcU128(u128::max_value());
        let ser = serde_json::to_string(&test).unwrap();
        assert_eq!(ser, "\"340282366920938463463374607431768211455\"");
        let de = serde_json::from_str::<RpcU128<u128>>(&ser).unwrap();
        assert_eq!(de, test);
    }

    #[test]
    fn test_serde_hex_attr() {
        #[derive(PartialEq, Debug, Serialize, Deserialize)]
        struct HexTest(#[serde(with = "super::serde_hex")] Vec<u8>);

        let test = HexTest(b"0123456789".to_vec());
        let ser = serde_json::to_string(&test).unwrap();
        assert_eq!(ser, "\"0x30313233343536373839\"");
        let de = serde_json::from_str::<HexTest>(&ser).unwrap();
        assert_eq!(de, test);
        // without 0x
        let de = serde_json::from_str::<HexTest>("\"30313233343536373839\"").unwrap();
        assert_eq!(de, test);
    }

    #[test]
    fn test_serde_text_attr() {
        #[derive(PartialEq, Debug, Serialize, Deserialize)]
        struct TextTest(#[serde(with = "super::serde_text")] Vec<u8>);

        let test = TextTest(b"0123456789".to_vec());
        let ser = serde_json::to_string(&test).unwrap();
        assert_eq!(ser, "\"0123456789\"");
        let de = serde_json::from_str::<TextTest>(&ser).unwrap();
        assert_eq!(de, test);
    }
}

/*
这段代码是ChainX区块链项目中用于处理JSON-RPC(远程过程调用)序列化和反序列化的一组工具.
它定义了如何处理`u128`类型(一种128位无符号整数)的序列化,以及如何处理一些特定的错误情况.
以下是对代码中各个部分的详细解释:

1. **错误代码常量**:
   - `RUNTIME_ERROR`:表示调用运行时失败的错误代码.
   - `RUNTIME_TRUSTEE_ERROR`:表示与受托人相关的运行时调用失败的错误代码.
   - `RUNTIME_TRUSTEE_DECODE_ERROR`:表示解码受托人信息失败的错误代码.
   - `RUNTIME_TRUSTEE_INEXISTENT_ERROR`:表示受托人不存在的错误代码.
   - `DECODE_ERROR`:表示交易无法解码的错误代码.
   - `DECODE_HEX_ERROR`:表示字节无法解码为十六进制的错误代码.

2. **错误转换函数**:
   - `runtime_error_into_rpc_err`:将运行时错误转换为RPC错误.
   - `trustee_decode_error_into_rpc_err`:将受托人解码错误转换为RPC错误.
   - `trustee_inexistent_rpc_err`:返回受托人不存在的RPC错误.
   - `hex_decode_error_into_rpc_err`:将十六进制解码错误转换为RPC错误.

3. **RPC类型定义**:
   - `RpcBalance`,`RpcPrice`,`RpcMiningWeight`,`RpcVoteWeight`:
   这些类型定义了在与RPC交互时使用的余额,价格,挖矿权重和质押权重的类型.

4. **RpcU128结构体**:
   - 一个帮助结构体,用于处理`u128`类型的序列化和反序列化.它使用`serde`库的自定义序列化和反序列化器.

5. **序列化和反序列化模块**:
   - `serde_num_str`:提供了将数字序列化为字符串和从字符串反序列化为数字的自定义序列化器和反序列化器.
   - `serde_hex`:提供了将字节序列化为十六进制字符串和从十六进制字符串反序列化为字节的自定义序列化器和反序列化器.
   - `serde_text`:提供了将字节序列化为文本字符串和从文本字符串反序列化为字节的自定义序列化器和反序列化器.

6. **测试模块**:
   - 包含了一系列测试,用于验证序列化和反序列化工具的正确性.

这段代码的目的是为了确保在与ChainX区块链进行RPC通信时,能够正确地处理和传输`u128`类型的数据,
以及在出现错误时能够返回合适的错误信息.通过自定义序列化和反序列化器,ChainX可以确保与JSON-RPC接口的兼容性,
即使在处理大整数时也能保证数据的准确性和一致性.


*/

