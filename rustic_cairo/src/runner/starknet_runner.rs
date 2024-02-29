use crate::fuzzer::error::Error;
use crate::json_helper::json_parser::Function;
use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use felt::Felt252;
use num_bigint::BigUint;
use num_traits::Zero;
use starknet_rs::definitions::block_context::BlockContext;
use starknet_rs::execution::CallInfo;
use starknet_rs::state::cached_state::CachedState;
use starknet_rs::state::state_cache::StateCache;
use starknet_rs::EntryPointType;
use starknet_rs::{
    definitions::constants::TRANSACTION_VERSION,
    execution::{
        execution_entry_point::ExecutionEntryPoint, CallType, TransactionExecutionContext,
    },
    state::{in_memory_state_reader::InMemoryStateReader, ExecutionResourcesManager},
    utils::{Address, ClassHash},
};
use std::thread::sleep;
use std::{collections::HashMap, sync::Arc};

use crate::fuzzer::coverage::{self, Coverage /*  CoverageData */};
use crate::mutator::types::Type;

use super::runner::Runner;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct RunnerStarknet {
    target_function: Function,
    contract_class: CasmContractClass,
    func_entrypoint_idx: usize,
    entrypoint_selector: BigUint,
    address: Address,
    class_hash: ClassHash,
    state: CachedState<InMemoryStateReader>,
    caller_address: Address,
    entry_point_type: EntryPointType,
    tx_execution_context: TransactionExecutionContext,
    block_context: BlockContext,
    resources_manager: ExecutionResourcesManager,
}

impl RunnerStarknet {
    pub fn new(
        contract_class: &CasmContractClass,
        func_entrypoint_idx: usize,
        target_function: Function,
    ) -> Self {
        let entrypoints = contract_class.clone().entry_points_by_type;
        let entrypoint_selector = &entrypoints
            .external
            .get(func_entrypoint_idx)
            .unwrap()
            .selector;

        // Create state reader with class hash data
        let mut contract_class_cache: HashMap<[u8; 32], CasmContractClass> = HashMap::new();

        let address = Address(1111.into()); //todo - make it configurable from the config
        let class_hash: ClassHash = [1; 32];
        let nonce = Felt252::zero(); //todo - make it configurable from the config

        contract_class_cache.insert(class_hash, contract_class.clone());
        let mut state_reader = InMemoryStateReader::default();
        state_reader
            .address_to_class_hash_mut()
            .insert(address.clone(), class_hash);
        state_reader
            .address_to_nonce_mut()
            .insert(address.clone(), nonce);

        // Create state from the state_reader and contract cache.
        let state = CachedState::new(Arc::new(state_reader), None, Some(contract_class_cache));
        let caller_address = Address(0000.into()); //todo - make it configurable from the config
        let entry_point_type = EntryPointType::External;

        let block_context = BlockContext::default();
        let tx_execution_context = TransactionExecutionContext::new(
            Address(0.into()),
            Felt252::zero(),
            Vec::new(),
            0,
            10.into(),
            block_context.invoke_tx_max_n_steps(),
            TRANSACTION_VERSION.clone(),
        );
        let resources_manager = ExecutionResourcesManager::default();

        let runner = RunnerStarknet {
            target_function: target_function,
            contract_class: contract_class.clone(),
            func_entrypoint_idx: func_entrypoint_idx,
            entrypoint_selector: entrypoint_selector.clone(),
            address: address,
            class_hash: class_hash,
            state: state,
            caller_address: caller_address,
            entry_point_type: entry_point_type,
            tx_execution_context: tx_execution_context,
            block_context: block_context,
            resources_manager: resources_manager,
        };
        runner
    }
    #[allow(dead_code)]
    pub fn get_state(self) -> CachedState<InMemoryStateReader> {
        return self.state;
    }
    #[allow(dead_code)]
    pub fn set_state(mut self, state: StateCache) -> Self {
        self.state.cache = state;
        self
    }
}

pub fn convert_calldata(inputs: Vec<Type>) -> Vec<Felt252> {
    let mut res: Vec<Felt252> = vec![];
    for i in inputs {
        match i {
            Type::Felt252(value) => res.push(value),
            Type::U8(value) => res.push(Felt252::from(value)),
            Type::U16(value) => res.push(Felt252::from(value)),
            Type::U32(value) => res.push(Felt252::from(value)),
            Type::U64(value) => res.push(Felt252::from(value)),
            Type::U128(value) => res.push(Felt252::from(value)),
            Type::Bool(value) => res.push(Felt252::from(value)),
            Type::Vector(_, vec) => res.append(&mut convert_calldata(vec)),
        }
    }
    res
}

impl Runner for RunnerStarknet {
    fn get_contract_class(&self) -> CasmContractClass {
        self.contract_class.clone()
    }
    fn get_func_entrypoint_idx(&self) -> usize {
        self.func_entrypoint_idx
    }
    fn execute(
        &mut self,
        inputs: Vec<Type>,
    ) -> Result<std::option::Option<Coverage>, (Coverage, Error)> {
        // Create an execution entry point
        let calldata: Vec<Felt252> = convert_calldata(inputs.clone());
        if calldata.len() != 11 {
            return Err((
                Coverage {
                    failure: false,
                    inputs: inputs,
                    data: vec![],
                },
                Error::Abort {
                    message: "Invalid calldata".to_string(),
                },
            ));
        }
        let exec_entry_point = ExecutionEntryPoint::new(
            self.address.clone(),
            calldata.clone(),
            Felt252::new(self.entrypoint_selector.clone()),
            self.caller_address.clone(),
            self.entry_point_type,
            Some(CallType::Delegate),
            Some(self.class_hash),
            1000000,
        );
        // Execute the entrypoint
        match exec_entry_point.execute(
            &mut self.state,
            &self.block_context,
            &mut self.resources_manager,
            &mut self.tx_execution_context,
            false,
            self.block_context.invoke_tx_max_n_steps(),
        ) {
            Ok(exec_info) => {
                let call_info = exec_info
                    .call_info
                    .clone()
                    .expect("Could not get call info");
                let coverage = Coverage {
                    failure: call_info.clone().failure_flag,
                    inputs: inputs,
                    data: call_info.clone().trace,
                };
                //eprintln!("DEBUG INPUTS {:?}", calldata.clone());
                return Ok(Some(coverage));
            }
            Err(e) => {
                let fuzz_error = Error::Unknown {
                    message: e.to_string(),
                };

                let coverage = Coverage {
                    failure: false,
                    inputs: inputs,
                    data: vec![],
                };
                return Err((coverage, fuzz_error));
            }
        };
    }

    fn get_target_parameters(&self) -> Vec<crate::mutator::types::Type> {
        //eprintln!("Target function inputs {:?}", self.target_function.inputs.clone());
        self.target_function.inputs.clone()
    }

    fn get_target_module(&self) -> String {
        return "fuzzinglabs".to_string();
    }

    fn get_target_function(&self) -> String {
        return "fuzzinglabs".to_string();
    }

    fn get_max_coverage(&self) -> usize {
        return 10000;
    }
}
