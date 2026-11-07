mod array_ops;
mod binary_trees;
mod builtin_sort;
mod closures;
mod fib;
mod float_loop;
mod hash_map;
mod int_loop;
mod mandelbrot;
mod nbody;
mod quicksort_ints;
mod sort_userdata;
mod spectral_norm;
mod strings;

use crate::harness::Case;

/// Every benchmark case. Add one entry here per case.
pub static CASES: &[Case] = &[
    sort_userdata::CASE,
    fib::CASE,
    int_loop::CASE,
    float_loop::CASE,
    nbody::CASE,
    spectral_norm::CASE,
    mandelbrot::CASE,
    binary_trees::CASE,
    quicksort_ints::CASE,
    strings::CASE,
    array_ops::CASE,
    hash_map::CASE,
    builtin_sort::CASE,
    closures::CASE,
];
