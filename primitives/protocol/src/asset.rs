// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

use chainx_primitives::{AssetId, Decimals};

// match to SLIP-0044 Registered coin types for BIP-0044
// [Registered coin types](https://github.com/satoshilabs/slips/blob/master/slip-0044.md)
//
// Particular, ChainX Native token PCX occupies Testnet index, which is not same in SLIP44 standard.
// so that, ChainX AssetId protocol is:
//
// 1. base token:
//      base token stands for the real token for this Asset on ChainX, all have "X_" prefix, means
//      cross chain (e.g. BTC is X_BTC, ETH is X_ETH), and ths base token AssetId is from SLIP44
//      standard "coin type".
//      But inside, we agree on using Testnet index 1 to stand for **mainnet Bitcoin asset**,
//      not testnet. And on the other hand, we use 0 to stand for ChainX native token "PCX",
//      and others is all match to SLIP44 "coin type" index.
//
// 2. some token which not in SLIP44 coin types:
//      e.g. USDT not int SLIP44, thus we use `0x01000000 | id` to extend AssetId for containing
//      there assets. The AssetId in this part is decided by ChainX.
//      For example, we agree on pointing 0x01 as the USDT, thus USDT AssetId is `0x01000000|0x01`
//
// 3. derived token on ChainX for the cross chain token.
//      ChainX would derived some special token which just on ChainX and it is not real cross
//      assets but also have some relationship to source chain assets. Thus we use some
//      particular prefix to distinguish with base token.
//      (e.g. L_BTC means locked bitcoin, S_DOT means shadow DOT)
//      to distinguish with base token AssetId, we use `<Some Prefix>|<base token AssetId>` to
//      express the derived token. Different derived situation have different prefix.
//      thus we agree on the prefix:
//      L_: use 0x90000000
//      S_: use 0xa0000000

/// Native asset of ChainX.
pub const PCX: AssetId = 0;
/// Decimals of PCX, the native token of ChainX.
pub const PCX_DECIMALS: Decimals = 8;

/// BTC asset in ChainX backed by the Mainnet Bitcoin.
pub const X_BTC: AssetId = 1;
/// Decimals of BTC.
pub const BTC_DECIMALS: Decimals = 8;
/// Reserved since this symbol had been used in legacy ChainX 1.0.
pub const L_BTC: AssetId = 0x90000000 | X_BTC;

/// ETH asset in ChainX backed by the Mainnet Ethereum.
pub const X_ETH: AssetId = 60;

/// DOT asset in ChainX backed by the Mainnet Polkadot.
pub const X_DOT: AssetId = 354;
/// Reserved since this symbol had been used in legacy ChainX 1.0.
pub const S_DOT: AssetId = 0xa0000000 | X_DOT;

const EXTEND: AssetId = 0x01000000;
/// USDT asset in ChainX.
pub const USDT: AssetId = EXTEND | 0x01;

/*
这段代码是ChainX区块链项目中关于资产ID(AssetId)的定义和解释.
它详细说明了ChainX如何根据SLIP-0044标准和BIP-0044标准来分配和管理不同类型的资产.以下是对代码中各个部分的详细解释:

1. **ChainX AssetId协议**:
   - ChainX的AssetId协议遵循SLIP-0044标准中的"coin type"来定义基础代币(base token),
   所有基础代币都有"X_"前缀,表示跨链(例如,比特币是X_BTC,以太坊是X_ETH).
   - ChainX的原生代币PCX使用0来表示,而不是SLIP-0044标准中的测试网络索引.
   - 对于不在SLIP-0044中的代币(如USDT),ChainX使用`0x01000000 | id`的方式来扩展AssetId,其中`id`是ChainX自定义的值.

2. **基础代币的定义**:
   - PCX:ChainX的原生代币,使用0作为AssetId.
   - X_BTC:代表ChainX中由主网比特币支持的BTC资产,使用1作为AssetId.
   - X_ETH:代表ChainX中由主网以太坊支持的ETH资产,使用60作为AssetId.
   - X_DOT:代表ChainX中由主网波卡支持的DOT资产,使用354作为AssetId.

3. **衍生代币的定义**:
   - L_BTC:代表锁定的比特币,使用0x90000000与X_BTC的AssetId组合来表示.
   - S_DOT:代表影子DOT,使用0xa0000000与X_DOT的AssetId组合来表示.

4. **扩展代币的定义**:
   - EXTEND:用于扩展AssetId的值,表示非SLIP-0044标准的代币.
   - USDT:在ChainX中的USDT资产,使用EXTEND与自定义的ID(0x01)组合来表示.

这段代码的设计允许ChainX支持多种不同的资产,包括原生代币,通过SLIP-0044标准定义的代币,
以及不在SLIP-0044中但由ChainX自定义的代币.通过这种方式,ChainX能够提供一个统一和标准化的方式来管理和交易多种区块链资产.
*/
