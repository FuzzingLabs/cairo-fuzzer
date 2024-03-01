fuzzinglabs:
	cargo run --release -- --cores 11 --contract ./tests1.0/fuzzinglabs.json --casm ./tests1.0/fuzzinglabs.casm --target-function "Fuzz_symbolic_execution"

teststorage:
	cargo run --release -- --cores 1 --contract ./tests1.0/teststorage.json --casm ./tests1.0/teststorage.casm --target-function "storage_test"
