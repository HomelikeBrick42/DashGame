use std::ops::{Add, Mul, Neg, Sub};

pub type T = f32;

mod sealed {
    use super::*;

    pub trait Sealed {}
    impl Sealed for Zero {}
    impl Sealed for T {}
}

pub trait Value: sealed::Sealed + Copy {}
impl Value for Zero {}
impl Value for T {}

#[derive(Debug, Clone, Copy)]
pub struct Zero;

impl From<Zero> for T {
    #[inline]
    fn from(Zero: Zero) -> Self {
        0.0
    }
}

impl Neg for Zero {
    type Output = Zero;

    #[inline]
    fn neg(self) -> Self::Output {
        self
    }
}

impl Add<Zero> for Zero {
    type Output = Zero;

    #[inline]
    fn add(self, Zero: Zero) -> Self::Output {
        self
    }
}

impl Add<T> for Zero {
    type Output = T;

    #[inline]
    fn add(self, rhs: T) -> Self::Output {
        rhs
    }
}

impl Add<Zero> for T {
    type Output = T;

    #[inline]
    fn add(self, Zero: Zero) -> Self::Output {
        self
    }
}

impl Sub<Zero> for Zero {
    type Output = Zero;

    #[inline]
    fn sub(self, Zero: Zero) -> Self::Output {
        self
    }
}

impl Sub<T> for Zero {
    type Output = T;

    #[inline]
    fn sub(self, rhs: T) -> Self::Output {
        -rhs
    }
}

impl Sub<Zero> for T {
    type Output = T;

