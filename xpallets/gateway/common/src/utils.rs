// Copyright 2019-2023 ChainX Project Authors. Licensed under GPL-3.0.

pub const MAX_TAPROOT_NODES: u32 = 350;

/// equal or more than 2/3, return an unsigned integer
#[inline]
pub fn two_thirds(sum: u32) -> Option<u32> {
    2_u32
        .checked_mul(sum)
        .map(|m| if m % 3 == 0 { m / 3 } else { m / 3 + 1 })
}

#[inline]
pub fn two_thirds_unsafe(sum: u32) -> u32 {
    two_thirds(sum).expect("the params should not overflow; qed")
}

/*
这段代码提供了两个函数,用于计算给定数量的三分之二(2/3)的值,这在某些共识算法和加密货币协议中是一个常见的计算需求.

### 函数解释:

1. **two_thirds**:
   - 这是一个安全函数,它接受一个无符号整数 `sum` 作为参数,并返回一个 `Option<u32>` 类型的值.
   这个函数首先计算 `sum` 的三分之二,确保结果是一个整数.如果 `sum` 可以被 3 整除,那么它就直接除以 3;
   否则,它会加 1 后再除以 3,以确保结果是一个整数.这个函数使用 `checked_mul` 来避免在乘法过程中可能发生的
   溢出问题,并返回一个 `Option` 类型,以便在计算结果不正确时能够返回 `None`.

2. **two_thirds_unsafe**:
   - 这是一个不安全版本的函数,它同样接受一个无符号整数 `sum` 作为参数,但返回一个 `u32` 类型的值,
   而不是 `Option`.这个函数假设输入的 `sum` 不会导致溢出,并且总是期望得到一个有效的结果.如果 `sum` 的
   三分之二可以整除,它就会直接返回结果;否则,它会加 1 后再除以 3.如果在任何情况下 `two_thirds` 函数
   返回 `None`,`two_thirds_unsafe` 将会 panic,因为它使用 `expect` 来断言预期的 `Some` 值.

### 使用场景:

这些函数在需要确定阈值或在多方签名(multisig)场景中分配权重时非常有用.例如,在比特币的 Taproot 协议中,
可能需要计算一个多方签名地址所需的最小签名数量,以满足某个阈值要求.`two_thirds` 函数可以用来安全地执行这种计算,
而 `two_thirds_unsafe` 则在开发者确信输入不会导致溢出时使用,以简化代码逻辑.
*/
