use cairo_rs::types::relocatable::Relocatable;
use felt::Felt;
use num_traits::Zero;
use starknet_rs::{
    business_logic::{
        execution::{
            execution_entry_point::ExecutionEntryPoint,
            objects::{CallType, TransactionExecutionContext},
        },
        fact_state::{
            contract_state::ContractState, in_memory_state_reader::InMemoryStateReader,
            state::ExecutionResourcesManager,
        },
        state::cached_state::CachedState,
    },
    definitions::{constants::TRANSACTION_VERSION, general_config::StarknetGeneralConfig},
    services::api::contract_class::{ContractClass, EntryPointType},
    starknet_storage::dict_storage::DictStorage,
    utils::Address,
};
use std::collections::HashMap;

pub fn runner(
    contract_class: &ContractClass,
    func_entrypoint: &String,
    data: &Vec<Felt>,
) -> Result<Option<Vec<(Relocatable, Relocatable)>>, String> {
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

    let ffc = DictStorage::new();
    let contract_class_storage = DictStorage::new();
    let mut contract_class_cache = HashMap::new();

    //  ------------ contract data --------------------

    let address = Address(1111.into()); // Do we really care about that ?
    let class_hash = [1; 32];
    let contract_state = ContractState::new(class_hash, 3.into(), HashMap::new()); // What is a contract state ?

    contract_class_cache.insert(class_hash, contract_class.clone());
    let mut state_reader = InMemoryStateReader::new(ffc, contract_class_storage);
    state_reader
        .contract_states_mut()
        .insert(address.clone(), contract_state);

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
        Felt::zero(),
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