    #[inline]
    fn sub(self, Zero: Zero) -> Self::Output {
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GenericMultiVector<S, E0, E1, E2, E01, E02, E12, E012>
where
    S: Value,
    E0: Value,
    E1: Value,
    E2: Value,
    E01: Value,
    E02: Value,
    E12: Value,
    E012: Value,
{
    pub s: S,
    pub e0: E0,
    pub e1: E1,
    pub e2: E2,
    pub e01: E01,
    pub e02: E02,
    pub e12: E12,
    pub e012: E012,
}

pub type MultiVector = GenericMultiVector<T, T, T, T, T, T, T, T>;
pub type Scalar = GenericMultiVector<T, Zero, Zero, Zero, Zero, Zero, Zero, Zero>;
pub type Vector = GenericMultiVector<Zero, T, T, T, Zero, Zero, Zero, Zero>;
pub type Line = Vector;
pub type BiVector = GenericMultiVector<Zero, Zero, Zero, Zero, T, T, T, Zero>;
pub type Point = BiVector;
pub type TriVector = GenericMultiVector<Zero, Zero, Zero, Zero, Zero, Zero, Zero, T>;

impl<S, E0, E1, E2, E01, E02, E12, E012> GenericMultiVector<S, E0, E1, E2, E01, E02, E12, E012>
where
    S: Value,
    E0: Value,
    E1: Value,
    E2: Value,
    E01: Value,
    E02: Value,
    E12: Value,
    E012: Value,
{
    #[inline]
    pub fn convert_into<OS, OE0, OE1, OE2, OE01, OE02, OE12, OE012>(
        self,
    ) -> GenericMultiVector<OS, OE0, OE1, OE2, OE01, OE02, OE12, OE012>
    where
        OS: Value,
        OE0: Value,
        OE1: Value,
        OE2: Value,
        OE01: Value,
        OE02: Value,
        OE12: Value,
        OE012: Value,
        S: Into<OS>,
        E0: Into<OE0>,
        E1: Into<OE1>,
        E2: Into<OE2>,
        E01: Into<OE01>,
        E02: Into<OE02>,
        E12: Into<OE12>,
        E012: Into<OE012>,
    {
        GenericMultiVector {
            s: self.s.into(),
            e0: self.e0.into(),
            e1: self.e1.into(),
            e2: self.e2.into(),
            e01: self.e01.into(),
            e02: self.e02.into(),
            e12: self.e12.into(),
            e012: self.e012.into(),
        }
    }
}

type NO<A> = <A as Neg>::Output;

impl<S, E0, E1, E2, E01, E02, E12, E012> Neg
    for GenericMultiVector<S, E0, E1, E2, E01, E02, E12, E012>
where
    S: Value + Neg,
    E0: Value + Neg,
    E1: Value + Neg,
    E2: Value + Neg,
    E01: Value + Neg,
    E02: Value + Neg,
    E12: Value + Neg,
    E012: Value + Neg,
    NO<S>: Value,
    NO<E0>: Value,
    NO<E1>: Value,
    NO<E2>: Value,
    NO<E01>: Value,
    NO<E02>: Value,
    NO<E12>: Value,
    NO<E012>: Value,
{
    type Output =
        GenericMultiVector<NO<S>, NO<E0>, NO<E1>, NO<E2>, NO<E01>, NO<E02>, NO<E12>, NO<E012>>;

    fn neg(self) -> Self::Output {
        Self::Output {
            s: -self.s,
            e0: -self.e0,
            e1: -self.e1,
            e2: -self.e2,
            e01: -self.e01,
            e02: -self.e02,
            e12: -self.e12,
            e012: -self.e012,
        }
    }
}

type AO<A, B> = <A as Add<B>>::Output;

impl<LS, LE0, LE1, LE2, LE01, LE02, LE12, LE012, RS, RE0, RE1, RE2, RE01, RE02, RE12, RE012>
    Add<GenericMultiVector<RS, RE0, RE1, RE2, RE01, RE02, RE12, RE012>>
    for GenericMultiVector<LS, LE0, LE1, LE2, LE01, LE02, LE12, LE012>
where
    LS: Value + Add<RS>,
    LE0: Value + Add<RE0>,
    LE1: Value + Add<RE1>,
    LE2: Value + Add<RE2>,
    LE01: Value + Add<RE01>,
    LE02: Value + Add<RE02>,
    LE12: Value + Add<RE12>,
    LE012: Value + Add<RE012>,
    RS: Value,
    RE0: Value,
    RE1: Value,
    RE2: Value,
    RE01: Value,
    RE02: Value,
    RE12: Value,
    RE012: Value,
    AO<LS, RS>: Value,
    AO<LE0, RE0>: Value,
    AO<LE1, RE1>: Value,
    AO<LE2, RE2>: Value,
    AO<LE01, RE01>: Value,
    AO<LE02, RE02>: Value,
    AO<LE12, RE12>: Value,
    AO<LE012, RE012>: Value,
{
    type Output = GenericMultiVector<
        AO<LS, RS>,
        AO<LE0, RE0>,
        AO<LE1, RE1>,
        AO<LE2, RE2>,
        AO<LE01, RE01>,
        AO<LE02, RE02>,
        AO<LE12, RE12>,
        AO<LE012, RE012>,
    >;

    #[inline]
    fn add(
        self,
        rhs: GenericMultiVector<RS, RE0, RE1, RE2, RE01, RE02, RE12, RE012>,
    ) -> Self::Output {
        Self::Output {
            s: self.s + rhs.s,
            e0: self.e0 + rhs.e0,
            e1: self.e1 + rhs.e1,
            e2: self.e2 + rhs.e2,
            e01: self.e01 + rhs.e01,
            e02: self.e02 + rhs.e02,
            e12: self.e12 + rhs.e12,
            e012: self.e012 + rhs.e012,
        }
    }
}

type SO<A, B> = <A as Sub<B>>::Output;

impl<LS, LE0, LE1, LE2, LE01, LE02, LE12, LE012, RS, RE0, RE1, RE2, RE01, RE02, RE12, RE012>
    Sub<GenericMultiVector<RS, RE0, RE1, RE2, RE01, RE02, RE12, RE012>>
    for GenericMultiVector<LS, LE0, LE1, LE2, LE01, LE02, LE12, LE012>
where
    LS: Value + Sub<RS>,
    LE0: Value + Sub<RE0>,
    LE1: Value + Sub<RE1>,
    LE2: Value + Sub<RE2>,
    LE01: Value + Sub<RE01>,
    LE02: Value + Sub<RE02>,
    LE12: Value + Sub<RE12>,
    LE012: Value + Sub<RE012>,
    RS: Value,
    RE0: Value,
    RE1: Value,
    RE2: Value,
    RE01: Value,
    RE02: Value,
    RE12: Value,
    RE012: Value,
    SO<LS, RS>: Value,
    SO<LE0, RE0>: Value,
    SO<LE1, RE1>: Value,
    SO<LE2, RE2>: Value,
    SO<LE01, RE01>: Value,
    SO<LE02, RE02>: Value,
    SO<LE12, RE12>: Value,
    SO<LE012, RE012>: Value,
{
    type Output = GenericMultiVector<
        SO<LS, RS>,
        SO<LE0, RE0>,
        SO<LE1, RE1>,
        SO<LE2, RE2>,
        SO<LE01, RE01>,
        SO<LE02, RE02>,
        SO<LE12, RE12>,
        SO<LE012, RE012>,
    >;

    #[inline]
    fn sub(
        self,
        rhs: GenericMultiVector<RS, RE0, RE1, RE2, RE01, RE02, RE12, RE012>,
    ) -> Self::Output {
        Self::Output {
            s: self.s - rhs.s,
            e0: self.e0 - rhs.e0,
            e1: self.e1 - rhs.e1,
            e2: self.e2 - rhs.e2,
            e01: self.e01 - rhs.e01,
            e02: self.e02 - rhs.e02,
            e12: self.e12 - rhs.e12,
            e012: self.e012 - rhs.e012,
        }
    }
}

/*
(a + b*e0 + c*e1 + d*e2 + e*e01 + f*e02 + g*e12 + h*e012)(i + j*e0 + k*e1 + l*e2 + m*e01 + n*e02 + o*e12 + p*e012)

a*(i + j*e0 + k*e1 + l*e2 + m*e01 + n*e02 + o*e12 + p*e012)
+ b*e0*(i + j*e0 + k*e1 + l*e2 + m*e01 + n*e02 + o*e12 + p*e012)
+ c*e1*(i + j*e0 + k*e1 + l*e2 + m*e01 + n*e02 + o*e12 + p*e012)
+ d*e2*(i + j*e0 + k*e1 + l*e2 + m*e01 + n*e02 + o*e12 + p*e012)
+ e*e01*(i + j*e0 + k*e1 + l*e2 + m*e01 + n*e02 + o*e12 + p*e012)
+ f*e02*(i + j*e0 + k*e1 + l*e2 + m*e01 + n*e02 + o*e12 + p*e012)
+ g*e12*(i + j*e0 + k*e1 + l*e2 + m*e01 + n*e02 + o*e12 + p*e012)
+ h*e012*(i + j*e0 + k*e1 + l*e2 + m*e01 + n*e02 + o*e12 + p*e012)

a*i + a*j*e0 + a*k*e1 + a*l*e2 + a*m*e01 + a*n*e02 + a*o*e12 + a*p*e012
+ b*e0*i + b*e0*j*e0 + b*e0*k*e1 + b*e0*l*e2 + b*e0*m*e01 + b*e0*n*e02 + b*e0*o*e12 + b*e0*p*e012
+ c*e1*i + c*e1*j*e0 + c*e1*k*e1 + c*e1*l*e2 + c*e1*m*e01 + c*e1*n*e02 + c*e1*o*e12 + c*e1*p*e012
+ d*e2*i + d*e2*j*e0 + d*e2*k*e1 + d*e2*l*e2 + d*e2*m*e01 + d*e2*n*e02 + d*e2*o*e12 + d*e2*p*e012
+ e*e01*i + e*e01*j*e0 + e*e01*k*e1 + e*e01*l*e2 + e*e01*m*e01 + e*e01*n*e02 + e*e01*o*e12 + e*e01*p*e012
+ f*e02*i + f*e02*j*e0 + f*e02*k*e1 + f*e02*l*e2 + f*e02*m*e01 + f*e02*n*e02 + f*e02*o*e12 + f*e02*p*e012
+ g*e12*i + g*e12*j*e0 + g*e12*k*e1 + g*e12*l*e2 + g*e12*m*e01 + g*e12*n*e02 + g*e12*o*e12 + g*e12*p*e012
+ h*e012*i + h*e012*j*e0 + h*e012*k*e1 + h*e012*l*e2 + h*e012*m*e01 + h*e012*n*e02 + h*e012*o*e12 + h*e012*p*e012

a*i + a*j*e0 + a*k*e1 + a*l*e2 + a*m*e01 + a*n*e02 + a*o*e12 + a*p*e012
+ b*e0*i + b*e0*k*e1 + b*e0*l*e2 + b*e0*o*e12
+ c*e1*i + c*e1*j*e0 + c*e1*k*e1 + c*e1*l*e2 + c*e1*m*e01 + c*e1*n*e02 + c*e1*o*e12 + c*e1*p*e012
+ d*e2*i + d*e2*j*e0 + d*e2*k*e1 + d*e2*l*e2 + d*e2*m*e01 + d*e2*n*e02 + d*e2*o*e12 + d*e2*p*e012
+ e*e01*i + e*e01*k*e1 + e*e01*l*e2 + e*e01*o*e12
+ f*e02*i + f*e02*k*e1 + f*e02*l*e2 + f*e02*o*e12
+ g*e12*i + g*e12*j*e0 + g*e12*k*e1 + g*e12*l*e2 + g*e12*m*e01 + g*e12*n*e02 + g*e12*o*e12 + g*e12*p*e012
+ h*e012*i + h*e012*k*e1 + h*e012*l*e2 + h*e012*o*e12

a*i + a*j*e0 + a*k*e1 + a*l*e2 + a*m*e01 + a*n*e02 + a*o*e12 + a*p*e012
+ b*i*e0 + b*k*e01 + b*l*e02 + b*o*e012
+ c*i*e1 - c*j*e01 + c*k + c*l*e12 - c*m*e0 - c*n*e012 + c*o*e2 - c*p*e02
+ d*i*e2 - d*j*e02 - d*k*e12 + d*l + d*m*e012 - d*n*e0 - d*o*e1 + d*p*e01
+ e*i*e01 + e*k*e0 + e*l*e012 + e*o*e2
+ f*i*e02 - f*k*e012 + f*l*e0 - f*o*e01
+ g*i*e12 + g*j*e012 - g*k*e2 + g*l*e1 - g*m*e02 + g*n*e1 - g*o - g*p*e0
+ h*i*e012 - h*k*e02 + h*l*e01 - h*o*e0

(a*i + c*k + d*l - g*o)
+ (a*j + b*i - c*m - d*n + e*k + f*l + g*p + h*o)*e0
+ (a*k + c*i - d*o + g*l + g*n)*e1
+ (a*l + d*i + c*o + d*i + e*o - g*k)*e2
+ (a*m + b*k - c*j + d*p + e*i - f*o + h*l)*e01
+ (a*n + b*l - d*j - c*p + f*i - g*m - h*k)*e02
+ (a*o + c*l - d*k + g*i)*e12
+ (a*p + b*p - c*n + d*m + e*l - f*k + g*j + h*i)*e012
*/

type MO<A, B> = <A as Mul<B>>::Output;

impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P> Mul<GenericMultiVector<I, J, K, L, M, N, O, P>>
    for GenericMultiVector<A, B, C, D, E, F, G, H>
where
    A: Value + Mul<I> + Mul<J> + Mul<K> + Mul<L>,
    B: Value + Mul<I>,
    C: Value + Mul<K> + Mul<M> + Mul<I> + Mul<O>,
    D: Value + Mul<L> + Mul<N> + Mul<O> + Mul<I>,
    E: Value + Mul<K> + Mul<O>,
    F: Value + Mul<L>,
    G: Value + Mul<O> + Mul<P> + Mul<L> + Mul<N> + Mul<K>,
    H: Value + Mul<O>,
    I: Value,
    J: Value,
    K: Value,
    L: Value,
    M: Value,
    N: Value,
    O: Value,
    P: Value,
    MO<A, I>: Add<MO<C, K>>,
    MO<A, J>: Add<MO<B, I>>,
    MO<A, K>: Add<MO<C, I>>,
    MO<A, L>: Add<MO<D, I>>,
    AO<MO<A, I>, MO<C, K>>: Add<MO<D, L>>,
    AO<MO<A, J>, MO<B, I>>: Sub<MO<C, M>>,
    AO<MO<A, K>, MO<C, I>>: Sub<MO<D, O>>,
    AO<MO<A, L>, MO<D, I>>: Add<MO<C, O>>,
    AO<AO<MO<A, I>, MO<C, K>>, MO<D, L>>: Sub<MO<G, O>>,
    SO<AO<MO<A, J>, MO<B, I>>, MO<C, M>>: Sub<MO<D, N>>,
    SO<AO<MO<A, K>, MO<C, I>>, MO<D, O>>: Add<MO<G, L>>,
    AO<AO<MO<A, L>, MO<D, I>>, MO<C, O>>: Add<MO<D, I>>,
    SO<AO<AO<MO<A, I>, MO<C, K>>, MO<D, L>>, MO<G, O>>: Value,
    SO<SO<AO<MO<A, J>, MO<B, I>>, MO<C, M>>, MO<D, N>>: Add<MO<E, K>>,
    AO<SO<AO<MO<A, K>, MO<C, I>>, MO<D, O>>, MO<G, L>>: Add<MO<G, N>>,
    AO<AO<AO<MO<A, L>, MO<D, I>>, MO<C, O>>, MO<D, I>>: Add<MO<E, O>>,
    AO<SO<SO<AO<MO<A, J>, MO<B, I>>, MO<C, M>>, MO<D, N>>, MO<E, K>>: Add<MO<F, L>>,
    AO<AO<SO<AO<MO<A, K>, MO<C, I>>, MO<D, O>>, MO<G, L>>, MO<G, N>>: Value,
    AO<AO<AO<AO<MO<A, L>, MO<D, I>>, MO<C, O>>, MO<D, I>>, MO<E, O>>: Sub<MO<G, K>>,
    AO<AO<SO<SO<AO<MO<A, J>, MO<B, I>>, MO<C, M>>, MO<D, N>>, MO<E, K>>, MO<F, L>>: Add<MO<G, P>>,
    SO<AO<AO<AO<AO<MO<A, L>, MO<D, I>>, MO<C, O>>, MO<D, I>>, MO<E, O>>, MO<G, K>>: Value,
    AO<AO<AO<SO<SO<AO<MO<A, J>, MO<B, I>>, MO<C, M>>, MO<D, N>>, MO<E, K>>, MO<F, L>>, MO<G, P>>:
        Add<MO<H, O>>,
    AO<
        AO<
            AO<AO<SO<SO<AO<MO<A, J>, MO<B, I>>, MO<C, M>>, MO<D, N>>, MO<E, K>>, MO<F, L>>,
            MO<G, P>,
        >,
        MO<H, O>,
    >: Value,
{
    type Output = GenericMultiVector<
        SO<AO<AO<MO<A, I>, MO<C, K>>, MO<D, L>>, MO<G, O>>,
        AO<
            AO<
                AO<AO<SO<SO<AO<MO<A, J>, MO<B, I>>, MO<C, M>>, MO<D, N>>, MO<E, K>>, MO<F, L>>,
                MO<G, P>,
            >,
            MO<H, O>,
        >,
        AO<AO<SO<AO<MO<A, K>, MO<C, I>>, MO<D, O>>, MO<G, L>>, MO<G, N>>,
        SO<AO<AO<AO<AO<MO<A, L>, MO<D, I>>, MO<C, O>>, MO<D, I>>, MO<E, O>>, MO<G, K>>,
        Zero,
        Zero,
        Zero,
        Zero,
    >;

    fn mul(self, rhs: GenericMultiVector<I, J, K, L, M, N, O, P>) -> Self::Output {
        let a = self.s;
        let b = self.e0;
        let c = self.e1;
        let d = self.e2;
        let e = self.e01;
        let f = self.e02;
        let g = self.e12;
        let h = self.e012;
        let i = rhs.s;
        let j = rhs.e0;
        let k = rhs.e1;
        let l = rhs.e2;
        let m = rhs.e01;
        let n = rhs.e02;
        let o = rhs.e12;
        let p = rhs.e012;
        Self::Output {
            s: a * i + c * k + d * l - g * o,
            e0: a * j + b * i - c * m - d * n + e * k + f * l + g * p + h * o,
            e1: a * k + c * i - d * o + g * l + g * n,
            e2: a * l + d * i + c * o + d * i + e * o - g * k,
            e01: Zero,
            e02: Zero,
            e12: Zero,
            e012: Zero,
        }
    }
}
