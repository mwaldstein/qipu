use super::super::analyzer::TranscriptAnalyzer;

#[test]
fn test_analyze_empty_transcript() {
    let transcript = "";
    let metrics = TranscriptAnalyzer::analyze(transcript);

    assert_eq!(metrics.total_commands, 0);
    assert_eq!(metrics.unique_commands, 0);
    assert_eq!(metrics.error_count, 0);
    assert_eq!(metrics.retry_count, 0);
    assert_eq!(metrics.help_invocations, 0);
    assert_eq!(metrics.first_try_success_rate, 0.0);
    assert_eq!(metrics.iteration_ratio, 0.0);
}

#[test]
fn test_analyze_single_command() {
    let transcript = "qipu create --title 'Test Note'";
    let metrics = TranscriptAnalyzer::analyze(transcript);

    assert_eq!(metrics.total_commands, 1);
    assert_eq!(metrics.unique_commands, 1);
    assert_eq!(metrics.error_count, 0);
    assert_eq!(metrics.retry_count, 0);
    assert_eq!(metrics.first_try_success_rate, 1.0);
    assert_eq!(metrics.iteration_ratio, 1.0);
}

#[test]
fn test_analyze_multiple_commands() {
    let transcript = "qipu create --title 'Test 1'\nqipu create --title 'Test 2'\nqipu list";
    let metrics = TranscriptAnalyzer::analyze(transcript);

    assert_eq!(metrics.total_commands, 3);
    assert_eq!(metrics.unique_commands, 2);
    assert_eq!(metrics.retry_count, 1);
}

#[test]
fn test_analyze_with_errors() {
    let transcript =
        "qipu create --title 'Test 1'\nError: command failed\nqipu create --title 'Test 1'";
    let metrics = TranscriptAnalyzer::analyze(transcript);

    assert_eq!(metrics.total_commands, 2);
    assert_eq!(metrics.error_count, 1);
}

#[test]
fn test_analyze_help_invocations() {
    let transcript = "qipu --help\nqipu create --title 'Test'\nqipu list --help";
    let metrics = TranscriptAnalyzer::analyze(transcript);

    assert_eq!(metrics.total_commands, 3);
    assert_eq!(metrics.help_invocations, 2);
}

#[test]
fn test_iteration_ratio() {
    let transcript = "qipu create\nqipu create\nqipu create\nqipu list\nqipu list";
    let metrics = TranscriptAnalyzer::analyze(transcript);

    assert_eq!(metrics.total_commands, 5);
    assert_eq!(metrics.unique_commands, 2);
    assert_eq!(metrics.retry_count, 3);
    assert_eq!(metrics.iteration_ratio, 2.5);
}

#[test]
fn test_extract_commands_basic() {
    let transcript = "qipu create --title 'Test'\nqipu list\nqipu link --from a --to b";
    let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].command, "create");
    assert_eq!(commands[0].exit_code, Some(0));
    assert_eq!(commands[1].command, "list");
    assert_eq!(commands[1].exit_code, Some(0));
    assert_eq!(commands[2].command, "link");
    assert_eq!(commands[2].exit_code, Some(0));
}

#[test]
fn test_extract_commands_with_explicit_exit_code() {
    let transcript = "qipu create --title 'Test'\nExit Code: 0\nqipu invalid\nExit status: 1";
    let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0].command, "create");
    assert_eq!(commands[0].exit_code, Some(0));
    assert_eq!(commands[1].command, "invalid");
    assert_eq!(commands[1].exit_code, Some(1));
}

#[test]
fn test_extract_commands_with_implicit_error() {
    let transcript =
        "qipu create --title 'Test'\nError: something failed\nqipu create --title 'Test'";
    let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0].command, "create");
    assert_eq!(commands[0].exit_code, Some(1));
    assert_eq!(commands[1].command, "create");
    assert_eq!(commands[1].exit_code, Some(0));
}

#[test]
fn test_extract_commands_help_detection() {
    let transcript = "qipu --help\nqipu create --help\nqipu list";
    let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].command, "help");
    assert_eq!(commands[0].exit_code, Some(0));
    assert_eq!(commands[1].command, "help");
    assert_eq!(commands[1].exit_code, Some(0));
    assert_eq!(commands[2].command, "list");
    assert_eq!(commands[2].exit_code, Some(0));
}

#[test]
fn test_extract_commands_various_exit_code_formats() {
    let transcript =
        "qipu create\nexit code: 0\nqipu delete\nExit Status: 127\nqipu search\nexit code 255";
    let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].exit_code, Some(0));
    assert_eq!(commands[1].exit_code, Some(127));
    assert_eq!(commands[2].exit_code, Some(255));
}

#[test]
fn test_extract_commands_empty_transcript() {
    let transcript = "";
    let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

    assert_eq!(commands.len(), 0);
}

#[test]
fn test_extract_commands_no_matching_commands() {
    let transcript = "Some random text\nWithout commands\nJust output";
    let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

    assert_eq!(commands.len(), 0);
}

#[test]
fn test_extract_commands_mixed_with_output() {
    let transcript = "Starting session...\nqipu create --title 'Test'\nNote created successfully\nqipu list\nList output\nDone";
    let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0].command, "create");
    assert_eq!(commands[1].command, "list");
}

#[test]
fn test_extract_commands_case_insensitive_exit() {
    let transcript =
        "qipu create\nEXIT CODE: 0\nqipu delete\nexit code: 1\nqipu search\nExit Code: 2";
    let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].exit_code, Some(0));
    assert_eq!(commands[1].exit_code, Some(1));
    assert_eq!(commands[2].exit_code, Some(2));
}

#[test]
fn test_extract_commands_with_multiple_errors_keywords() {
    let transcript =
        "qipu create\nERROR: invalid input\nqipu delete\nFailed: not found\nqipu search";
    let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].exit_code, Some(1));
    assert_eq!(commands[1].exit_code, Some(1));
    assert_eq!(commands[2].exit_code, Some(0));
}

#[test]
fn test_extract_commands_nonzero_exit_code() {
    let transcript = "qipu create\nExit code: 130";
    let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command, "create");
    assert_eq!(commands[0].exit_code, Some(130));
}

#[test]
fn test_extract_commands_large_exit_code() {
    let transcript = "qipu create\nExit code: 255";
    let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command, "create");
    assert_eq!(commands[0].exit_code, Some(255));
}

#[test]
fn test_extract_commands_exit_code_takes_precedence() {
    let transcript = "qipu create\nExit code: 0\nqipu delete\nError: failed\nExit code: 1";
    let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0].exit_code, Some(0));
    assert_eq!(commands[1].exit_code, Some(1));
}

#[test]
fn test_extract_commands_subcommand_with_flags() {
    let transcript = "qipu create --title 'Test' --tag work\nqipu list --format json\nqipu link --from a --to b --type reference";
    let commands = TranscriptAnalyzer::extract_commands_with_exit_codes(transcript);

    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].command, "create");
    assert_eq!(commands[1].command, "list");
    assert_eq!(commands[2].command, "link");
}
