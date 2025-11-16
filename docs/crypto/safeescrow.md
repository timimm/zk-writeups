---
title: Crypto - SafeEscrow
description: 2024 | DreamHack | Crypto | Blockchain
---

[TOC]

## 1. Puzzle Description

Do you have knowledge about zero-knowledge proofs? You should have some.

> 题目连接：https://dreamhack.io/wargame/challenges/1297

## 2. Analysis
<details>
<summary><font color=MediumAquamarine>查看合约核心源代码 👀</font></summary>

```solidity
contract SafeEscrow {
    bool public solved;

    Verifier v;

    constructor() {
        v = new Verifier();
    }

    function withdraw(uint256[8] calldata proof) external {
        uint256 Nullifier = 3631369181433719484956790922555555011136438559751492114283630303736666045113;
        uint256 WalletIndex = 6453692159775602397386942979474506661254012100833066612593672063063229257634;
        // wallet_address = 0x2dEc1802F473ffA1Fd162888C7a2bb08624867d5

        uint256[2] memory publicInputs = [Nullifier, WalletIndex];
        v.verifyProof(proof, publicInputs);
        checkEmptyWallet(calculateAddress(WalletIndex));
        _withdraw(proof, publicInputs);
    }

    function calculateAddress(uint256 walletIndex) internal pure returns (address targetWallet) {
        targetWallet = address(bytes20(keccak256(abi.encodePacked(walletIndex))));
    }

    function checkEmptyWallet(address tw) internal view {
        uint256 size;
        assembly {
            size := extcodesize(tw)
        }

        require(!(size == 0 && address(tw).balance == 0));
    }

    function _withdraw(uint256[8] memory proof, uint256[2] memory pi) internal {
        solved = true;
    }
}

```

</details>

首先查看合约逻辑，硬编码了 `Nullifier, WalletIndex`两个参数值，应该是公共输入。其中 `WalletIndex` 是地址的 10 进制表示，合约会对这个地址调用 `checkEmptyWallet` 函数，确保其 balance 不为 0。

合约逻辑非常简明，我们只需要满足：

- `WalletIndex` 地址的 balance 不为 0；
- 向合约输入一个合法 proof 。

因此，我们去查看circuit的逻辑结构。

<details>
<summary><font color=MediumAquamarine>查看电路源代码 👀</font></summary>

```go
type Circuit struct {
	Secret       frontend.Variable
	Nullifier    frontend.Variable `gnark:",public"`
	WalletIndex  frontend.Variable `gnark:",public"`
	WalletSanity frontend.Variable
}

func (circuit *Circuit) Define(api frontend.API) error {
	api.AssertIsEqual(api.Mul(api.Mul(api.Mul(api.Mul(circuit.Secret, circuit.Secret), circuit.Secret), circuit.Secret), circuit.Secret), circuit.Nullifier)
	api.AssertIsEqual(api.Mul(circuit.Secret, circuit.WalletIndex), circuit.Nullifier)    // wallet index
	api.AssertIsEqual(api.Add(circuit.WalletIndex, circuit.WalletSanity), circuit.Secret) // wallet index / sanity check
	return nil
}
```
</details>

电路逻辑可以总结为以下等式：

$$
\begin{aligned}
&secret ^ 5 = nullifier \\
&secret * wallet\_index = nullifier \\
&wallet\_index = wallet\_sanity = secret
\end{aligned}
$$

一共四个未知数，但从合约中我们已经知道其中两个未知数 `wallet_index, nullifier`。三个等式解两个未知数：

```python
from Crypto.Util.number import inverse
nullifier = 3631369181433719484956790922555555011136438559751492114283630303736666045113
wi = 6453692159775602397386942979474506661254012100833066612593672063063229257634
fr = 21888242871839275222246405745257275088548364400416034343698204186575808495617
inv = inverse(wi, fr)

secret = inv * nullifier % fr
print(secret)

assert(pow(secret, 5, fr) == nullifier)

ws = (secret - wi) % fr
print(ws)
```

得到隐私输入后，我们可以利用 gnark 生成 proof ，这也许是整个题目中最难写的part：

