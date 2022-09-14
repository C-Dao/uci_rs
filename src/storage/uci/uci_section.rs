use super::uci_option::UciOption;

pub struct UciSection{
    name: String,
    sec_type: String,
    options: Vec<UciOption>,
}