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
use crate::fuzzer::error::Error;
use std::{collections::HashMap, sync::Arc};

use crate::fuzzer::coverage::{self, Coverage, CoverageData};
use crate::mutator::types::Type;

use super::runner::Runner;

#[derive(Clone, Debug)]
pub struct RunnerStarknet {
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
    pub fn new(contract_class: &CasmContractClass, func_entrypoint_idx: usize) -> Self {
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

impl Runner for RunnerStarknet {
    fn execute(&mut self, inputs: Vec<Type>) -> Result<std::option::Option<Coverage>, (Coverage, Error)> {
        // Create an execution entry point
        let calldata: Vec<Felt252> = inputs.to_vec().iter().map(|x| match x {
            Type::Felt252(x) => x.clone(),
            _ => Felt252::zero(),
        }).collect();
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
        eprintln!("inputs {:?}", inputs);
        eprintln!("call data {:?}", calldata);
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
                    .expect("Could not get call info").trace;
                let coverage = Coverage { inputs: inputs, data: call_info.into_iter().map(|(a, b)| CoverageData {pc_ap : (a, b) }).collect() };
                return Ok(Some(coverage));
            }
            Err(e) =>{
                let fuzz_error = Error::Abort { message: e.to_string() };
                let coverage = Coverage { inputs: inputs, data: Vec::new() };
             return Err((coverage, fuzz_error));
             }
        };
    }

    fn get_target_parameters(&self) -> Vec<crate::mutator::types::Type> {
        return vec![Type::Vector(Box::new(Type::Felt252(Felt252::zero())), vec![Type::Felt252(Felt252::zero())])];
    }

    fn get_target_module(&self) -> String {
        todo!()
    }

    fn get_target_function(&self) -> String {
        todo!()
    }

    fn get_max_coverage(&self) -> usize {
        return 10000;
    }
}
