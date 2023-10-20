use std::ops::{Add, Mul, Neg, Sub};

use itertools::{EitherOrBoth::*, Itertools};

use crate::ntt::*;

#[derive(Debug, Clone)]
pub struct Polynomial {
    pub coef: Vec<i64>,
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
            coef: self.coef.iter().map(|a| -a).collect(),
        }
    }
}

impl Mul<Polynomial> for Polynomial {
    type Output = Polynomial;

    fn mul(self, rhs: Polynomial) -> Self::Output {
        let n = self.coef.len() as i64;
        let M = self.coef.iter().max().unwrap().pow(2) as i64 * n + 1;
        let c = working_modulus(n, M);
        println!("consts -- {:?} {}", c, M);
        let a_forward = forward(self.coef, &c);
        let b_forward = forward(rhs.coef, &c);

        println!("a -- {:?}", a_forward);
        println!("b -- {:?}", b_forward);

        let mul = a_forward
            .iter()
            .rev()
            .zip_longest(b_forward.iter().rev())
            .map(|p| match p {
                Both(&a, &b) => (a * b) % c.N,
                Left(&_a) => 0,
                Right(&_b) => 0,
            })
            .rev()
            .collect::<Vec<i64>>();
        println!("mul -- {:?}", mul);
        Polynomial {
            coef: inverse(mul, &c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Polynomial;

    #[test]
    fn add() {
        let a = Polynomial {
            coef: vec![1, 2, 3, 4],
        };
        let b = Polynomial { coef: vec![1, 2] };
        println!("{:?}", a + b);
    }

    #[test]
    fn mul() {
        let a = Polynomial { coef: vec![1, 2] };
        let b = Polynomial { coef: vec![0, 1] };
        println!("{:?}", a * b);
    }
}
