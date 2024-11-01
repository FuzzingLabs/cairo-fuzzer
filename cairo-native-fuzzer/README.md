## Cairo Native Fuzzer

Cairo Native Fuzzer is a rewrite of the Cairo Fuzzer based on [Cairo native from Lambdaclass](https://github.com/lambdaclass/cairo_native) developed to enhance fuzzer execution speed.

### Roadmap 

### Step 1 : Create a basic fuzzer based on Cairo Native : 
- [x] Implement the Cairo Native runner
- [x] Implement the fuzzer based on Cairo Native runner
- [ ] Import existing Felt252 mutator from the cairo-fuzzer

### Step 2 : Integrate existing cairo-fuzzer features into Cairo Native fuzzer : 
- [ ] Multithreading
- [ ] Support config files
- [ ] Property testing

### Step 3 : Advanced features :
- [ ] Support `u8`, `u16`, `u32`, `u64`, `u128` and `u256` arguments