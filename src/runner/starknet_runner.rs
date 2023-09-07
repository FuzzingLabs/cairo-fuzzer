use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use felt::Felt252;
use num_bigint::BigUint;
use num_traits::Zero;
use starknet_rs::definitions::block_context::BlockContext;
use starknet_rs::execution::CallInfo;
use starknet_rs::state::cached_state::CachedState;
use starknet_rs::EntryPointType;
use starknet_rs::{
    definitions::constants::TRANSACTION_VERSION,
    execution::{
        execution_entry_point::ExecutionEntryPoint, CallType, TransactionExecutionContext,
    },
    state::{in_memory_state_reader::InMemoryStateReader, ExecutionResourcesManager},
    utils::{Address, ClassHash},
};

use std::{collections::HashMap, sync::Arc};

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

        let address = Address(1111.into());
        let class_hash: ClassHash = [1; 32];
        let nonce = Felt252::zero();

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
        let caller_address = Address(0000.into());
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
        //println!("Runner setup : {:?}", runner);
        runner
    }
}

impl Runner for RunnerStarknet {
    fn runner(mut self, data: &Vec<Felt252>) -> Result<(Self, CallInfo), String> {
        // Create an execution entry point
        let calldata = data.to_vec();
        //println!("here {:?}", self.state);
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
                //println!("here 2{:?}", self.state);
                let call_info = exec_info
                    .call_info
                    .clone()
                    .expect("Could not get call info");
                return Ok((self, call_info));
            }
            Err(e) => return Err(e.to_string()),
        };
    }
}
