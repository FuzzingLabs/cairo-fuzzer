use cairo_rs::types::relocatable::Relocatable;
use felt::Felt252;
use num_traits::Zero;
use starknet_rs::{
    business_logic::{
        execution::{
            execution_entry_point::ExecutionEntryPoint,
            objects::{CallType, TransactionExecutionContext},
        },
        fact_state::{
            in_memory_state_reader::InMemoryStateReader, state::ExecutionResourcesManager,
        },
        state::cached_state::CachedState,
    },
    definitions::{constants::TRANSACTION_VERSION, general_config::StarknetGeneralConfig},
    services::api::contract_class::{ContractClass, EntryPointType},
    utils::{Address, ClassHash},
};
use std::collections::HashMap;

pub fn runner(
    contract_class: &ContractClass,
    func_entrypoint: &String,
    data: &Vec<Felt252>,
) -> Result<Option<Vec<Relocatable>>, String> {
    // ---------------------------------------------------------
    //  Create program and entry point types for contract class
    // ---------------------------------------------------------

    let entry_points_by_type = contract_class.entry_points_by_type();
    let entrypoint_selector = entry_points_by_type
        .get(&EntryPointType::External) // Should we call only "External" functions?
        .unwrap()
        .iter()
        .find(|entrypoint| &entrypoint.get_offset() == func_entrypoint)
        .unwrap()
        .selector()
        .clone();
    //* --------------------------------------------
    //*    Create state reader with class hash data
    //* --------------------------------------------

    let mut contract_class_cache = HashMap::new();

    //  ------------ contract data --------------------

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

    //* ---------------------------------------
    //*    Create state with previous data
    //* ---------------------------------------

    let mut state = CachedState::new(state_reader, Some(contract_class_cache));

    //* ------------------------------------
    //*    Create execution entry point
    //* ------------------------------------

    let mut calldata = [].to_vec();
    for i in data {
        calldata.push((*i).clone());
    }
    let caller_address = Address(0000.into());
    let entry_point_type = EntryPointType::External;
    let exec_entry_point = ExecutionEntryPoint::new(
        address,
        calldata.clone(),
        entrypoint_selector,
        caller_address,
        entry_point_type,
        Some(CallType::Delegate),
        Some(class_hash),
    );

    //* --------------------
    //*   Execute contract
    //* ---------------------
    let general_config = StarknetGeneralConfig::default();
    let tx_execution_context = TransactionExecutionContext::new(
        Address(0.into()),
        Felt252::zero(),
        Vec::new(),
        0,
        10.into(),
        general_config.invoke_tx_max_n_steps(),
        TRANSACTION_VERSION,
    );
    let mut resources_manager = ExecutionResourcesManager::default();
    match exec_entry_point.execute(
        &mut state,
        &general_config,
        &mut resources_manager,
        &tx_execution_context,
    ) {
        Ok(exec_info) => return Ok(Some(exec_info.trace)),
        Err(e) => return Err(e.to_string()),
    };
}
