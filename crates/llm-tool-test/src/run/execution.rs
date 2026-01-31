use crate::adapter::{TokenUsage, ToolAdapter};
use crate::evaluation::EvaluationMetrics;
use crate::fixture::TestEnv;
use crate::scenario::Scenario;

pub fn execute_tool(
    adapter: &Box<dyn ToolAdapter>,
    s: &Scenario,
    env: &TestEnv,
    tool: &str,
    model: &str,
    effective_timeout: u64,
) -> anyhow::Result<(String, i32, f64, Option<TokenUsage>)> {
    let start_time = std::time::Instant::now();
    println!("Running tool '{}' with model '{}'...", tool, model);
    let (output, exit_code, cost_opt, token_usage) =
        adapter.run(s, &env.root, Some(model), effective_timeout)?;
    let _duration = start_time.elapsed();

    let cost = cost_opt.unwrap_or(0.0);

    Ok((output, exit_code, cost, token_usage))
}

pub fn create_adapter_and_check(tool: &str) -> anyhow::Result<Box<dyn ToolAdapter>> {
    use crate::adapter::{
        claude_code::ClaudeCodeAdapter, mock::MockAdapter, opencode::OpenCodeAdapter,
    };
    let adapter: Box<dyn ToolAdapter> = match tool {
        "claude-code" => Box::new(ClaudeCodeAdapter),
        "mock" => Box::new(MockAdapter),
        "opencode" => Box::new(OpenCodeAdapter),
        _ => anyhow::bail!("Unknown tool: {}", tool),
    };

    println!("Checking availability for tool: {}", tool);
    adapter.check_availability()?;

    Ok(adapter)
}

pub fn run_evaluation_flow(
    adapter: &Box<dyn ToolAdapter>,
    s: &Scenario,
    env: &TestEnv,
    tool: &str,
    model: &str,
    effective_timeout: u64,
    no_judge: bool,
) -> anyhow::Result<(
    String,
    i32,
    f64,
    Option<TokenUsage>,
    std::time::Duration,
    EvaluationMetrics,
)> {
    let (output, exit_code, cost, token_usage) =
        execute_tool(adapter, s, env, tool, model, effective_timeout)?;
    let duration = std::time::Instant::now().elapsed();

    println!("Running evaluation...");
    let metrics = crate::evaluation::evaluate(s, &env.root, no_judge)?;
    println!("Evaluation metrics: {:?}", metrics);

    Ok((output, exit_code, cost, token_usage, duration, metrics))
}

pub fn determine_outcome(metrics: &EvaluationMetrics) -> String {
    if metrics.gates_passed < metrics.gates_total {
        format!(
            "Fail: {}/{} gates passed",
            metrics.gates_passed, metrics.gates_total
        )
    } else {
        "Pass".to_string()
    }
}
