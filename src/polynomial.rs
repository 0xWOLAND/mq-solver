use std::{
    fmt::Display,
    ops::{Add, Mul, Neg, Sub},
};

use itertools::{EitherOrBoth::*, Itertools};
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};

use crate::{ntt::*, numbers::BigInt};

#[derive(Debug, Clone)]
pub struct Polynomial {
    pub coef: Vec<BigInt>,
}

impl Polynomial {
    pub fn new(coef: Vec<BigInt>) -> Self {
        let n = coef.len();
        let ZERO = BigInt::from(0);

        // if is not power of 2
        if !(n & (n - 1) == 0) {
            let pad = n.next_power_of_two() - n;
            return Self {
                coef: vec![ZERO; pad]
                    .into_iter()
                    .chain(coef.into_iter())
                    .collect_vec(),
            };
        }
        Self { coef }
    }

    pub fn mul_brute(self, rhs: Polynomial) -> Polynomial {
        let a = self.len();
        let b = rhs.len();
        let ZERO = BigInt::from(0);

        let mut out: Vec<BigInt> = vec![ZERO; a + b];

        for i in 0..a {
            for j in 0..b {
                let e = i + j;
                out[e] += self.coef[i] * rhs.coef[j];
            }
        }

        Polynomial { coef: out }
    }

    pub fn mul(self, rhs: Polynomial, c: &Constants) -> Polynomial {
        let v1_deg = self.degree();
        let v2_deg = rhs.degree();
        let n = (self.len() + rhs.len()).next_power_of_two();
        let ZERO = BigInt::from(0);

        let v1 = vec![ZERO; n - self.len()]
            .into_iter()
            .chain(self.coef.into_iter())
            .collect();
        let v2 = vec![ZERO; n - rhs.len()]
            .into_iter()
            .chain(rhs.coef.into_iter())
            .collect();

        let a_forward = forward(v1, &c);
        let b_forward = forward(v2, &c);

        let mut mul = vec![ZERO; n as usize];
        mul.par_iter_mut()
            .enumerate()
            .for_each(|(i, x)| *x = (a_forward[i] * b_forward[i]).rem(c.N));

        let coef = inverse(mul, &c);
        let start = coef.iter().position(|&x| x != 0).unwrap();

        Polynomial {
            coef: coef[start..=(start + v1_deg + v2_deg)].to_vec(),
        }
    }

    pub fn diff(mut self) -> Self {
        let N = self.len();
        for n in (1..N).rev() {
            self.coef[n] = self.coef[n - 1] * BigInt::from(N - n);
        }
        self.coef[0] = BigInt::from(0);
        let start = self.coef.iter().position(|&x| x != 0).unwrap();
        self.coef = self.coef[start..].to_vec();

        self
    }

    pub fn len(&self) -> usize {
        self.coef.len()
    }

    pub fn degree(&self) -> usize {
        let start = self.coef.iter().position(|&x| x != 0).unwrap();
        self.len() - start - 1
    }
}

impl Add<Polynomial> for Polynomial {
    type Output = Polynomial;

    fn add(self, rhs: Polynomial) -> Self::Output {
        Polynomial {
            coef: self
                .coef
                .iter()
                .rev()
                .zip_longest(rhs.coef.iter().rev())
                .map(|p| match p {
                    Both(&a, &b) => a + b,
                    Left(&a) => a,
                    Right(&b) => b,
                })
                .rev()
                .collect(),
        }
    }
}

impl Sub<Polynomial> for Polynomial {
    type Output = Polynomial;

    fn sub(self, rhs: Polynomial) -> Self::Output {
        self + (-rhs)
    }
}

impl Neg for Polynomial {
    type Output = Polynomial;

    fn neg(self) -> Self::Output {
        Polynomial {
            coef: self.coef.iter().map(|a| -(*a)).collect(),
        }
    }
}

impl Display for Polynomial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.coef.iter().map(|&x| write!(f, "{} ", x)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Polynomial;
    use crate::{ntt::working_modulus, numbers::BigInt};

    #[test]
    fn add() {
        let a = Polynomial::new(vec![1, 2, 3, 4].iter().map(|&x| BigInt::from(x)).collect());
        let b = Polynomial::new(vec![1, 2].iter().map(|&x| BigInt::from(x)).collect());
        println!("{}", a + b);
    }

    #[test]
    fn mul() {
        let a = Polynomial::new(vec![1, 2, 3].iter().map(|&x| BigInt::from(x)).collect());
        let b = Polynomial::new(
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9]
                .iter()
                .map(|&x| BigInt::from(x))
                .collect(),
        );

        let N = BigInt::from((a.len() + b.len()).next_power_of_two());
        let M = (*a
            .coef
            .iter()
            .max()
            .unwrap()
            .max(b.coef.iter().max().unwrap())
            << 1)
            * N
            + 1;
        let c = working_modulus(N, M);

        println!("{}", a.mul(b, &c));
    }

    #[test]
    fn diff() {
        let a = Polynomial::new(vec![3, 2, 1].iter().map(|&x| BigInt::from(x)).collect());
        let da = a.diff();
        println!("{}", da);
    }
}
