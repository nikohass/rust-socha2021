#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

fn main() {
    unsafe {
        let b: __m256i = _mm256_set_epi32(1, 2, 3, 4, 5, 6, 7, 8);
        let res = _mm256_add_epi32(b, b);
        println!("{:?}", res);
    }

    unsafe {
        //let zeros = _mm256_setzero_ps();
        //let ones = _mm256_set1_ps(1.0);
        let mut floats = _mm256_set_ps(1.0, 2.0, 3.0, 4.0, 1., 1., 1., 1.);
        floats = _mm256_add_ps(floats, floats);
        floats = _mm256_mul_ps(floats, floats);
        floats = _mm256_div_ps(floats, floats);
        println!("{:?}", floats);
    }
}
