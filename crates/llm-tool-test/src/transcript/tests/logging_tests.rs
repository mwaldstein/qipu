use super::super::writer::TranscriptWriter;
use std::fs;

#[test]
fn test_log_spawn_basic() {
    let dir = tempfile::tempdir().unwrap();
    let writer = TranscriptWriter::new(dir.path().to_path_buf(), dir.path().to_path_buf()).unwrap();

    let result = writer.log_spawn("qipu", &["init".to_string()]);
    assert!(result.is_ok());

    let events = writer.read_events().unwrap();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert_eq!(event["event"], "spawn");
    assert_eq!(event["command"], "qipu");
    assert!(event["ts"].is_number());
    assert!(event["args"].is_array());
    let args: Vec<String> = serde_json::from_value(event["args"].clone()).unwrap();
    assert_eq!(args, vec!["init"]);
}

#[test]
fn test_log_spawn_with_multiple_args() {
    let dir = tempfile::tempdir().unwrap();
    let writer = TranscriptWriter::new(dir.path().to_path_buf(), dir.path().to_path_buf()).unwrap();

    let args = vec![
        "--title".to_string(),
        "Test Note".to_string(),
        "--type".to_string(),
        "permanent".to_string(),
    ];
    writer.log_spawn("qipu", &args).unwrap();

    let events = writer.read_events().unwrap();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert_eq!(event["event"], "spawn");
    assert_eq!(event["command"], "qipu");
    let parsed_args: Vec<String> = serde_json::from_value(event["args"].clone()).unwrap();
    assert_eq!(parsed_args.len(), 4);
    assert_eq!(parsed_args[0], "--title");
    assert_eq!(parsed_args[1], "Test Note");
    assert_eq!(parsed_args[2], "--type");
    assert_eq!(parsed_args[3], "permanent");
}

#[test]
fn test_log_spawn_empty_args() {
    let dir = tempfile::tempdir().unwrap();
    let writer = TranscriptWriter::new(dir.path().to_path_buf(), dir.path().to_path_buf()).unwrap();

    writer.log_spawn("qipu", &[]).unwrap();

    let events = writer.read_events().unwrap();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert_eq!(event["event"], "spawn");
    assert_eq!(event["command"], "qipu");
    let args: Vec<String> = serde_json::from_value(event["args"].clone()).unwrap();
    assert!(args.is_empty());
}

#[test]
fn test_log_spawn_creates_events_file() {
    let dir = tempfile::tempdir().unwrap();
    let writer = TranscriptWriter::new(dir.path().to_path_buf(), dir.path().to_path_buf()).unwrap();

    writer.log_spawn("test", &["arg1".to_string()]).unwrap();

    let events_path = dir.path().join("events.jsonl");
    assert!(events_path.exists());

    let content = fs::read_to_string(&events_path).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 1);

    let event: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(event["event"], "spawn");
    assert_eq!(event["command"], "test");
}

#[test]
fn test_log_output_basic() {
    let dir = tempfile::tempdir().unwrap();
    let writer = TranscriptWriter::new(dir.path().to_path_buf(), dir.path().to_path_buf()).unwrap();

    let result = writer.log_output("Some output text\n");
    assert!(result.is_ok());

    let events = writer.read_events().unwrap();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert_eq!(event["event"], "output");
    assert_eq!(event["text"], "Some output text\n");
    assert!(event["ts"].is_number());
}

#[test]
fn test_log_output_multiline() {
    let dir = tempfile::tempdir().unwrap();
    let writer = TranscriptWriter::new(dir.path().to_path_buf(), dir.path().to_path_buf()).unwrap();

    let text = "Line 1\nLine 2\nLine 3\n";
    writer.log_output(text).unwrap();

    let events = writer.read_events().unwrap();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert_eq!(event["text"], text);
}

#[test]
fn test_log_output_empty() {
    let dir = tempfile::tempdir().unwrap();
    let writer = TranscriptWriter::new(dir.path().to_path_buf(), dir.path().to_path_buf()).unwrap();

    writer.log_output("").unwrap();

    let events = writer.read_events().unwrap();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert_eq!(event["event"], "output");
    assert_eq!(event["text"], "");
}

