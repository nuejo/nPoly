extern crate chrono;
extern crate rand;
use crate::algebras::real::*;
use crate::algebras::ScalarRing;
use crate::algebras::complex::*;
use rand::distributions::Distribution;


use rand::distributions::uniform::{UniformFloat, UniformSampler, UniformInt};

use rand::prelude::Rng;

pub trait Testable: ScalarRing {
    type Sampler: Distribution<Self>;
    fn gen_sampler(max_size: Self) -> Self::Sampler;
}


#[derive(Clone, Copy, Debug)]
pub struct DistCC(UniformFloat<f64>, UniformFloat<f64>);

impl Distribution<CC> for DistCC {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CC {
        CC::new(self.0.sample(rng), self.0.sample(rng))
    }
}

impl Testable for CC {
    type Sampler = DistCC;

    fn gen_sampler(max_size: Self) -> Self::Sampler {
        let bound = (max_size.0.norm() / 2.0).sqrt();
        DistCC(UniformFloat::<f64>::new(-bound, bound), 
               UniformFloat::<f64>::new(-bound, bound))
    }
}

use crate::algebras::integers::*;

#[derive(Clone, Copy, Debug)]
pub struct DistZZ(UniformInt<i32>);

impl Distribution<ZZ> for DistZZ {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ZZ {
        ZZ(self.0.sample(rng))
    }
}

impl Testable for ZZ {
    type Sampler = DistZZ;

    fn gen_sampler(max_size: Self) -> Self::Sampler {
        DistZZ(UniformInt::<i32>::new(-max_size.0.abs(), max_size.0.abs()))
    }
}

use crate::algebras::finite_field::FF;

#[derive(Clone, Copy, Debug)]
pub struct DistFF(UniformInt<i32>);

impl Distribution<FF> for DistFF {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> FF {
        FF(self.0.sample(rng))
    }
}

impl Testable for FF {
    type Sampler = DistFF;

    fn gen_sampler(max_size: Self) -> Self::Sampler {
        DistFF(UniformInt::<i32>::new(-max_size.0.abs(), max_size.0.abs()))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DistRR(UniformFloat<f64>);

impl Distribution<RR> for DistRR {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> RR {
        RR(self.0.sample(rng))
    }
}

impl Testable for RR {
    type Sampler = DistRR;

    fn gen_sampler(max_size: Self) -> Self::Sampler {
        DistRR(UniformFloat::<f64>::new(-max_size.0.abs(), max_size.0.abs()))
    }
}


use crate::algebras::polyring::*;
use chrono::Duration;
use generic_array::GenericArray;


/// Specifies the test that is going to be performed
///
/// num_el:    Number of elements in the polynomial
/// deg_size:  Maximum degrees of each of the indeterminates
/// mult_func: The multiplication function being used
/// ring:      The ring of the polynomials (TODO this isn't really necessary, we could just make
///            our own
///
#[derive(Clone)]
pub struct Tester<'a, P: PolyRing> {
    pub num_el: usize,
    pub deg_sizes: GenericArray<usize, P::NumVar>,
    pub coeff_size: P::Coeff,
    pub mult_func: fn(&Poly<'a, P>, &Poly<'a, P>) -> Poly<'a, P>,
    pub ring: &'a P,
}


// use typenum::Unsigned;

// impl<'a, P: PolyRing<Coeff=RR>> Poly<'a, P> {
//     pub fn new(sparsity: f64, num: usize) -> Tester<'a, P> {
//         let num_f: f64 = num as f64;

//         if sparsity > 1.0 || sparsity <= 0.0 {
//             panic!("Sparsity needs to be 0.0 < sparsity <= 1.0. Sparsity given = {}", sparsity);
//         }
        
//         // The individual degree bounds of each of the indeterminates
//         let deg_bounds = num_f.powf(1.0 / (P::NumVar::to_usize() as f64)).ceil();

//         let num_terms = (sparsity * num_f).floor();





//     }
// }

/// Public testing interface
impl<'a, P: PolyRing<Coeff: Testable>> Poly<'a, P> {
    pub fn mult_tester(tester: Tester<'a, P>) -> Duration {

        let mut rng = rand::thread_rng();
        let dist = <P::Coeff as Testable>::gen_sampler(tester.coeff_size);

        // A function to randomly generate a polynomial with n coefficients
        let mut make_poly = || -> Self {
            let res_vec = 
                (0..tester.num_el).map(|_| 
                     Term {
                         coeff: dist.sample(&mut rng),
                         mon: tester.deg_sizes.iter()
                             .map(|i| rng.gen::<usize>() % i)
                             .collect()
                        }).collect();
            Poly::from_terms(res_vec, Some(tester.ring))
        };

        let a = make_poly();
        let b = make_poly();

        Duration::span(|| { (tester.mult_func)(&a, &b); })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::polym::*;
    use crate::kronecker::*;
    use generic_array::*;
    use typenum::{U2, U10};
    use crate::algebras::MyMulMonoid;

    #[test]
    fn mult_tester_test() {

        let ring = PRDomain::<RR, GLex<MultiIndex<U2>>>::new(vec!['x', 'y']);

        let mut tester = Tester {
            num_el: 5,
            deg_sizes: arr![usize; 2, 3],
            coeff_size: RR(10.0),
            mult_func: kronecker_mult,
            ring: &ring,
        };

        let time = Poly::mult_tester(tester.clone());
        println!("Kronecker time = {} microseconds", time.num_microseconds().unwrap());

        tester.mult_func = Poly::ref_mul;

        let time = Poly::mult_tester(tester);
        println!("Hashmap time = {} microseconds", time.num_microseconds().unwrap());


        let ring = PRDomain::<RR, GLex<MultiIndex<U10>>>::new(vec!['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j']);
        let mut tester_big = Tester {
            num_el: 1000,
            deg_sizes: arr![usize; 20, 20, 20, 20, 20, 20, 20, 20, 20, 20],
            coeff_size: RR(10.0),
            mult_func: kronecker_mult,
            ring: &ring,
        };

        let time = Poly::mult_tester(tester_big.clone());
        println!("Kronecker time = {} microseconds", time.num_microseconds().unwrap());

        tester_big.mult_func = Poly::ref_mul;

        let time = Poly::mult_tester(tester_big);
        println!("Hashmap time = {} microseconds", time.num_microseconds().unwrap());
    }


    use crate::algebras::complex::CC;
    use crate::polyu::UniVarOrder;

    #[test]
    fn bench_dense_main() {
        let ring = PRDomain::<CC, UniVarOrder>::new(vec!['x']);

        let mut tester_big = Tester {
            num_el: 1000,
            deg_sizes: arr![usize; 1000],
            coeff_size: CC::new(10.0, 10.0),
            mult_func: kronecker_mult,
            ring: &ring,
        };

        let time = Poly::mult_tester(tester_big.clone());
        println!("FFT time = {} microseconds", time.num_microseconds().unwrap());

    }
}
