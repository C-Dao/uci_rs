pub struct UciOption{
    name: String,
    values: Vec<String>,
    opt_type: UciOptionType
}

pub enum UciOptionType{
    TypeOption,
    TypeList,
}