use fp_evm::Context;
use pallet_evm::{Precompile, PrecompileResult, PrecompileSet};
use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_dispatch::Dispatch;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use sp_core::H160;
use sp_std::marker::PhantomData;

/// We include the nine Istanbul precompiles
/// (https://github.com/ethereum/go-ethereum/blob/3c46f557/core/vm/contracts.go#L69)
/// as well as a special precompile for dispatching Substrate extrinsics
pub struct ChainXPrecompiles<R>(PhantomData<R>);

impl<R> ChainXPrecompiles<R>
where
    R: pallet_evm::Config,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(PhantomData::<R>)
    }
    /// Return all addresses that contain precompiles. This can be used to populate dummy code
    /// under the precompile.
    pub fn used_addresses() -> sp_std::vec::Vec<H160> {
        sp_std::vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 1024, 1025, 1026, 1027]
            .into_iter()
            .map(hash)
            .collect()
    }
}

/// The following distribution has been decided for the precompiles
/// 0-1023: Ethereum Mainnet Precompiles
/// 1024-2047 Precompiles that are not in Ethereum Mainnet but are neither ChainX specific
/// 2048-4095 ChainX specific precompiles
impl<R> PrecompileSet for ChainXPrecompiles<R>
where
    R: xpallet_assets_bridge::Config
        + xpallet_gateway_common::Config
        + xpallet_gateway_records::Config,
    Dispatch<R>: Precompile,
{
    fn execute(
        &self,
        address: H160,
        input: &[u8],
        target_gas: Option<u64>,
        context: &Context,
        is_static: bool,
    ) -> Option<PrecompileResult> {
        match address {
            // Ethereum precompiles :
            a if a == hash(1) => Some(ECRecover::execute(input, target_gas, context, is_static)),
            a if a == hash(2) => Some(Sha256::execute(input, target_gas, context, is_static)),
            a if a == hash(3) => Some(Ripemd160::execute(input, target_gas, context, is_static)),
            a if a == hash(4) => Some(Identity::execute(input, target_gas, context, is_static)),
            a if a == hash(5) => Some(Modexp::execute(input, target_gas, context, is_static)),
            a if a == hash(6) => Some(Bn128Add::execute(input, target_gas, context, is_static)),
            a if a == hash(7) => Some(Bn128Mul::execute(input, target_gas, context, is_static)),
            a if a == hash(8) => Some(Bn128Pairing::execute(input, target_gas, context, is_static)),
            a if a == hash(9) => Some(Blake2F::execute(input, target_gas, context, is_static)),
            // Non-ChainX specific nor Ethereum precompiles :
            a if a == hash(1024) => {
                Some(Sha3FIPS256::execute(input, target_gas, context, is_static))
            }
            a if a == hash(1025) => Some(Dispatch::<R>::execute(
                input, target_gas, context, is_static,
            )),
            a if a == hash(1026) => Some(ECRecoverPublicKey::execute(
                input, target_gas, context, is_static,
            )),
            a if a == hash(1027) => Some(crate::withdraw::Withdraw::<R>::execute(
                input, target_gas, context, is_static,
            )),
            _ => None,
        }
    }
    fn is_precompile(&self, address: H160) -> bool {
        Self::used_addresses().contains(&address)
    }
}

fn hash(a: u64) -> H160 {
    H160::from_low_u64_be(a)
}

/*
这段代码是 ChainX 区块链中与以太坊虚拟机(EVM)预编译合约相关的部分.
预编译合约是一些在以太坊网络上有固定地址并提供特定功能的智能合约.ChainX 区块链集成了这些预编译合约,以便与以太坊兼容,并提供一些特定的功能.

以下是代码中定义的主要组件和它们的用途:

- `ChainXPrecompiles<R>`: 这是一个结构体,它包含了 ChainX 区块链中所有的预编译合约.
`PhantomData<R>` 是一个类型标记,表示这个结构体与 `pallet_evm::Config` 相关的某种运行时配置有关.

- `new()`: 一个构造函数,用于创建 `ChainXPrecompiles` 的新实例.

- `used_addresses()`: 一个方法,返回所有包含预编译合约的地址.这些地址是硬编码的,并且可以通过 `hash` 函数转换为 `H160` 类型的以太坊地址.

- `execute()`: 这是 `PrecompileSet` trait 的实现,它定义了如何执行预编译合约.根据传入的地址,它会调用相应的预编译合约函数.
例如,`ECRecover`,`Sha256`,`Identity` 等都是以太坊主网上的预编译合约.

- `is_precompile()`: 一个方法,用于检查给定的地址是否是一个预编译合约的地址.它通过检查 `used_addresses()` 返回的地址列表来确定.

- `hash()`: 一个辅助函数,将 `u64` 类型的数字转换为 `H160` 类型的以太坊地址.

代码中还提到了一个特定的地址分配方案:

- 0-1023 范围内的地址分配给了以太坊主网预编译合约.
- 1024-2047 范围内的地址分配给了非 ChainX 特定且不在以太坊主网上的预编译合约.
- 2048-4095 范围内的地址分配给了 ChainX 特定的预编译合约.

这种分配方案允许 ChainX 区块链在保持与以太坊兼容性的同时,还能够提供一些特定的,定制的功能.
通过这种方式,ChainX 能够支持以太坊智能合约的执行,同时扩展其功能以满足特定的需求.
*/
