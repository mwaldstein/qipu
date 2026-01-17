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

    /// The original source of the information
    #[arg(long)]
    pub source: Option<String>,

    /// Name of the human or agent who created the note
    #[arg(long)]
    pub author: Option<String>,

    /// Name of the LLM model used to generate the content
    #[arg(long)]
    pub generated_by: Option<String>,

    /// Hash or ID of the prompt used to generate the content
    #[arg(long)]
    pub prompt_hash: Option<String>,

    /// Flag indicating if a human has manually reviewed the content
    #[arg(long)]
    pub verified: Option<bool>,
}
