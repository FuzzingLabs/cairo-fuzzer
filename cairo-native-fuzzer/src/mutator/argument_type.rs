use starknet_types_core::felt::Felt;

/// Enum representing the types of arguments that can be passed to a function
#[derive(Debug)]
pub enum ArgumentType {
    Felt,
    // TODO: Add support for other types
}

/// Helper function to map argument types based on their debug names
/// This function takes a debug name string and returns the corresponding `ArgumentType`
pub fn map_argument_type(debug_name: &str) -> Option<ArgumentType> {
    match debug_name {
        "felt252" => Some(ArgumentType::Felt),
        // TODO: Add support for other types
        _ => None,
    }
}
