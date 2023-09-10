# distributed-halo2
Combines Multi-Party Computation (MPC) and Halo2 to facilitate rapid and private delegated proof generation within cloud settings.

## Introduction

ZK Proof Generation is a computationally expensive and time consuming process. It is not viable to generate zero-knowledge proofs on consumer devices like phones and laptops. Delegating proof generation to centralised cloud hardware providers is not always an option due to potential leakage of private witness data and linkage of witness data to proofs. 

zkHub enables delegated proof generation while preserving witness data. 
It solves this problem by efficiently combining Multi-Party Computation with Zero Knowledge Proofs. 
Every time a proof needs to be generated, the witness data required to generate the proof is split up into multiple chunks using Packed Secret Sharing - an efficient additive secret sharing scheme. Once split, each chunk is shared with different cloud hardware nodes, which communicate without revealing their parts of the shares, generate the proof and share it with the client.

With this technique, proof generation speed ups of 22x are observed over local generation. 

For the hackathon, we combine MPC with the halo2 - seamlessly enabling speed ups in all applications built over the framework (ezkl, Semaphore, etc).
It involved building distributed versions of FFT and MSM operations in the commitment and generation  phases of proof generation, and integrating them with halo2 libraries. 

Techniques from research papers like DIZK (OB22), EoS and zkSaas were used in the implementation

## Repository Breakdown - 

### dist-primitives 
Implements distributed versions of the primitive operations - Multi Scalar Multiplication, Fast Fourier Transforms, Partial Products. 

**src/dfft**

This implementation aims to efficiently and securely perform FFT computations among multiple parties, leveraging the linearity property of the FFT function while minimizing communication rounds. The protocol involves an initial setup, data masking, computation, and secure sharing of results.

Initial Setup:
Use Packed Secret Sharing to pack shares into vectors of size l.
The FFT algorithm is a key component, and the main idea is to perform computations on packed data as long as possible before reaching a level where local computation becomes impossible. 
For m inputs, we start with packed vectors at a specific level, denoted as i = log m.
These packed vectors are arranged so that the values within each vector have the "j" indices as far apart as possible, pushing the problem down to deeper recursive layers. Even with the best initial ordering, there comes a level (i = log ℓ + 1) where local computation becomes impossible due to data structure constraints.
For m inputs, we start with packed vectors at a specific level, denoted as i = log m. 

We use a server and only two rounds of communication to minimize communication between parties and introduce a more efficient solution. 
The linearity property of FFTs is leveraged here. The steps for implementing FFT are as follows - 

1) Random values (r1, ..., rm) are shared among the parties.
2) Shares of (s1, ..., sm) are generated using FFT operations on the random values.
3) The parties mask their data with (r1, ..., rm) and send it to a powerful party (P1)
4) P1 performs computations on the masked data, reducing it to level i = 1.
5) The result is sent back to the parties, who can then securely obtain the FFT output by subtracting the masked values.

**src/dpp**

This folder contains the functionality to efficiently perform Partial Product Operations among multiple parties. 
The goal is to securely compute a product of elements x1 to xm, but not sequentially (which would take O(m) rounds), rather in a more parallelized and efficient way. The approach relies on rewriting the product in a way that allows for parallel computation.
Instead of computing the product sequentially, we break the problem into smaller products over disjoint subsets of the input xi's.

Initial Setup: 
Similar to dfft, all parties start with Packed Secret Shares of vectors x1 to xm/l, where each xi is an l-sized vector consisting of the corresponding input values.
The protocol simultaneously computes products Fpart(x[1, l], ..., x[1, m/l]), Fpart(x[1, m/l + 1], ..., x[1, 2m/l]), ..., Fpart(x[1, (l-1)m/l + 1], ..., x[1, m]) in parallel.
These products are computed using techniques that involve O(m) total computation and communication.


**src/dmsm**

We compute secret shares of the scalars involved in the operation and then compute the shares of the resulting group elements. This enables the use of arithmetic field operations in the exponent, which are necessary for group exponentiation and element multiplication. Since we are dealing with group operations in elliptic curve groups, which can't be directly represented as arithmetic operations over finite fields (unlike traditional polynomial-based secret sharing schemes) we extend the concept of polynomial-based secret sharing to handle group operations. 

**Addition in the Exponent**
We allow parties to locally multiply their shares of two vectors in the exponent, resulting in a valid packed secret sharing of the product of those vectors.

**Multiplication in the Exponent**
Parties can locally compute the product of shares of two vectors of field elements in the exponent. However, this operation increases the degree of the resulting sharing, so degree reduction techniques are applied to mitigate this.

**Parallel Computation**
The protocol breaks down the multi-scalar multiplication operation into smaller parts, essentially computing multiple instances of MSM in parallel. This is achieved by splitting the input vectors into smaller segments and processing them independently.

Implementation Steps - 

1) Parties obtain random values and use them to generate shares.
2) They compute local multiplications and transformations on the input shares to obtain intermediate results.
3) These intermediate results are then combined to compute the final result securely.

### secret-sharing

This is an implementation of the Packed Secret Sharing Algorithm over the halo2 Evaluation Domain for computing a batch of packed secret shares of random vectors without reconstructing the actual vectors. 

It requires a Vandermonde matrix [Vn,n−t] over the finite field F as auxiliary input. This matrix is used for computations during the protocol.
The parties involved in the protocol do not have any specific inputs; they are generating random values.

The protocol ensures that even if some parties are corrupted (up to t parties), the security of the protocol is maintained. The total communication and computation complexity of this protocol is O(n^2), but since it generates packed shares of O(n−t) vectors, each of length O(n), the amortized cost of sharing a random value using this protocol is O(1).

### halo2 

PSE's version of the Halo2 framework with KZG Commitment Scheme. 
We replace the function calls to "best_fft" and "best_multiexp" with dfft and dmsms for this version. We plan on working 

### mpc-net 

Inspired by [[https://github.com/alex-ozdemir/collaborative-zksnark]]

This implements all the networking logic between the king party and the worker parties, 

### kzg_test

