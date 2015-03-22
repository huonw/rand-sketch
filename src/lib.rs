#![feature(test, core)]
#![allow(non_snake_case)]

/*!

## TODO: update text for the stream type.

# Updated benchmarks

| bench| assoc | stream | typeparam | original |
|---|---|---|---|---|
| `gen_` | 708 | 706 | **688** | 717 |
| `iter` | **262** | **265** | 274 | 322 |
| `iter__noiterbb` | **258** | **255** | 270 | 263 |
| `range_gen` | 1292 | **1247** | **1244** |  |
| `range_gen__bb` | 3960 | 3937 | **3847** |  |
| `range_iter` | 765 | **724** | 1018 |  |
| `range_iter__bb` | 779 | **665** | 1088 |  |
| `range_iter__noiterbb` | **384** | 394 | 405 |  |

# Old text

`assoc` and `typeparam` are two different ways to encode constraints
for `Random` (aka `Random`). `assoc` uses an associated type and
generic conversions to provide a nice API, with computations
"front-loaded"/cached if possible (particularly for
`gen_iter`). `typeparam` has an extra type parameter on `Random`, and
doesn't benefit from caching (i.e. assumes the optimiser will handle
everything).

## Example

(of either `assoc` or `typeparam`, in the method form they
would use in a final API, which this crate doesn't implement)

```rust
// thread-local
let x: u32 = rand::random(..);

// typed variable
let x: u32 = rng.gen(..);
let y: f32 = rng.gen(a..b);

// inline type hint (extra type param compared to today)
let type_hint = rng.gen::<i64, _>(..);
```

## Benchmarks

```text
test gen_assoc                      ... bench:       676 ns/iter (+/- 29)
test gen_original                   ... bench:       684 ns/iter (+/- 14)
test gen_typeparam                  ... bench:       681 ns/iter (+/- 13)
test iter__noiterbb_assoc           ... bench:       270 ns/iter (+/- 260)
test iter__noiterbb_original        ... bench:       252 ns/iter (+/- 14)
test iter__noiterbb_typeparam       ... bench:       262 ns/iter (+/- 15)
test iter_assoc                     ... bench:       264 ns/iter (+/- 29)
test iter_original                  ... bench:       313 ns/iter (+/- 31)
test iter_typeparam                 ... bench:       272 ns/iter (+/- 12)
test range_gen__bb_assoc            ... bench:      3894 ns/iter (+/- 252)
test range_gen__bb_typeparam        ... bench:      3909 ns/iter (+/- 420)
test range_gen_assoc                ... bench:      1263 ns/iter (+/- 64)
test range_gen_typeparam            ... bench:      1223 ns/iter (+/- 77)
test range_iter__bb_assoc           ... bench:       749 ns/iter (+/- 37)
test range_iter__bb_typeparam       ... bench:      1062 ns/iter (+/- 34)
test range_iter__noiterbb_assoc     ... bench:       370 ns/iter (+/- 23)
test range_iter__noiterbb_typeparam ... bench:       405 ns/iter (+/- 158)
test range_iter_assoc               ... bench:       738 ns/iter (+/- 30)
test range_iter_typeparam           ... bench:       986 ns/iter (+/- 65)
```

All benchmarks used `u32`.

### Legend

- `original` calls the relevant function from `std::rand` (aka crates.io's `rand`)
- `gen` calls `black_box(gen())` 100 times
- `iter` runs `for x in black_box(gen_iter()) { black_box(x) }`
- `range` passes `4..321`, others just pass `..`.
- `range_*__bb` calls `black_box` on `4..321` before passing it
- `noiterbb` avoids the `black_box` around the `gen_iter` (i.e. let
  the inliner/optimiser see everything)

### Summary

The associated type version is essentially always more performant than
the type parameter version. Especially for ranged `gen_iter`, where
caching the internal computations required for `u32` makes a noticable
difference, even when the compiler has full information (the
`noiterbb` case), and hence could theoretically lift the comptations
in `typeparam` out of the loop.


*/


extern crate test;
extern crate rand;

pub mod stream;
pub mod assoc;
pub mod typeparam;

pub trait Into<Target> {
    fn into(self) -> Target;
}


mod original {
    #[cfg(test)]
    use test::{Bencher, black_box};
    #[cfg(test)]
    use rand::{self, Rng};

    #[bench]
    fn iter(b: &mut Bencher) {
        let rng: rand::XorShiftRng = rand::random();

        b.iter(|| {
            for x in black_box(rng.clone().gen_iter::<u32>().take(100)) {
                black_box(x);
            }
        })
    }

    #[bench]
    fn iter__noiterbb(b: &mut Bencher) {
        let rng: rand::XorShiftRng = rand::random();

        b.iter(|| {
            for x in (rng.clone().gen_iter::<u32>().take(100)) {
                black_box(x);
            }
        })
    }

    #[bench]
    fn gen_(b: &mut Bencher) {
        let mut rng: rand::XorShiftRng = rand::random();

        b.iter(|| {
            for _ in 4..321 {
                black_box(rng.gen::<u32>());
            }
        })
    }

}
