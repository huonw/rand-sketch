use std::marker;
use std::ops::{Range, RangeFull};
use rand::Rng;

pub trait Rand<Distribution> {
    type Stream: RandStream<Self>;
    fn rand(dist: Distribution) -> Self::Stream;
}

pub trait RandStream<T> {
    fn next<R: Rng>(&self, rng: &mut R) -> T;
}

/// Create a single random value, mediated by `constraint`.
pub fn gen<Gen: Rand<Dist>, Dist, R: Rng>(rng: &mut R, dist: Dist)
                                             -> Gen
{
    Gen::rand(dist).next(rng)
}

/// Create an infinite sequence of random values, mediated by `constraint`.
pub fn gen_iter<Gen: Rand<Dist>, Dist, R: Rng>(rng: R, dist: Dist)
    -> GenIter<Gen, Dist, R>
{
    GenIter {
        stream: Gen::rand(dist),
        rng: rng,
    }
}
pub struct GenIter<Gen: Rand<Dist>, Dist, R: Rng> {
    stream: Gen::Stream,
    rng: R,
}
impl<Gen: Rand<Dist>, Dist, R: Rng> Iterator for GenIter<Gen, Dist, R> {
    type Item = Gen;

    fn next(&mut self) -> Option<Gen> {
        Some(self.stream.next(&mut self.rng))
    }
}

pub struct IntegerStreamBounded<T> {
    low: T,
    range: T,
    accept_zone: T,
}
pub struct IntegerStreamFull<T> {
    _marker: marker::PhantomData<T>,
}

impl Rand<Range<u32>> for u32 {
    type Stream = IntegerStreamBounded<u32>;
    fn rand(dist: Range<u32>) -> IntegerStreamBounded<u32> {
        assert!(dist.start < dist.end);
        let range = dist.end - dist.start;
        let max = !0;
        let zone = max - (max % range);
        IntegerStreamBounded {
            low: dist.start,
            range: range,
            accept_zone: zone,
        }
    }
}
impl Rand<RangeFull> for u32 {
    type Stream = IntegerStreamFull<u32>;
    fn rand(_dist: RangeFull) -> IntegerStreamFull<u32> {
        IntegerStreamFull {
            _marker: marker::PhantomData,
        }
    }
}

impl RandStream<u32> for IntegerStreamBounded<u32> {
    fn next<R: Rng>(&self, rng: &mut R) -> u32 {
        loop {
            let v = rng.next_u32();

            if v < self.accept_zone {
                return self.low.wrapping_add((v % self.range))
            }
        }
    }
}

impl RandStream<u32> for IntegerStreamFull<u32> {
    fn next<R: Rng>(&self, rng: &mut R) -> u32 {
        rng.next_u32()
    }
}


use std::mem;

impl Rand<Range<i64>> for i64 {
    type Stream = IntegerStreamBounded<i64>;
    fn rand(dist: Range<i64>) -> IntegerStreamBounded<i64> {
        assert!(dist.start < dist.end);
        let range = dist.end.wrapping_sub(dist.start);
        let max = !0;
        let zone = max - (max % range);
        IntegerStreamBounded {
            low: dist.start,
            range: range,
            accept_zone: unsafe {mem::transmute(zone)},
        }
    }
}
impl Rand<RangeFull> for i64 {
    type Stream = IntegerStreamFull<i64>;
    fn rand(_dist: RangeFull) -> IntegerStreamFull<i64> {
        IntegerStreamFull {
            _marker: marker::PhantomData,
        }
    }
}


impl RandStream<i64> for IntegerStreamBounded<i64> {
    fn next<R: Rng>(&self, rng: &mut R) -> i64 {
        let zone: u64 = unsafe {mem::transmute(self.accept_zone)};
        let range: u64 = unsafe {mem::transmute(self.range)};
        loop {
            let v = rng.next_u64();

            if v < zone {
                let value: i64 = unsafe {mem::transmute(v % range)};
                return self.low.wrapping_add(value)
            }
        }
    }
}
impl RandStream<i64> for IntegerStreamFull<i64> {
    fn next<R: Rng>(&self, rng: &mut R) -> i64 {
        unsafe {mem::transmute(rng.next_u64())}
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
