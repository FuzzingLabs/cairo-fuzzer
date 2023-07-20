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
        // ---------------------------------------------------------
        //  Create program and entry point types for contract class
        // ---------------------------------------------------------

        let entry_points_by_type = self.contract_class.entry_points_by_type();
        let entrypoint_selector = entry_points_by_type
            .get(&EntryPointType::External) // Should we call only "External" functions?
            .unwrap()
            .iter()
            .find(|entrypoint| &entrypoint.offset().to_string() == func_entrypoint)
            .unwrap()
            .selector()
            .clone();
        //* --------------------------------------------
        //*    Create state reader with class hash data
        //* --------------------------------------------

        let mut contract_class_cache = HashMap::new();

        //  ------------ contract data --------------------

        let address = Address(1111.into());
        let class_hash = [1; 32];

        contract_class_cache.insert(class_hash, self.contract_class);
        let mut state_reader = InMemoryStateReader::default();
        state_reader
            .address_to_class_hash_mut()
            .insert(address.clone(), class_hash);

        //* ---------------------------------------
        //*    Create state with previous data
        //* ---------------------------------------

        let mut state = CachedState::new(state_reader, Some(contract_class_cache), None);

        //* ------------------------------------
        //*    Create execution entry point
        //* ------------------------------------

        let mut calldata: Vec<Felt252> = [].to_vec();
        for i in data {
            calldata.push((*i).clone());
        }
        let caller_address = Address(0000.into());
        let entry_point_type = EntryPointType::External;
        let exec_entry_point = ExecutionEntryPoint::new(
            address.clone(),
            calldata.clone(),
            entrypoint_selector,
            caller_address,
            entry_point_type,
            Some(CallType::Delegate),
            Some(class_hash),
            0,
        );

        //* --------------------
        //*   Execute contract
        //* ---------------------
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
            true,
        ) {
            Ok(exec_info) => {
                return Ok(Some(exec_info.trace));
            }
            Err(e) => return Err(e.to_string()),
        };
    }
}
