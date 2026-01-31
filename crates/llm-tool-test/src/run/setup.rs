use crate::fixture::TestEnv;
use crate::scenario::{Scenario, Setup};
use crate::transcript::TranscriptWriter;
use crate::utils::resolve_fixtures_path;
use std::path::PathBuf;

pub fn setup_scenario_env(
    s: &Scenario,
    results_dir: &PathBuf,
) -> anyhow::Result<(TestEnv, String, String)> {
    let fixtures_path = resolve_fixtures_path("");
    let fixtures_path_str = fixtures_path.to_str().unwrap_or("llm-test-fixtures");
    let scenario_path = format!("{}/{}.yaml", fixtures_path_str, s.name);
    let scenario_yaml = std::fs::read_to_string(&scenario_path)?;
    let prompt = s.task.prompt.clone();

    println!(
        "Setting up environment for template folder: {}",
        s.template_folder
    );
    let env_root = results_dir.join("fixture");
    let env = TestEnv::new(env_root)?;
    env.setup_fixture(&s.template_folder)?;
    println!("Environment created at: {:?}", env.root);

    let _prime_output = env.get_prime_output();

    Ok((env, scenario_yaml, prompt))
}

pub fn execute_setup_commands(
    setup: &Setup,
    env: &TestEnv,
    writer: &TranscriptWriter,
    effective_timeout: u64,
) -> anyhow::Result<(bool, Vec<(String, bool, String)>)> {
    println!("Running {} setup command(s)...", setup.commands.len());
    let runner = crate::session::SessionRunner::new();
    let mut setup_success = true;
    let mut setup_commands: Vec<(String, bool, String)> = Vec::new();

    for (i, cmd) in setup.commands.iter().enumerate() {
        println!("  Command {}/{}: {}", i + 1, setup.commands.len(), cmd);
        let (output, exit_code) =
            runner.run_command("sh", &["-c", cmd], &env.root, effective_timeout)?;

        let success = exit_code == 0;
        setup_commands.push((cmd.to_string(), success, output.clone()));

        writer.append_event(&serde_json::json!({
            "type": "setup_command",
            "index": i,
            "command": cmd,
            "exit_code": exit_code,
            "output": output,
            "success": success,
        }))?;

        if !success {
            setup_success = false;
            println!("  Command failed with exit code {}", exit_code);
        }
    }
    println!("Setup complete.");

    Ok((setup_success, setup_commands))
}

pub fn prepare_writer_and_setup(
    results_dir: &PathBuf,
    env: &TestEnv,
    s: &Scenario,
    effective_timeout: u64,
) -> anyhow::Result<(PathBuf, TranscriptWriter, bool, Vec<(String, bool, String)>)> {
    let artifacts_dir = results_dir.join("artifacts");
    std::fs::create_dir_all(&artifacts_dir)?;
    let writer = TranscriptWriter::new(artifacts_dir.clone(), results_dir.clone())?;

    let (setup_success, setup_commands) = if let Some(setup) = &s.setup {
        execute_setup_commands(setup, env, &writer, effective_timeout)?
    } else {
        (true, vec![])
    };

    Ok((artifacts_dir, writer, setup_success, setup_commands))
}
