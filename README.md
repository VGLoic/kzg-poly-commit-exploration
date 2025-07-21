# KZG polynomial commitments exploration

This repository is an exploration to KZG (Kate, Zaverucha and Goldberg) polynomial commitments. The goal is to explore the topic and document every progress in this README. The target would be to have a CLI that allows to handle the different steps in order to perform a polynomial commitment.

The codebase will be written in Rust. As I'm still learning things about Rust, this README will also contain some documentation about my learning in Rust.

## Starting point

- I have a good understanding of modular arithmetics but I am not a mathematician. I understand elliptic curve basic theory but I never got my hands dirty on it yet (theoretically or in software),
- I have followed the lectures of the [Applied ZK learning group of 0xParc](https://learn.0xparc.org/materials/circom/learning-group-1/intro-zkp/),
- I have read the [article of Dankrad Feist about KZG polynomial commitments](https://dankradfeist.de/ethereum/2020/06/16/kate-polynomial-commitments.html).

## Exploration steps

I like my [latest read from Dankrad Feist](https://dankradfeist.de/ethereum/2020/06/16/kate-polynomial-commitments.html) so I will follow up a bit the steps in the article.

### Dive into the groups

A good first achievement would be to have a command in order to run a trusted setup and generate the associated artifacts.

For my trusted setup I first need to have a prime field, two elliptic curves and a third group where I have a bilinear map as a valid pairing.

I don't really know what to take, I will do a few additional reads:
- [Exploring Elliptic Curve Pairings by Vitalik Buterin](https://vitalik.eth.limo/general/2017/01/14/exploring_ecp.html),
- [A (relatively easy to understand) primer on elliptic curve cryptography)](https://blog.cloudflare.com/a-relatively-easy-to-understand-primer-on-elliptic-curve-cryptography/), found from the above post,
- [BLS12-381 for the rest of us (revised version)](https://eth2book.info/latest/part2/building_blocks/bls12-381/) from Ben Edgington, I found this after a Google search about which elliptic curves I could use.

The two first resources were good reads, I am still in the middle of reading the third one which explains a huge amount of things. There are actually a good number of potential candidates for the curves.
However, the family of BLS curves, and in particular the **BLS12-381** is a good construction for the whole package:
- the first elliptic curve group is defined by the equation `y^2 = x^3 + 4` over the field `F_q` with `q = 0x1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab` (in hexadecimal). The size of `q` is `381 bits`, hence the `381` in the name of the curve,
- the second elliptic curve group is actually derived once we said we wanted a pairing, i.e. we needed another group with the same order than the first one. Actually, by `extending` the `F_q` field `12` times, i.e. `F_q^12` and plugging it with the same elliptic curve equation, we can find the subgroup with the target order. The `12`, called the `embedding degree` of the curve, is the `12` in the name of the curve. By luck, a `twist` can be built, introducing a mapping from `F_q_^2` with `y^2 = x^3 + 4(1 + i)` to our elliptic curve over `F_q^12`, it should allow to work with way easier coordinates. Therefore, the second group can either be the first elliptic curve over `F_q^12` or the second one over `F_q^2`, in the literature I found the `F_q_12` version but in implementation, it is likely I meet the `F_q_2` version,
- the third group, target of the pairing will actually be a subgroup of `F_q^12`. I have not yet seen how to construct the pairing so more on this later,
- I have not found the exact definition of the pairing, it can be found in [this course](https://static1.squarespace.com/static/5fdbb09f31d71c1227082339/t/5ff394720493bd28278889c6/1609798774687/PairingsForBeginners.pdf), we'll treat it as an unknown for now and we'll see if we need it when implementing things.

This whole part is way heavier than what I described here, I tried my best to summarize it to the absolute minimum, refer to the Ben Edington's article to dig on this, it is very good.

In terms of implementations, the book of Ben Edington refers to multiple resources, for now I am still exploring a bit the ones that may be compatible for Rust:
- [blst](https://crates.io/crates/blst): provides a simple API for BLS signature, not sure it could handle my simple trusted setup plan,
- [constantine](https://github.com/mratsim/constantine): not sure yet how to use this one,
- [blsh](https://github.com/one-hundred-proof/blsh): seems promising to play around,
- [algebra](https://github.com/arkworks-rs/algebra): I discovered it a bit later, I have not tried it yet.

#### Library choice

I finally dig in `blst`, I was also looking at `blsh` codebase to see how it was using the `blst` library.

The library is mostly made for dealing with BLS signatures but still exposes low levels utilities in order to work with the elliptic curve groups and the finite field groups. 

It took me some time to have something working as the library does not have a very good documentation and my understanding of the different formats was not on point. I had to read a good amount of times the serialization conventions from [ZCash](https://github.com/zcash/librustzcash/blob/6e0364cd42a2b3d2b958a54771ef51a8db79dd29/pairing/src/bls12_381/README.md#serialization) and [Ben Edington's book](https://eth2book.info/capella/part2/building_blocks/bls12-381/#point-compression) in order to understand the compression and serialization utilities from the library. The [explanation](https://eth2book.info/capella/part2/building_blocks/bls12-381/#coordinate-systems) for the various coordinate systems are also a must read.

I got what I needed in two tests:
- one for operations where I test addition of two points and addition of one point `n` times,
- one for testing compression and serialization.

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_point_addition_and_scalar_multiplication() {
        unsafe {
            let g1 = blst::blst_p1_generator();

            let mut p1_via_addition = blst::blst_p1::default();
            blst::blst_p1_add_or_double(&mut p1_via_addition, g1, g1);

            let mut p1_via_multiplication = blst::blst_p1::default();
            let scalar_as_bytes = 2_u8.to_be_bytes();
            blst::blst_p1_mult(
                &mut p1_via_multiplication,
                g1,
                scalar_as_bytes.as_ptr(),
                scalar_as_bytes.len() * 8,
            );

            assert!(blst::blst_p1_in_g1(g1), "g1 must be in the first group");
            assert_eq!(
                p1_via_multiplication, p1_via_addition,
                "results must be the same via multiplication and via addition"
            );
            assert_ne!(
                p1_via_multiplication, *g1,
                "result must be different than g1"
            );
            assert!(
                blst::blst_p1_in_g1(&p1_via_multiplication),
                "result must be in first group"
            );
        }
    }

    #[test]
    fn test_compression_and_serialization() {
        unsafe {
            let g1 = blst::blst_p1_generator();

            let mut p1 = blst::blst_p1::default();
            blst::blst_p1_add_or_double(&mut p1, g1, g1);

            let mut compressed_p1 = [0; 48];
            blst::blst_p1_compress(compressed_p1.as_mut_ptr(), &p1);
            let mut uncompressed_p1_affine = blst::blst_p1_affine::default();
            match blst::blst_p1_uncompress(&mut uncompressed_p1_affine, compressed_p1.as_ptr()) {
                blst::BLST_ERROR::BLST_SUCCESS => {}
                other => {
                    println!("Got error while uncompressing: {other:?}");
                    panic!("Fail to uncompress")
                }
            };
            let mut uncompressed_p1 = blst::blst_p1::default();
            blst::blst_p1_from_affine(&mut uncompressed_p1, &uncompressed_p1_affine);
            assert_eq!(
                uncompressed_p1, p1,
                "result after uncompression must be equal to p1"
            );

            let mut serialized_p1 = [0; 96];
            blst::blst_p1_serialize(serialized_p1.as_mut_ptr(), &p1);
            let mut deserialized_p1_affine = blst::blst_p1_affine::default();
            match blst::blst_p1_deserialize(&mut deserialized_p1_affine, serialized_p1.as_ptr()) {
                blst::BLST_ERROR::BLST_SUCCESS => {}
                other => {
                    println!("Got error while deserializing: {other:?}",);
                    panic!("Fail to deserialize")
                }
            };

            let mut deserialized_p1 = blst::blst_p1::default();
            blst::blst_p1_from_affine(&mut deserialized_p1, &deserialized_p1_affine);
            assert_eq!(
                deserialized_p1, p1,
                "result after deserialization must be equal to p1"
            );
        }
    }
}
```

#### Digging until I got tests for polynomial commitments

I wanted to write my trusted setup script but I liked my first approach with test. So I took the time to write two more tests for polynomial commitments without smart things and with simple polynomials. I did one test for order 1 polynomial, the other is for order 2 polynomial. The approach is the same in both tests:
1. start by generating a secret,
2. the secret is used in order to compute the multiple of the generators of the first and second group,
3. use these quantities in order to compute the polynomial commitment,
4. choose a point at which we want to open the polynomial and derive the quotient polynomial manually (non automated),
5. evaluate the quotient polynomial at the secret using the artifacts of step 2,
6. compute the remaining quantities and the two pairings,
7. compare the pairings, they must be equal.

Not gonna lie, it took me some time to make it work. I discovered a lot about bytes in general, in particular the difference between [little endian and big endian](https://www.techtarget.com/searchnetworking/definition/big-endian-and-little-endian). There are still some things that are not perfectly clear for me, like the `scalar` type of the `blst` crate, I will try to understand it a bit better. However, it allowed me to illustrate concretely the KZG polynomial commitment process and I'm quite happy to have this working.

Here is the test made for the order one polynomial:
```rust
#[test]
fn test_commitment_for_polynomial_degree_one() {
    let mut s_bytes = [0; 48]; // Field elements are encoded in big endian form with 48 bytes
    rand::rng().fill_bytes(&mut s_bytes);
    let mut s_as_scalar = blst::blst_scalar::default();
    unsafe {
        blst::blst_scalar_from_be_bytes(&mut s_as_scalar, s_bytes.as_ptr(), s_bytes.len());
    };

    let mut s_g1 = blst::blst_p1::default();
    unsafe {
        blst::blst_p1_mult(
            &mut s_g1,
            blst::blst_p1_generator(),
            s_as_scalar.b.as_ptr(),
            s_as_scalar.b.len() * 8,
        );
    };
    let mut s_g2 = blst::blst_p2::default();
    unsafe {
        blst::blst_p2_mult(
            &mut s_g2,
            blst::blst_p2_generator(),
            s_as_scalar.b.as_ptr(),
            s_as_scalar.b.len() * 8,
        );
    };

    // Polynomial to commit is `p(x) = 5x + 10
    // a1 = 5, a0 = 10`
    let a0 = blst_scalar_from_u8(10);
    let mut constant_part = blst::blst_p1::default();
    unsafe {
        blst::blst_p1_mult(
            &mut constant_part,
            blst::blst_p1_generator(),
            a0.b.as_ptr(),
            a0.b.len() * 8,
        );
    };

    let a1 = blst_scalar_from_u8(5);
    let mut order_one_part = blst::blst_p1::default();
    unsafe {
        blst::blst_p1_mult(&mut order_one_part, &s_g1, a1.b.as_ptr(), a1.b.len() * 8);
    };
    let mut commitment = blst::blst_p1::default();
    unsafe {
        blst::blst_p1_add_or_double(&mut commitment, &constant_part, &order_one_part);
    };

    // We evaluate the polynomial at z = 1: `p(z) = y = p(1) = 15`
    // Quotient polynomial: `q(x) = (p(x) - y) / (x - z) = (5x - 5) / (x - 1) = 5`
    let q_as_scalar = blst_scalar_from_u8(5);
    let mut q_at_s = blst::blst_p1::default();
    unsafe {
        blst::blst_p1_mult(
            &mut q_at_s,
            blst::blst_p1_generator(),
            q_as_scalar.b.as_ptr(),
            q_as_scalar.b.len() * 8,
        );
    };

    let z = unsafe { *blst::blst_p2_generator() };
    let divider = blst_p2_sub(&s_g2, &z);
    let lhs = bilinear_map(&q_at_s, &divider);

    let y_as_scalar = blst_scalar_from_u8(15);
    let mut y = blst::blst_p1::default();
    unsafe {
        blst::blst_p1_mult(
            &mut y,
            blst::blst_p1_generator(),
            y_as_scalar.b.as_ptr(),
            y_as_scalar.b.len() * 8,
        );
    };
    let commitment_part = blst_p1_sub(&commitment, &y);
    let g2 = unsafe { *blst::blst_p2_generator() };
    let rhs = bilinear_map(&commitment_part, &g2);

    assert_eq!(lhs, rhs);
}
```

## Repository setup

Environment variables can be set up using `.env` file at the root of the repository, see `.env.example` for a list of the supported environment variables.

A single executable as a CLI is present, use `cargo run -- --help` to show the available commands.

## Resources

- [Applied ZK learning group of 0xParc](https://learn.0xparc.org/materials/circom/learning-group-1/intro-zkp/): a set of videos going through Circom and a bit of applied cryptography,
- [Article of Dankrad Feist about KZG polynomial commitments](https://dankradfeist.de/ethereum/2020/06/16/kate-polynomial-commitments.html): article for high level explanation of the KZG polynomial commitment,
- [Exploring Elliptic Curve Pairings by Vitalik Buterin](https://vitalik.eth.limo/general/2017/01/14/exploring_ecp.html): nice overview of elliptic curve and pairings, dive a little bit in the math,
- [A (relatively easy to understand) primer on elliptic curve cryptography)](https://blog.cloudflare.com/a-relatively-easy-to-understand-primer-on-elliptic-curve-cryptography/): nice introduction to elliptic curve,
- [BLS12-381 for the rest of us (revised version)](https://eth2book.info/latest/part2/building_blocks/bls12-381/): in depth documentation about BLS12-381, its definition but also how to use it,
- [Pairings for beginners](https://static1.squarespace.com/static/5fdbb09f31d71c1227082339/t/5ff394720493bd28278889c6/1609798774687/PairingsForBeginners.pdf): serious mathematical way to elliptic curves and pairings.

