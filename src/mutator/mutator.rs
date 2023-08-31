//! Basic fuzzer mutation strategies, largely ported from honggfuzz
/*
 *
 * Authors:
 * Robert Swiecki <swiecki@google.com>
 * Brandon Falk <bfalk@gamozolabs.com>
 *
 * Copyright 2010-2018 by Google Inc. All Rights Reserved.
 * Copyright 2020 by Brandon Falk <bfalk@gamozolabs.com>
 *
 * Licensed under the Apache License, Version 2.0 (the "License"); you may
 * not use this file except in compliance with the License. You may obtain
 * a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
 * implied. See the License for the specific language governing
 * permissions and limitations under the License.
 *
 */

use felt::Felt252;
extern crate alloc;

/// An empty database that never returns an input, useful for fuzzers without
/// corpuses or input databases.
pub struct EmptyDatabase;

impl InputDatabase for EmptyDatabase {
    fn num_inputs(&self) -> usize { 0 }
    fn input(&self, _idx: usize) -> Option<&[Felt252]> { None }
}

/// Routines to generically access a corpus/input database for a fuzzer. It's
/// up to the database to implement this trait, allowing generic access to
/// the number of inputs in the database, and accessors for a specific input.
///
/// The inputs should have zero-indexed IDs such that any input in the range of
/// [0, self.num_inputs()) should be a valid input.
///
/// If the `idx` does not lead to the same input each run, the determinism of
/// the mutator is unstable and can produce different results across different
/// runs.
pub trait InputDatabase {
    /// Get the number of inputs in the database
    fn num_inputs(&self) -> usize;

    /// Get an input with a specific zero-index identifier
    /// If the `idx` is invalid or otherwise not available, this returns `None`
    fn input(&self, idx: usize) -> Option<&[Felt252]>;
}

/// A basic random number generator based on xorshift64 with 64-bits of state
pub struct Rng {
    /// The RNG's seed and state
    pub seed: u64,

    /// If set, `rand_exp` behaves the same as `rand`
    pub exp_disabled: bool,
}

impl Rng {
    /// Generate a random number
    #[inline]
    pub fn next(&mut self) -> u64 {
        let val = self.seed;
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 43;
        val
    }

    /// Generates a random number with uniform distribution in the range of
    /// [min, max]
    #[inline]
    pub fn rand(&mut self, min: usize, max: usize) -> usize {
        // Make sure the range is sane
        assert!(max >= min, "Bad range specified for rand()");

        // If there is no range, just return `min`
        if min == max {
            return min;
        }
        
        // If the range is unbounded, just return a random number
        if min == 0 && max == core::usize::MAX {
            return self.next() as usize;
        }

        // Pick a random number in the range
        min + (self.next() as usize % (max - min + 1))
    }
    
    /// Generates a random number with exponential distribution in the range of
    /// [min, max] with a worst case deviation from uniform of 0.5x. Meaning
    /// this will always return uniform at least half the time.
    #[inline]
    pub fn rand_exp(&mut self, min: usize, max: usize) -> usize {
        // If exponential random is disabled, fall back to uniform
        if self.exp_disabled {
            return self.rand(min, max);
        }

        if self.rand(0, 1) == 0 {
            // Half the time, provide uniform
            self.rand(min, max)
        } else {
            // Pick an exponentially difficult random number
            let x = self.rand(min, max);
            self.rand(min, x)
        }
    }
}
/// An empty database that never returns an input, useful for fuzzers without
/// corpuses or input databases.

/// A mutator, a playground for corrupting the public `input` vector when
/// `mutate` is invoked
pub struct Mutator {
    /// Input vector to mutate, this is just an entire input files bytes
    ///
    /// It is strongly recommended that you do `input.clear()` and
    /// `input.extend_from_slice()` to update this buffer, to prevent the
    /// backing from being deallocated and reallocated.
    pub input: Vec<Felt252>,

