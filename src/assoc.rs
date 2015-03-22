use Into;
use rand::Rng;
use std::ops::{RangeFull, Range, RangeFrom, RangeTo};

/// Data types that can be created randomly.
pub trait Random {
    /// A type that mediates/constraints what values are generated
    type Constraint = RangeFull;

    /// Create a random value using the given constraints and random number generator
    fn gen<R: Rng>(constraint: &Self::Constraint, rng: &mut R) -> Self;
}


/// Create a single random value, mediated by `constraint`.
pub fn gen<Rand: Random, Constraint: Into<Rand::Constraint>, R: Rng>(rng: &mut R, constraint: Constraint) -> Rand {
    Random::gen(&constraint.into(), rng)
}

/// Create an infinite sequence of random values, mediated by `constraint`.
pub fn gen_iter<Rand: Random, Constraint: Into<Rand::Constraint>, R: Rng>(rng: R, constraint: Constraint) -> GenIter<Rand, R> {
    let c = constraint.into();
    GenIter {
        constraint: c,
        rng: rng
    }
}

pub struct GenIter<Rand: Random, R: Rng> {
    constraint: Rand::Constraint,
    rng: R
}

impl<Rand: Random, R: Rng> Iterator for GenIter<Rand, R> {
    type Item = Rand;

    fn next(&mut self) -> Option<Rand> {
        Some(Random::gen(&self.constraint, &mut self.rng))
    }
}

/// Constraints for generating integers. This can be used with
/// `gen` and `gen_iter` via the various `Range*` types,
/// e.g. `gen(rng, ..)`, `gen_iter(rng, 0..10)`.
pub struct IntegerConstraint<X> {
    inner: IntegerConstraint_<X>
}

enum IntegerConstraint_<X> {
    Full,
    Bounded { low: X, range: X, accept_zone: X }
}

impl Random for u32 {
    type Constraint = IntegerConstraint<u32>;

    fn gen<R: Rng>(constraint: &IntegerConstraint<u32>, rng: &mut R) -> u32 {
        match constraint.inner {
            IntegerConstraint_::Full => {::test::black_box(123456); rng.gen::<u32>()},
            IntegerConstraint_::Bounded {low, range, accept_zone} => {
                ::test::black_box(7890123);
                loop {
                    let v = rng.gen::<u32>();

                    if v < accept_zone {
                        return low.wrapping_add((v % range))
                    }
                }
            }
        }
    }
}
impl Into<IntegerConstraint<u32>> for RangeFull {
    fn into(self) -> IntegerConstraint<u32> {
        IntegerConstraint { inner: IntegerConstraint_::Full }
    }
}
impl Into<IntegerConstraint<u32>> for Range<u32> {
    fn into(self) -> IntegerConstraint<u32> {
        assert!(self.start < self.end);
        let range = self.end - self.start;
        let max = !0;
        let zone = max - (max % range);
        IntegerConstraint {
            inner: IntegerConstraint_::Bounded {
                low: self.start,
                range: range,
                accept_zone: zone,
            }
        }
    }
}
impl Into<IntegerConstraint<u32>> for RangeFrom<u32> {
    #[allow(unsigned_negation)]
    fn into(self) -> IntegerConstraint<u32> {
        if self.start == 0 {
            IntegerConstraint { inner: IntegerConstraint_::Full }
        } else {
            let range = -self.start;
            let max = !0;
            let zone = max - (max % range);
            IntegerConstraint {
                inner: IntegerConstraint_::Bounded {
                    low: self.start,
                    range: range,
                    accept_zone: zone,
                }
            }
        }
    }
}
impl Into<IntegerConstraint<u32>> for RangeTo<u32> {
    fn into(self) -> IntegerConstraint<u32> {
        (0..self.end).into()
    }
}

impl Random for i64 {
    type Constraint = IntegerConstraint<u64>;

