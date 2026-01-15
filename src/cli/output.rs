pub use crate::lib::format::OutputFormat;
use clap::ValueEnum;

// Implement ValueEnum for OutputFormat to work with clap
impl ValueEnum for OutputFormat {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            OutputFormat::Human,
            OutputFormat::Json,
            OutputFormat::Records,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            OutputFormat::Human => Some(clap::builder::PossibleValue::new("human")),
            OutputFormat::Json => Some(clap::builder::PossibleValue::new("json")),
            OutputFormat::Records => Some(clap::builder::PossibleValue::new("records")),
        }
    }
}
