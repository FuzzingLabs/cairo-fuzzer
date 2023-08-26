use felt::Felt252;

pub trait Runner {
    fn runner(
        self,
        func_name: usize,
        data: &Vec<Felt252>,
    ) -> Result<Option<Vec<(u32, u32)>>, String>;
}
