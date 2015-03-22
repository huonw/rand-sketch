use rand::Rng;
use std::marker;
use std::ops::{RangeFull, Range, RangeFrom, RangeTo};

/// Data types that can be created randomly, with `Constraint`
/// restricting what values can be created.
pub trait Random<Constraint = RangeFull> {
    fn gen<R: Rng>(constraint: &Constraint, rng: &mut R) -> Self;
}

/// Create a single random value, mediated by `constraint`.
pub fn gen<Rand: Random<Constraint>, Constraint, R: Rng>(rng: &mut R, constraint: Constraint) -> Rand {
    Random::gen(&constraint, rng)
}

/// Create an infinite sequence of random values, mediated by `constraint`.
pub fn gen_iter<Rand: Random<Constraint>, Constraint, R: Rng>(rng: R, constraint: Constraint) -> GenIter<Rand, Constraint, R> {
    GenIter {
        constraint: constraint,
        rng: rng,
        _marker: marker::PhantomData,
    }
}

pub struct GenIter<Rand: Random<Constraint>, Constraint, R: Rng> {
    constraint: Constraint,
    rng: R,
    _marker: marker::PhantomData<fn() -> Rand>,
}

impl<Constraint, Rand: Random<Constraint>, R: Rng> Iterator for GenIter<Rand, Constraint, R> {
    type Item = Rand;

    fn next(&mut self) -> Option<Rand> {
        Some(Random::gen(&self.constraint, &mut self.rng))
    }
}


impl Random<RangeFull> for u32 {
    fn gen<R: Rng>(_: &RangeFull, rng: &mut R) -> u32 {
        rng.gen()
    }
}
impl Random<Range<u32>> for u32 {
    fn gen<R: Rng>(range: &Range<u32>, rng: &mut R) -> u32 {
        assert!(range.start < range.end);
        let range_ = range.end - range.start;
        let max = !0;
        let zone = max - (max % range_);
        loop {
            let v = rng.gen();
            if v < zone {
                return range.start + (v % range_)
            }
        }
    }
}

impl Random<RangeTo<u32>> for u32 {
    fn gen<R: Rng>(range: &RangeTo<u32>, rng: &mut R) -> u32 {
        Random::gen(&(0..range.end), rng)
    }
}
impl Random<RangeFrom<u32>> for u32 {
    #[allow(unsigned_negation)]
    fn gen<R: Rng>(range: &RangeFrom<u32>, rng: &mut R) -> u32 {
        if range.start == 0 {
            return rng.gen()
        }
        let range_ = -range.start;
        let max = !0;
        let zone = max - (max % range_);
        loop {
            let v = rng.gen();
            if v < zone {
                return range.start + (v % range_)
            }
        }
    }
}
impl Random<RangeFull> for i64 {
    fn gen<R: Rng>(_: &RangeFull, rng: &mut R) -> i64 {
        rng.gen()
    }
}
impl Random<Range<i64>> for i64 {
    fn gen<R: Rng>(range: &Range<i64>, rng: &mut R) -> i64 {
        assert!(range.start < range.end);
        let range_ = range.end.wrapping_sub(range.start) as u64;
        let max = !0;
        let zone = max - (max % range_);
        loop {
            let v: u64 = rng.gen();
            if v < zone {
                return range.start.wrapping_add((v % range_) as i64)
            }
        }
    }
}

impl Random<RangeFrom<i64>> for i64 {
    fn gen<R: Rng>(range: &RangeFrom<i64>, rng: &mut R) -> i64 {
        if range.start == -::std::i64::MIN {
            return rng.gen()
        }
        let range_ = -::std::i64::MIN.wrapping_add(range.start) as u64;
        let max = !0;
        let zone = max - (max % range_);
        loop {
            let v = rng.gen();
            if v < zone {
                return range.start.wrapping_add((v % range_) as i64)
            }
        }
    }
}

impl Random<Range<f64>> for f64 {
    fn gen<R: Rng>(range: &Range<f64>, rng: &mut R) -> f64 {
        assert!(range.start < range.end);
        range.start + rng.gen() * (range.end - range.start)
    }
}
impl Random<RangeFull> for f64 {
    fn gen<R: Rng>(_: &RangeFull, rng: &mut R) -> f64 {
        rng.gen()
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