```go
func main() {
	fpSize := 4 * 8

	var circuit Circuit
	vkBytes, err := os.ReadFile("./verifying.key")
	if err != nil {
		panic(err)
	}

	vk := groth16.NewVerifyingKey(ecc.BN254)
	vk.ReadFrom(bytes.NewReader(vkBytes))

	file, err := os.Create("verifier.sol")
	if err != nil {
		panic(err)
	}
	defer file.Close()
	err = vk.ExportSolidity(file)
	if err != nil {
		panic(err)
	}

	// 1. get R1CS
	r1cs, err := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit)
	if err != nil {
		panic(err)
	}

	// print R1CS
	internal, secret, public := r1cs.GetNbVariables()
	fmt.Printf("public, secret, internal %v, %v, %v\n", public, secret, internal)

	// 2. get pk
	pkFile, err := os.ReadFile("proving.key")
	if err != nil {
		panic(err)
	}
	pk := groth16.NewProvingKey(ecc.BN254)
	pk.ReadFrom(bytes.NewReader(pkFile))

	// 3. build witness
	assignment := Circuit{
		Secret:       "10577078978341052228994623870320872030440906461492423559009608430703",         // 设置 Secret 变量
		Nullifier:    "3631369181433719484956790922555555011136438559751492114283630303736666045113", // 设置 Nullifier 变量
		WalletIndex:  "6453692159775602397386942979474506661254012100833066612593672063063229257634", // 设置 WalletIndex 变量
		WalletSanity: "15434550722640751803200514994777392297615224330023874192596955682522187668686"}
	witness, _ := frontend.NewWitness(&assignment, ecc.BN254.ScalarField())
	publicWitness, _ := witness.Public()
	fmt.Println("Public: ", publicWitness)

	// 持久化
	pubWitFile, err := os.Create("publicWitness.json")
	// assert.NoError(err)
	defer pubWitFile.Close()

	data := fmt.Sprint(publicWitness.Vector())
	fmt.Printf("public wit: %v\n", data)
	_, err = pubWitFile.WriteString(data)
	// assert.NoError(err)

	// prove
	proof, err := groth16.Prove(r1cs, pk, witness)
	bn254proof := proof.(*groth16_bn254.Proof)
	fmt.Printf("%+v\n", bn254proof.MarshalSolidity())
	if err != nil {
		panic(err)
	}
	// proofPath := fmt.Sprintf("proof")
	// proofFile, _ := os.OpenFile(proofPath, os.O_CREATE|os.O_WRONLY, 0666)
	// defer proofFile.Close()
	// proof.WriteTo(proofFile)

	// proof[i] = new(big.Int).SetBytes(proofBytes[fpSize*i : fpSize*(i+1)])

	// 5. verify-1
	err2 := groth16.Verify(proof, vk, publicWitness)
	if err2 != nil {
		panic(err2)
	}
	fmt.Println("Gnark Verified!")

	// 6. verify-2
	// proofBytes, err := hex.DecodeString(string(bn254proof.MarshalSolidity()))
	proofBytes := bn254proof.MarshalSolidity()
	if len(proofBytes) != fpSize*8 {
		panic("proofBytes != fpSize*8")
	}
	// checkErr(err, "decode proof hex failed")

	var final_proof [8]*big.Int

	// proof.Ar, proof.Bs, proof.Krs
	for i := 0; i < 8; i++ {
		final_proof[i] = new(big.Int).SetBytes(proofBytes[fpSize*i : fpSize*(i+1)])
	}
	fmt.Printf("final_proof: %v\n", printGroth16Proof(final_proof))

}

func printGroth16Proof(proof [8]*big.Int) string {
	strs := make([]string, len(proof))

	// 将数组中的每个big.Int元素转换为字符串
	for i, num := range proof {
		strs[i] = num.String()
	}

	result := strings.Join(strs, ",")

	return "[" + result + "]"
}
```

代码输出了可以直接传递给合约的数组形式。

```
[8099153543188745508175831438214699454316597283777416373805285378337894635219, 4752283743648249369871703999591739156697179853430237160185953056683691919040, 3672793038585677738256671951490594097269506939888886853074694423123390129322, 14595975645544950832872828444328013359931446076397165248168461021684756054536, 18094993552006574819160086020644554537527958371006997742353144028933191486115, 1931243151356508693977569118323975116222139982223999685371150925902773089898, 19765490139519166876225668675861823609972368815565063303605763659883139179045, 11442623890359346105849251388097905454073958723760352904628113343002100881590]
```

最后使用 web3py 工具写合约交互脚本即可。（别忘了给那个地址先充值一点eth，这是题目的第一个要求）。

我自己在 S 网部署了一个合约测试，最后可以直接使用 cast 工具查看是否 solved：

```bash
cast call 0x0bde4e8bdb83bd29f073c72709daf1e07036e394 "solved()" --rpc-url https://eth-sepolia.g.alchemy.com/v2/your_api_key
```

返回结果： `0x0000000000000000000000000000000000000000000000000000000000000001`.

Done!