    fn gen<R: Rng>(constraint: &IntegerConstraint<u64>, rng: &mut R) -> i64 {
        match constraint.inner {
            IntegerConstraint_::Full => rng.gen::<i64>(),
            IntegerConstraint_::Bounded {low, range, accept_zone} => {
                loop {
                    let v = rng.gen::<u64>();

                    if v < accept_zone {
                        return low.wrapping_add(v % range) as i64
                    }
                }
            }
        }
    }
}
impl Into<IntegerConstraint<u64>> for RangeFull {
    fn into(self) -> IntegerConstraint<u64> {
        IntegerConstraint { inner: IntegerConstraint_::Full }
    }
}
impl Into<IntegerConstraint<u64>> for Range<i64> {
    fn into(self) -> IntegerConstraint<u64> {
        assert!(self.start < self.end);
        let range = self.end.wrapping_sub(self.start) as u64;
        let max = !0;
        let zone = max - (max % range);
        IntegerConstraint {
            inner: IntegerConstraint_::Bounded {
                low: self.start as u64,
                range: range,
                accept_zone: zone,
            }
        }
    }
}
impl Into<IntegerConstraint<u64>> for RangeFrom<i64> {
    fn into(self) -> IntegerConstraint<u64> {
        if self.start == ::std::i64::MIN {
            IntegerConstraint { inner: IntegerConstraint_::Full }
        } else {
            let range = -::std::i64::MIN.wrapping_add(self.start) as u64;
            let max = !0;
            let zone = max - (max % range);
            IntegerConstraint {
                inner: IntegerConstraint_::Bounded {
                    low: self.start as u64,
                    range: range,
                    accept_zone: zone,
                }
            }
        }
    }
}

/// Constraints for generating floats. This can be used with
/// `gen` and `gen_iter` via the various `Range*` types,
/// e.g. `gen(rng, ..)`, `gen_iter(rng, 0.0 .. 10.0)`.
pub struct FloatConstraint<X> {
    inner: Option<Range<X>>
}
impl Random for f64 {
    type Constraint = FloatConstraint<f64>;

    fn gen<R: Rng>(cons: &FloatConstraint<f64>, rng: &mut R) -> f64 {
        match cons.inner {
            None => rng.gen(),
            Some(ref range) => {
                range.start + rng.gen() * (range.end - range.start)
            }
        }
    }
}

impl Into<FloatConstraint<f64>> for RangeFull {
    fn into(self) -> FloatConstraint<f64> {
        FloatConstraint { inner: None }
    }
}
impl Into<FloatConstraint<f64>> for Range<f64> {
    fn into(self) -> FloatConstraint<f64> {
        FloatConstraint { inner: Some(self) }
    }
}

#[cfg(test)]
use test::{Bencher, black_box};
#[cfg(test)]
use rand;

#[bench]
fn iter(b: &mut Bencher) {
    let rng: rand::XorShiftRng = rand::random();

    b.iter(|| {
        for x in black_box(gen_iter::<u32, _, _>(rng.clone(), ..).take(100)) {
            black_box(x);
        }
    })
}

#[bench]
fn iter__noiterbb(b: &mut Bencher) {
    let rng: rand::XorShiftRng = rand::random();

    b.iter(|| {
        for x in (gen_iter::<u32, _, _>(rng.clone(), ..).take(100)) {
            black_box(x);
        }
    })
}

#[bench]
fn range_iter__bb(b: &mut Bencher) {
    let rng: rand::XorShiftRng = rand::random();

    b.iter(|| {
        for x in black_box(gen_iter::<u32, _, _>(rng.clone(), black_box(4..321)).take(100)) {
            black_box(x);
        }
    })
}

#[bench]
fn range_iter(b: &mut Bencher) {
    let rng: rand::XorShiftRng = rand::random();

    b.iter(|| {
        for x in black_box(gen_iter::<u32, _, _>(rng.clone(), 4..321).take(100)) {
            black_box(x);
        }
    })
}

#[bench]
fn range_iter__noiterbb(b: &mut Bencher) {
    let rng: rand::XorShiftRng = rand::random();

    b.iter(|| {
        for x in (gen_iter::<u32, _, _>(rng.clone(), 4..321).take(100)) {
            black_box(x);
        }
    })
}

#[bench]
fn gen_(b: &mut Bencher) {
    let mut rng: rand::XorShiftRng = rand::random();

    b.iter(|| {
        for _ in 4..321 {
            black_box(gen::<u32, _, _>(&mut rng, ..));
        }
    })
}

#[bench]
fn range_gen__bb(b: &mut Bencher) {
    let mut rng: rand::XorShiftRng = rand::random();

    b.iter(|| {
        for _ in 4..321 {
            black_box(gen::<u32, _, _>(&mut rng, black_box(4..321)));
        }
    })
}

#[bench]
fn range_gen(b: &mut Bencher) {
    let mut rng: rand::XorShiftRng = rand::random();

    b.iter(|| {
        for _ in 4..321 {
            black_box(gen::<u32, _, _>(&mut rng, (4..321)));
        }
    })
}
