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
