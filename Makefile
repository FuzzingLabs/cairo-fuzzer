run:
	cargo run --release -- --cores 10 --contract ./tests1.0/fuzzinglabs.json --casm ./tests1.0/fuzzinglabs.casm --function "Fuzz_symbolic_execution"