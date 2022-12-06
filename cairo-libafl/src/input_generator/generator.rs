use libafl::generators::Generator;
use libafl::prelude::BytesInput;
use libafl::{bolts::rands::Rand, state::HasRand, Error};
use std::marker::PhantomData;

#[derive(Clone, Debug)]
/// Generates random printable characters
pub struct MyRandPrintablesGenerator<S>
where
    S: HasRand,
{
    size: usize,
    phantom: PhantomData<S>,
}

impl<S> Generator<BytesInput, S> for MyRandPrintablesGenerator<S>
where
    S: HasRand,
{
    fn generate(&mut self, state: &mut S) -> Result<BytesInput, Error> {
        let size = self.size;
        let printables = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz \t\n!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~".as_bytes();
        let random_bytes: Vec<u8> = (0..size)
            .map(|_| *state.rand_mut().choose(printables))
            .collect();
        Ok(BytesInput::new(random_bytes))
    }

    /// Generates up to `DUMMy_BYTES_MAX` non-random dumMy bytes (0)
    fn generate_dummy(&self, _state: &mut S) -> BytesInput {
        let size = 11;
        BytesInput::new(vec![0_u8; size])
    }
}

impl<S> MyRandPrintablesGenerator<S>
where
    S: HasRand,
{
    /// Creates a new [`RandPrintablesGenerator`], generating up to `max_size` random printable characters.
    #[must_use]
    pub fn new(size: usize) -> Self {
        Self {
            size,
            phantom: PhantomData,
        }
    }
}
