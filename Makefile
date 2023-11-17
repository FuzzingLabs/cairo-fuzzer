fuzzinglabs:
	cargo run --release -- --cores 10 --contract ./tests1.0/fuzzinglabs.json --casm ./tests1.0/fuzzinglabs.casm --function "Fuzz_symbolic_execution"

teststorage:
	cargo run --release -- --cores 1 --contract ./tests1.0/teststorage.json --casm ./tests1.0/teststorage.casm --function "storage_test"