    /// If non-zero length, this contains a list of valid indicies into
    /// `input`, indicating which bytes of the input should mutated. This often
    /// comes from instrumentation like access tracking or taint tracking to
    /// indicate which parts of the input are used. This will prevent us from
    /// corrupting parts of the input which have zero effect on the program.
    ///
    /// It's possible you can have this take any meaning you want, all it does
    /// is limit the corruption/splicing locations to the indicies in this
    /// vector. Feel free to change this to have different meanings, like
    /// indicate indicies which are used in comparison instructions!
    ///
    /// Since we use `rand_exp` to pick from this, this list should remain
    /// sorted for best behavior. If you cannot sort this, you should probably
    /// change the behaviors of `rand_offset` to be uniform
    ///
    /// It is strongly recommended that you do `accessed.clear()` and
    /// `accessed.extend_from_slice()` to update this buffer, to prevent the
    /// backing from being deallocated and reallocated.
    pub accessed: Vec<usize>,

    /// Maximum size to allow inputs to expand to
    pub max_input_size: usize,

    /// The random number generator used for mutations
    pub rng: Rng,

    /// The mutations should prefer creating ASCII-printable characters
    pub printable: bool,
}

impl Mutator {
    pub fn new() -> Self {
        Mutator {
            input:          Vec::new(),
            accessed:       Vec::new(),
            max_input_size: 1024,
            printable:      false,
            rng: Rng {
                seed:         0x12640367f4b7ea35,
                exp_disabled: false,
            },
        }
    }
        /// Set whether or not this mutator should produce only ASCII-printable
    /// characters.
    ///
    /// If non-printable characters are used in part of the corpus or existing
    /// input, they may be inherited and still exist in the output of the
    /// fuzzer.
    pub fn printable(mut self, printable: bool) -> Self {
        self.printable = printable;
        self
    }

    /// Allows enabling and disabling of exponential random in the fuzzer. If
    /// disabled, all random selections will be uniform.
    pub fn rand_exp(mut self, exponential_random: bool) -> Self {
        self.rng.exp_disabled = !exponential_random;
        self
    }

    /// Sets the seed for the internal RNG
    pub fn seed(mut self, seed: u64) -> Self {
        self.rng.seed = seed ^ 0x12640367f4b7ea35;
        self
    }

    /// Sets the maximum input size
    pub fn max_input_size(mut self, size: usize) -> Self {
        self.max_input_size = size;
        self
    }

        /// Pick a random offset in the input to corrupt. Any mutation
    /// strategy which needs to pick a random byte should us this, such that
    /// a bias can be applied to the offsets and automatically affect all
    /// aspects of the fuzzer
    ///
    /// If `insert` is set to `true`, then the offset being returned is
    /// expected to be for insertion. This means that we'll have a chance of
    /// returning an offset before or after sections of the input. For example,
    /// we may return an index which is `self.input.len()`, which would allow
    /// for the user to insert data at the end of the input.
    ///
    /// If the input is zero length, then this will return a zero. Thus, it
    /// is up to the caller to know whether or not the input size being zero
    /// matters. For example, flipping a byte at offset 0 on a 0 size input
    /// would cause a panic, but inserting at offset 0 may be desired.
    pub fn rand_offset_int(&mut self, plus_one: bool) -> usize {
        if !self.accessed.is_empty() {
            // If we have an accessed list, use an index from the list of known
            // accessed bytes from the input. This prevents us from corrupting
            // a byte which cannot possibly influence the binary, as it is
            // never read during the fuzz case.
            self.accessed[self.rng.rand_exp(0, self.accessed.len() - 1)]
        } else if !self.input.is_empty() {
            // We have no accessed list, just return a random index
            self.rng
                .rand_exp(0, self.input.len() - (!plus_one) as usize)
        } else {
            // Input is entirely empty, just return index 0 such that
            // things that insert into the input know that they should
            // just insert at 0.
            0
        }
    }

    /// Generate a random offset, see `rand_offset_int` for more info
    pub fn rand_offset(&mut self) -> usize {
        self.rand_offset_int(false)
    }
}

/// A byte corruption skeleton which has user-supplied corruption logic which
/// will be used to mutate a byte which is passed to it
///
/// $corrupt takes a &mut self, `u8` as arguments, and returns a `u8` as the
/// corrupted value.
#[macro_export]
macro_rules! byte_corruptor {
    ($func:ident, $corrupt:expr) => {
        /// Corrupt a byte in the input
        fn $func(&mut self) {
            // Only corrupt a byte if there are bytes present
            if !self.mutator.input.is_empty() {
                // Pick a random byte offset
                let offset = self.mutator.rand_offset();

                // Perform the corruption
                self.mutator.input[offset] = ($corrupt)(self, self.mutator.input[offset].clone()).into();
            }
        }
    };
}
