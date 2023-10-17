	    =========================================================================================================================

                 _______  _______ _________ _______  _______         _______           _______  _______  _______  _______ 
                (  ____ \(  ___  )\__   __/(  ____ )(  ___  )       (  ____ \|\     /|/ ___   )/ ___   )(  ____ \(  ____ )
                | (    \/| (   ) |   ) (   | (    )|| (   ) |       | (    \/| )   ( |\/   )  |\/   )  || (    \/| (    )|
                | |      | (___) |   | |   | (____)|| |   | | _____ | (__    | |   | |    /   )    /   )| (__    | (____)|
                | |      |  ___  |   | |   |     __)| |   | |(_____)|  __)   | |   | |   /   /    /   / |  __)   |     __)
                | |      | (   ) |   | |   | (\ (   | |   | |       | (      | |   | |  /   /    /   /  | (      | (\ (   
                | (____/\| )   ( |___) (___| ) \ \__| (___) |       | )      | (___) | /   (_/\ /   (_/\| (____/\| ) \ \__
                (_______/|/     \|\_______/|/   \__/(_______)       |/       (_______)(_______/(_______/(_______/|/   \__/

	    =========================================================================================================================

# Options:

```
Usage: cairo-fuzzer [OPTIONS]

Options:
      --cores <CORES>              Set the number of threads to run [default: 1]
      --contract <CONTRACT>        Set the path of the JSON artifact to load [default: ]
      --casm <CASM>                Set the path of the JSON CASM artifact to load [default: ]
      --function <FUNCTION>        Set the function to fuzz [default: ]
      --workspace <WORKSPACE>      Workspace of the fuzzer [default: fuzzer_workspace]
      --inputfolder <INPUTFOLDER>  Path to the inputs folder to load [default: ]
      --crashfolder <CRASHFOLDER>  Path to the crashes folder to load [default: ]
      --inputfile <INPUTFILE>      Path to the inputs file to load [default: ]
      --crashfile <CRASHFILE>      Path to the crashes file to load [default: ]
      --dict <DICT>                Path to the dictionnary file to load [default: ]
      --logs                       Enable fuzzer logs in file
      --seed <SEED>                Set a custom seed (only applicable for 1 core run)
      --run-time <RUN_TIME>        Number of seconds this fuzzing session will last
      --config <CONFIG>            Load config file
      --replay                     Replay the corpus folder
      --minimizer                  Minimize Corpora
      --proptesting                Property Testing
      --analyze                    Dump functions prototypes
      --iter <ITER>                Iteration Number [default: -1]
  -h, --help                       Print help
```

## Fuzzing function of a contract:
```sh
cargo run --release -- --cores 13 --contract ./tests1.0/fuzzinglabs.json --casm ./tests1.0/fuzzinglabs.casm --function "fuzzinglabs_starknet"
```

## Fuzzing function of a contract with a number of iteration max:
```sh
cargo run --release -- --cores 13 --contract ./tests1.0/fuzzinglabs.json --casm ./tests1.0/fuzzinglabs.casm --function "fuzzinglabs_starknet" --iter 100000
```

## Load old corpus:
```sh
cargo run --release -- --cores 13 --contract ./tests1.0/fuzzinglabs.json --casm ./tests1.0/fuzzinglabs.casm --function "fuzzinglabs_starknet" --inputfile "tests1.0/fuzzinglabs_starknet_2023-04-04--12:38:47.json"
```

## Fuzzing using a config file:
Example of config file:
```json
{
    "cores": 1,
    "logs": false,
    "replay": false,
    "minimizer": false,
    "contract_file": "tests1.0/fuzzinglabs.json",
    "casm_file": "tests1.0/fuzzinglabs.casm",
    "function_name": "Fuzz_symbolic_execution",
    "input_file": "",
    "crash_file": "",
    "input_folder": "",
    "crash_folder": "",
    "workspace": "fuzzer_workspace",
    "proptesting": false,
    "iter": -1,
    "dict": "tests1.0/dict"
}
```

```sh
cargo run --release -- --config tests/config.json 
```

## Replay corpus folder:
```sh
cargo run --release -- --cores 13 --contract ./tests1.0/fuzzinglabs.json --casm ./tests1.0/fuzzinglabs.casm  --function "fuzzinglabs_starknet" --replay --inputfolder fuzzer_workspace/fuzzinglabs_starknet/inputs
```

## Fuzzing property testing:
Function should start with `Fuzz_`
```rust
func Fuzz_symbolic_execution()
```

```sh
cargo run --release -- --cores 13 --contract ./tests1.0/fuzzinglabs.json --casm ./tests1.0/fuzzinglabs.casm  --proptesting --iter 500000
```

## Fuzzing with a dictionnary:

Dictionnary format is the same as other fuzzers such as Honggfuzz or libafl
```python
key1=999999999999
key2=888888888888
key3=777777777777
...
key9=111111111111
```

```sh
cargo run --release -- --cores 13 --contract ./tests1.0/fuzzinglabs.json --casm ./tests1.0/fuzzinglabs.casm  --function "Fuzz_symbolic_execution" --dict tests/dict
```
