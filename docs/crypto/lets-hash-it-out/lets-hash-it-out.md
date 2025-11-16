---
title: Crypto - Let's hash it out
description: 2023 | ZKHACK | Crypto
---

[TOC]


## 1. Puzzle Description

> It all feels random, but it might not be.

[https://zkhack.dev/events/puzzle1.html](https://zkhack.dev/events/puzzle1.html)

## 2. Preliminary

### BLS 签名方案

BLS 签名方案基于椭圆曲线配对（pairing）实现，由三个核心算法组成：

- **Key Generation**：选择素数阶循环群的生成元 $g$，随机选择私钥 $sk \in GF(r)$，然后生成公钥 $pk = sk \cdot g$。

- **Sign**: 给定消息 $m \in \mathbb{G}$，签名为：$\Sigma = sk \cdot m$。

- **Verify**: 验证签名的有效性，即检查 $e(m, pk) = e(g, \Sigma)$ 是否成立。

### Hash-to-curve 操作

BLS 签名方案只能处理群元素作为消息，而实际消息是任意长度的字符串。因此，需要以下两步映射过程：

- 将任意长度的字符串转换为固定长度的哈希值（例如 blake2s）；
- 将该哈希值进一步映射到群元素上（使用 Pedersen Hash 等方法）。

Hash:

```rust
use blake2s_simd::blake2s;

fn main() {
    let msg = "I love you";
    let msg_bytes = msg.as_bytes();
    let hash = blake2s_simd::blake2s(msg_bytes);
    println!("{:?}", hash);
}
```

to Curve: 将哈希值映射到群元素一般使用 Pedersen Hash：

```rust
use pairing::bls12_381::*;
use ark_crypto_primitives::crh::pedersen::*;
use blake2s_simd::blake2s;
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;

#[derive(Clone)]
struct PedersenWindow;

impl Window for PedersenWindow {
    const WINDOW_SIZE: usize = 128;
    const NUM_WINDOWS: usize = 2;
}

fn hash_to_curve(msg: &[u8]) -> G1Affine {
    let rng = &mut ChaCha20Rng::from_seed([1u8; 32]);
    let params = CRH::<G1Projective, PedersenWindow>::setup(rng).unwrap();
    let h = blake2s_simd::blake2s(msg);
    CRH::<G1Projective, PedersenWindow>::evaluate(&params, h.as_bytes()).unwrap()
}
```

## 3. Solution

本题关键是注意到 Pedersen Hash 的生成过程使用的是**固定的生成元集合**，且每个签名可看作对消息哈希比特向量与生成元向量的线性组合：

$$
\Sigma_i = sk \cdot \left(b_1(m_i) \cdot g_1 + b_2(m_i) \cdot g_2 + \dots + b_{256}(m_i) \cdot g_{256}\right)
$$

其中：

- $b(m_i)$ 是消息 $m_i$ 经 blake2s 哈希后的哈希值，
- $b_j(m_i)$ 是哈希值的第 $j$ 个比特位。

观察可知，该签名方案具有**线性特性**：

- 任意多个签名可以线性组合，形成对一个新（未知）消息的签名。

于是，我们可将所有给定的签名表示为矩阵形式：

- 定义矩阵 $A$ 为哈希比特组成的矩阵，大小为 $256 \times 256$，每一行对应一个已知消息的哈希值。
- 设目标消息哈希为向量 $y$，则寻找向量 $x$，满足线性方程：

$$
x A = y
$$

求解该方程即可得到组合系数 $x$。通过这些系数对已有的签名做线性组合，即可得到目标消息的有效签名。

令已知的哈希矩阵为：

$$
A = \begin{pmatrix}
b_1(m_1) & b_2(m_1) & \dots & b_{256}(m_1)\\
b_1(m_2) & b_2(m_2) & \dots & b_{256}(m_2)\\
\vdots & \vdots & \ddots & \vdots \\
b_1(m_{256}) & b_2(m_{256}) & \dots & b_{256}(m_{256})
\end{pmatrix}
$$

给定目标哈希为行向量 $y$：

$$
y = (b_1(m), b_2(m), \dots, b_{256}(m))
$$

则问题转化为在有限域 $F_r$ 中求解线性方程：

$$
x \cdot A = y
$$

计算出 $x$ 后，目标签名为对应签名的线性组合：

$$
\Sigma = x_1 \Sigma_1 + x_2 \Sigma_2 + \dots + x_{256} \Sigma_{256}
$$

注意：由于是在有限域上，因此矩阵求解时元素均需在 $F_r$ 中进行运算。

## 4. Conclusion

通过上述观察，这个 puzzle 暗示了 Pedersen Hash 和 BLS 签名的线性特性。只要提供足够多的已知消息及签名，我们便可通过线性组合的方式快速计算任意未知消息的有效签名，体现了签名方案的潜在线性漏洞和需要注意的安全设计要点。

具体代码实现和演示详见：[ZK-Puzzle-Writups](https://github.com/timimm/zk-writeups/tree/main/docs/crypto/lets-hash-it-out/code)。