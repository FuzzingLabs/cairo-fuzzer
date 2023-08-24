use cairo_rs::felt::Felt252;
use num_traits::Zero;
use starknet_contract_class::EntryPointType;
use starknet_rs::{
    definitions::{block_context::BlockContext, constants::TRANSACTION_VERSION},
    execution::{
        execution_entry_point::ExecutionEntryPoint, CallType, TransactionExecutionContext,
    },
    services::api::contract_classes::deprecated_contract_class::ContractClass,
    state::cached_state::CachedState,
    state::{in_memory_state_reader::InMemoryStateReader, ExecutionResourcesManager},
    utils::Address,
};
use std::collections::HashMap;

use super::runner::Runner;

#[derive(Clone)]
pub struct RunnerStarknet {
    contract_class: ContractClass,
}

impl RunnerStarknet {
    pub fn new(contract_class: &ContractClass) -> Self {
        return RunnerStarknet {
            contract_class: contract_class.clone(),
        };
    }
}

impl Runner for RunnerStarknet {
    fn runner(
        self,
        func_entrypoint: &String,
        data: &Vec<Felt252>,
    ) -> Result<Option<Vec<(u32, u32)>>, String> {
        let contract_class: CasmContractClass = serde_json::from_slice(program_data).unwrap();
        let entrypoints = contract_class.clone().entry_points_by_type;
        let fib_entrypoint_selector = &entrypoints.external.get(0).unwrap().selector;

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
        let calldata = [0.into(), 1.into(), 12.into()].to_vec();
        let caller_address = Address(0000.into());
        let entry_point_type = EntryPointType::External;

        let exec_entry_point = ExecutionEntryPoint::new(
            address,
            calldata.clone(),
            Felt252::new(fib_entrypoint_selector.clone()),
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

        // expected results
        let expected_call_info = CallInfo {
            caller_address: Address(0.into()),
            call_type: Some(CallType::Delegate),
            contract_address: Address(1111.into()),
            entry_point_selector: Some(Felt252::new(fib_entrypoint_selector)),
            entry_point_type: Some(EntryPointType::External),
            calldata,
            retdata: [144.into()].to_vec(),
            execution_resources: ExecutionResources {
                n_steps: 418,
                n_memory_holes: 0,
                builtin_instance_counter: HashMap::from([(
                    RANGE_CHECK_BUILTIN_NAME.to_string(),
                    15,
                )]),
            },
            class_hash: Some(class_hash),
            gas_consumed: 35220,
            ..Default::default()
        };
    }
}
