use crate::transcript::redact::redact_sensitive;
use crate::transcript::types::{EvaluationReport, RunMetadata, RunReport};
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub struct TranscriptWriter {
    pub base_dir: PathBuf,
    pub results_dir: PathBuf,
}

impl TranscriptWriter {
    pub fn new(artifacts_dir: PathBuf, results_dir: PathBuf) -> anyhow::Result<Self> {
        if !artifacts_dir.exists() {
            fs::create_dir_all(&artifacts_dir)?;
        }
        if !results_dir.exists() {
            fs::create_dir_all(&results_dir)?;
        }
        Ok(Self {
            base_dir: artifacts_dir,
            results_dir,
        })
    }

    pub fn write_raw(&self, content: &str) -> anyhow::Result<()> {
        fs::write(self.base_dir.join("transcript.raw.txt"), content)?;
        // Also generate human-readable version from the content
        self.generate_human_transcript(content)?;
        Ok(())
    }

    fn generate_human_transcript(&self, raw_content: &str) -> anyhow::Result<()> {
        let mut human_lines = Vec::new();

        for line in raw_content.lines() {
            if let Ok(event) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(event_type) = event.get("type").and_then(|v| v.as_str()) {
                    match event_type {
                        "step_start" => {
                            human_lines.push("---".to_string());
                            human_lines.push("NEW TURN".to_string());
                            human_lines.push("---".to_string());
                        }
                        "text" => {
                            if let Some(text) = event
                                .get("part")
                                .and_then(|p| p.get("text"))
                                .and_then(|t| t.as_str())
                            {
                                human_lines.push(text.to_string());
                                human_lines.push(String::new()); // blank line
                            }
                        }
                        "step_finish" => {
                            human_lines.push("---".to_string());
                        }
                        _ => {}
                    }
                }
            }
        }