#[test]
fn test_log_output_with_special_chars() {
    let dir = tempfile::tempdir().unwrap();
    let writer = TranscriptWriter::new(dir.path().to_path_buf(), dir.path().to_path_buf()).unwrap();

    let text = "Output with special chars: \t\n\r\"'\\";
    writer.log_output(text).unwrap();

    let events = writer.read_events().unwrap();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    let parsed_text: String = event["text"].as_str().unwrap().to_string();
    assert_eq!(parsed_text, text);
}

#[test]
fn test_log_complete_basic() {
    let dir = tempfile::tempdir().unwrap();
    let writer = TranscriptWriter::new(dir.path().to_path_buf(), dir.path().to_path_buf()).unwrap();

    let result = writer.log_complete(0, 45.5);
    assert!(result.is_ok());

    let events = writer.read_events().unwrap();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert_eq!(event["event"], "complete");
    assert_eq!(event["exit_code"], 0);
    assert!((event["duration_secs"].as_f64().unwrap() - 45.5).abs() < 0.01);
    assert!(event["ts"].is_number());
}

#[test]
fn test_log_complete_nonzero_exit() {
    let dir = tempfile::tempdir().unwrap();
    let writer = TranscriptWriter::new(dir.path().to_path_buf(), dir.path().to_path_buf()).unwrap();

    writer.log_complete(1, 30.0).unwrap();

    let events = writer.read_events().unwrap();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert_eq!(event["exit_code"], 1);
    assert!((event["duration_secs"].as_f64().unwrap() - 30.0).abs() < 0.01);
}

#[test]
fn test_log_complete_negative_exit() {
    let dir = tempfile::tempdir().unwrap();
    let writer = TranscriptWriter::new(dir.path().to_path_buf(), dir.path().to_path_buf()).unwrap();

    writer.log_complete(-1, 0.5).unwrap();

    let events = writer.read_events().unwrap();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    assert_eq!(event["exit_code"], -1);
}

#[test]
fn test_log_complete_fractional_duration() {
    let dir = tempfile::tempdir().unwrap();
    let writer = TranscriptWriter::new(dir.path().to_path_buf(), dir.path().to_path_buf()).unwrap();

    writer.log_complete(0, 123.456).unwrap();

    let events = writer.read_events().unwrap();
    assert_eq!(events.len(), 1);

    let event = &events[0];
    let duration = event["duration_secs"].as_f64().unwrap();
    assert!((duration - 123.456).abs() < 0.001);
}

#[test]
fn test_log_multiple_events() {
    let dir = tempfile::tempdir().unwrap();
    let writer = TranscriptWriter::new(dir.path().to_path_buf(), dir.path().to_path_buf()).unwrap();

    writer.log_spawn("qipu", &["init".to_string()]).unwrap();
    writer.log_output("Initializing store...\n").unwrap();
    writer.log_complete(0, 1.5).unwrap();

    let events = writer.read_events().unwrap();
    assert_eq!(events.len(), 3);

    assert_eq!(events[0]["event"], "spawn");
    assert_eq!(events[1]["event"], "output");
    assert_eq!(events[2]["event"], "complete");
}

#[test]
fn test_log_events_append() {
    let dir = tempfile::tempdir().unwrap();
    let writer = TranscriptWriter::new(dir.path().to_path_buf(), dir.path().to_path_buf()).unwrap();

    writer.log_spawn("cmd1", &[]).unwrap();
    writer.log_output("output1").unwrap();

    let events = writer.read_events().unwrap();
    assert_eq!(events.len(), 2);

    writer.log_output("output2").unwrap();
    writer.log_complete(0, 10.0).unwrap();

    let events = writer.read_events().unwrap();
    assert_eq!(events.len(), 4);
    assert_eq!(events[2]["text"], "output2");
    assert_eq!(events[3]["event"], "complete");
}

#[test]
fn test_log_spawn_timestamp_increasing() {
    let dir = tempfile::tempdir().unwrap();
    let writer = TranscriptWriter::new(dir.path().to_path_buf(), dir.path().to_path_buf()).unwrap();

    writer.log_spawn("cmd1", &[]).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));
    writer.log_spawn("cmd2", &[]).unwrap();

    let events = writer.read_events().unwrap();
    assert_eq!(events.len(), 2);

    let ts1 = events[0]["ts"].as_f64().unwrap();
    let ts2 = events[1]["ts"].as_f64().unwrap();
    assert!(ts2 > ts1);
}
