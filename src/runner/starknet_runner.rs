use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use felt::Felt252;
use num_traits::Zero;
use starknet_rs::definitions::block_context::BlockContext;
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

#[derive(Clone)]
pub struct RunnerStarknet {
    contract_class: CasmContractClass,
}

impl RunnerStarknet {
    pub fn new(contract_class: &CasmContractClass) -> Self {
        return RunnerStarknet {
            contract_class: contract_class.clone(),
        };
    }
}

impl Runner for RunnerStarknet {
    fn runner(
        self,
        func_entrypoint_idx: usize,
        data: &Vec<Felt252>,
    ) -> Result<Option<Vec<(u32, u32)>>, String> {
        let contract_class: CasmContractClass = self.contract_class;
        let entrypoints = contract_class.clone().entry_points_by_type;
        let entrypoint_selector = &entrypoints
            .external
            .get(func_entrypoint_idx)
            .unwrap()
            .selector;

        // Create state reader with class hash data
        let mut contract_class_cache = HashMap::new();

        let address = Address(1111.into());
        let class_hash: ClassHash = [1; 32];
        let nonce = Felt252::zero();

        contract_class_cache.insert(class_hash, contract_class);
        let mut state_reader = InMemoryStateReader::default();
        state_reader
            .address_to_class_hash_mut()
            .insert(address.clone(), class_hash);
        state_reader
            .address_to_nonce_mut()
            .insert(address.clone(), nonce);

        // Create state from the state_reader and contract cache.
        let mut state = CachedState::new(Arc::new(state_reader), None, Some(contract_class_cache));

        // Create an execution entry point
        let calldata = data.to_vec();
        let caller_address = Address(0000.into());
        let entry_point_type = EntryPointType::External;

        let exec_entry_point = ExecutionEntryPoint::new(
            address,
            calldata.clone(),
            Felt252::new(entrypoint_selector.clone()),
            caller_address,
            entry_point_type,
            Some(CallType::Delegate),
            Some(class_hash),
            100000,
        );

        // Execute the entrypoint
        let block_context = BlockContext::default();
        let mut tx_execution_context = TransactionExecutionContext::new(
            Address(0.into()),
            Felt252::zero(),
            Vec::new(),
            0,
            10.into(),
            block_context.invoke_tx_max_n_steps(),
            TRANSACTION_VERSION.clone(),
        );
        let mut resources_manager = ExecutionResourcesManager::default();
        match exec_entry_point.execute(
            &mut state,
            &block_context,
            &mut resources_manager,
            &mut tx_execution_context,
            false,
            block_context.invoke_tx_max_n_steps(),
        ) {
            Ok(exec_info) => {
                return Ok(Some(
                    exec_info
                        .call_info
                        .expect("could not get call_info from starknet execution")
                        .trace,
                ));
            }
            Err(e) => return Err(e.to_string()),
        };
    }
}