        let human_content = human_lines.join("\n");
        fs::write(self.base_dir.join("transcript.human.txt"), human_content)?;
        Ok(())
    }

    pub fn append_event(&self, event: &serde_json::Value) -> anyhow::Result<()> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.base_dir.join("events.jsonl"))?;
        writeln!(file, "{}", serde_json::to_string(event)?)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn log_spawn(&self, command: &str, args: &[String]) -> anyhow::Result<()> {
        let event = json!({
            "ts": Self::timestamp(),
            "event": "spawn",
            "command": command,
            "args": args,
        });
        self.append_event(&event)
    }

    #[allow(dead_code)]
    pub fn log_output(&self, text: &str) -> anyhow::Result<()> {
        let event = json!({
            "ts": Self::timestamp(),
            "event": "output",
            "text": text,
        });
        self.append_event(&event)
    }

    #[allow(dead_code)]
    pub fn log_complete(&self, exit_code: i32, duration_secs: f64) -> anyhow::Result<()> {
        let event = json!({
            "ts": Self::timestamp(),
            "event": "complete",
            "exit_code": exit_code,
            "duration_secs": duration_secs,
        });
        self.append_event(&event)
    }

    #[allow(dead_code)]
    fn timestamp() -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
    }

    #[allow(dead_code)]
    pub fn read_events(&self) -> anyhow::Result<Vec<serde_json::Value>> {
        let path = self.base_dir.join("events.jsonl");
        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&path)?;
        let mut events = Vec::new();
        for line in content.lines() {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
                events.push(value);
            }
        }
        Ok(events)
    }

    pub fn write_run_metadata(&self, metadata: &RunMetadata) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(metadata)?;
        fs::write(self.base_dir.join("run.json"), json)?;
        Ok(())
    }

    pub fn create_store_snapshot(&self, work_dir: &std::path::Path) -> anyhow::Result<()> {
        let snapshot_dir = self.base_dir.join("store_snapshot");
        fs::create_dir_all(&snapshot_dir)?;

        let qipu_dir = work_dir.join(".qipu");

        if qipu_dir.exists() {
            let snapshot_qipu_dir = snapshot_dir.join(".qipu");
            self.copy_dir(&qipu_dir, &snapshot_qipu_dir)?;
        }

        let output = std::process::Command::new("qipu")
            .arg("dump")
            .arg("--format")
            .arg("json")
            .current_dir(work_dir)
            .output();

        match output {
            Ok(result) if result.status.success() => {
                fs::write(snapshot_dir.join("export.json"), result.stdout)?;
                Ok(())
            }
            Ok(result) => {
                eprintln!(
                    "Warning: Failed to create store snapshot: {}",
                    String::from_utf8_lossy(&result.stderr)
                );
                Ok(())
            }
            Err(e) => {
                eprintln!("Warning: Failed to run qipu dump: {}", e);
                Ok(())
            }
        }
    }

    fn copy_dir(&self, src: &PathBuf, dst: &PathBuf) -> anyhow::Result<()> {
        fs::create_dir_all(dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if file_type.is_file() {
                fs::copy(&src_path, &dst_path)?;
            } else if file_type.is_dir() {
                self.copy_dir(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }

    fn write_report_header(&self, report: &RunReport, content: &mut String) {
        content.push_str("# Test Run Report\n\n");
        content.push_str("## Scenario\n\n");
        content.push_str(&format!("- **ID**: {}\n", report.scenario_id));
        content.push_str(&format!("- **Tool**: {}\n", report.tool));
        content.push_str(&format!("- **Model**: {}\n", report.model));
        content.push_str(&format!("- **Timestamp**: {}\n\n", report.timestamp));
    }

    fn write_execution_section(&self, report: &RunReport, content: &mut String) {
        content.push_str("## Execution\n\n");
        content.push_str(&format!("- **Duration**: {:.2}s\n", report.duration_secs));
        if let Some(cost) = report.cost_usd {
            content.push_str(&format!("- **Cost**: ${:.4}\n", cost));
        }

        if !report.setup_commands.is_empty() {
            content.push_str(&format!(
                "- **Setup**: {}\n",
                if report.setup_success {
                    "Success"
                } else {
                    "Failed"
                }
            ));
            content.push_str("\n### Setup Commands\n\n");
            for cmd_result in &report.setup_commands {
                let status = if cmd_result.success { "✓" } else { "✗" };
                let redacted_command = redact_sensitive(&cmd_result.command);
                content.push_str(&format!("- {} `{}`\n", status, redacted_command));
            }
            content.push('\n');
        }
        if let Some(ref usage) = report.token_usage {
            content.push_str(&format!(
                "- **Token Usage**: {} input, {} output\n",
                usage.input, usage.output
            ));
        }
        content.push_str(&format!("- **Outcome**: {}\n\n", report.outcome));
    }

    fn write_evaluation_section(&self, report: &RunReport, content: &mut String) {
        content.push_str("## Evaluation Metrics\n\n");
        content.push_str(&format!(
            "- **Gates Passed**: {}/{}\n",
            report.gates_passed, report.gates_total
        ));
        content.push_str(&format!("- **Notes Created**: {}\n", report.note_count));
        content.push_str(&format!("- **Links Created**: {}\n", report.link_count));
        if let Some(score) = report.composite_score {
            content.push_str(&format!("- **Composite Score**: {:.2}\n", score));
        }
        content.push('\n');

        if !report.gate_details.is_empty() {
            content.push_str("### Gate Details\n\n");
            for detail in &report.gate_details {
                let status = if detail.passed { "✓" } else { "✗" };
                let redacted_message = redact_sensitive(&detail.message);
                content.push_str(&format!(
                    "- {} {}: {}\n",
                    status, detail.gate_type, redacted_message
                ));
            }
            content.push('\n');
        }
    }

    fn write_efficiency_section(&self, report: &RunReport, content: &mut String) {
        content.push_str("## Efficiency\n\n");
        content.push_str(&format!(
            "- **Total Commands**: {}\n",
            report.efficiency.total_commands
        ));
        content.push_str(&format!(
            "- **Unique Commands**: {}\n",
            report.efficiency.unique_commands
        ));
        content.push_str(&format!(
            "- **Error Count**: {}\n",
            report.efficiency.error_count
        ));
        content.push_str(&format!(
            "- **First Try Success Rate**: {:.1}%\n",
            report.efficiency.first_try_success_rate * 100.0
        ));
        content.push_str(&format!(
            "- **Iteration Ratio**: {:.2}\n\n",
            report.efficiency.iteration_ratio
        ));
    }

    fn write_quality_section(&self, report: &RunReport, content: &mut String) {
        content.push_str("## Quality\n\n");
        content.push_str(&format!(
            "- **Average Title Length**: {:.1}\n",
            report.quality.avg_title_length
        ));
        content.push_str(&format!(
            "- **Average Body Length**: {:.1}\n",
            report.quality.avg_body_length
        ));
        content.push_str(&format!(
            "- **Average Tags per Note**: {:.2}\n",
            report.quality.avg_tags_per_note
        ));
        content.push_str(&format!(
            "- **Links per Note**: {:.2}\n",
            report.quality.links_per_note
        ));
        content.push_str(&format!(
            "- **Orphan Notes**: {}\n",
            report.quality.orphan_notes
        ));
    }

    pub fn write_report(&self, report: &RunReport) -> anyhow::Result<()> {
        let mut content = String::new();
        self.write_report_header(report, &mut content);
        self.write_execution_section(report, &mut content);
        self.write_evaluation_section(report, &mut content);
        self.write_efficiency_section(report, &mut content);
        self.write_quality_section(report, &mut content);

        fs::write(self.results_dir.join("report.md"), content)?;
        Ok(())
    }

    pub fn write_evaluation(&self, evaluation: &EvaluationReport) -> anyhow::Result<()> {
        let mut content = String::new();

        content.push_str("# Evaluation\n\n");

        content.push_str("## Summary\n\n");
        content.push_str(&format!("- **Scenario**: {}\n", evaluation.scenario_id));
        content.push_str(&format!("- **Tool**: {}\n", evaluation.tool));
        content.push_str(&format!("- **Model**: {}\n", evaluation.model));
        content.push_str(&format!("- **Outcome**: {}\n\n", evaluation.outcome));

        if let Some(judge_score) = evaluation.judge_score_1_to_5 {
            content.push_str("## Judge Score\n\n");
            content.push_str(&format!("**{}** / 5\n\n", judge_score));
        }

        content.push_str("## Metrics\n\n");
        content.push_str(&format!(
            "- **Gates Passed**: {}/{}\n",
            evaluation.gates_passed, evaluation.gates_total
        ));
        content.push_str(&format!("- **Notes Created**: {}\n", evaluation.note_count));
        content.push_str(&format!("- **Links Created**: {}\n", evaluation.link_count));
        content.push_str(&format!(
            "- **Duration**: {:.2}s\n",
            evaluation.duration_secs
        ));
        if let Some(cost) = evaluation.cost_usd {
            content.push_str(&format!("- **Cost**: ${:.4}\n", cost));
        }
        content.push_str(&format!(
            "- **Composite Score**: {:.2}\n\n",
            evaluation.composite_score
        ));

        if !evaluation.judge_feedback.is_empty() {
            content.push_str("## Judge Feedback\n\n");
            for feedback in &evaluation.judge_feedback {
                content.push_str(&format!("{}\n", feedback));
            }
            content.push('\n');
        }

        content.push_str("## Human Review\n\n");
        content.push_str("<!--\n");
        content.push_str("Human Score: __/5\n\n");
        content.push_str("Further Human Notes:\n");
        content.push_str("-->\n\n");

        content.push_str("## Links\n\n");
        content.push_str("- [Transcript](transcript.raw.txt)\n");
        content.push_str("- [Metrics](metrics.json)\n");
        content.push_str("- [Events](events.jsonl)\n");
        content.push_str("- [Fixture](../fixture/)\n");
        content.push_str("- [Store Snapshot](store_snapshot/export.json)\n");

        fs::write(self.results_dir.join("evaluation.md"), content)?;
        Ok(())
    }
}
