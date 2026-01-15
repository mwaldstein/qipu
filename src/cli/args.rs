use super::parse::parse_note_type;
use crate::lib::note::NoteType;
use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct CreateArgs {
    /// Note title
    pub title: String,

    /// Note type
    #[arg(long, short = 'T', value_parser = parse_note_type)]
    pub r#type: Option<NoteType>,

    /// Tags (can be specified multiple times)
    #[arg(long, short, action = clap::ArgAction::Append)]
    pub tag: Vec<String>,

    /// Open in editor after creation
    #[arg(long, short)]
    pub open: bool,
}
