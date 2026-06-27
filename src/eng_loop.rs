//! # Engineering Loop — Tantra's Core Orchestration Cycle
//!
//! The engineering loop is a meta-development cycle that Tantra uses to
//! autonomously plan, implement, test, review, and learn from coding tasks.
//!
//! ## Cycle
//! 1. **Plan** — Research the task, search memory for context, create a plan
//! 2. **Code** — Delegate implementation to OpenHands (or local tools)
//! 3. **Test** — Compile and run tests on the implementation
//! 4. **Review** — Analyze test results, decide if the loop continues
//! 5. **Learn** — Store outcomes in memory for future reference
//!
//! ## Usage
//! ```bash
//! tantra eng-loop "Add a new API endpoint for user portfolio history"
//! tantra plan "Refactor the order book rendering"
//! tantra run-loop --max-iterations 3 "Fix all compiler warnings"
//! ```

use chrono::Utc;
use serde::{Deserialize, Serialize};

/// The current state of the engineering loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoopPhase {
    Plan,
    Code,
    Test,
    Review,
    Learn,
    Complete,
}

/// A single step in the engineering loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopStep {
    pub phase: LoopPhase,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub output: Option<String>,
    pub success: bool,
}

/// The full engineering loop run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngLoopRun {
    pub task: String,
    pub max_iterations: usize,
    pub iteration: usize,
    pub steps: Vec<LoopStep>,
    pub plan: Option<String>,
    pub code_result: Option<String>,
    pub test_result: Option<String>,
    pub review_result: Option<String>,
    pub lessons: Vec<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub success: bool,
}

impl EngLoopRun {
    pub fn new(task: String, max_iterations: usize) -> Self {
        Self {
            task,
            max_iterations,
            iteration: 0,
            steps: Vec::new(),
            plan: None,
            code_result: None,
            test_result: None,
            review_result: None,
            lessons: Vec::new(),
            started_at: Utc::now().to_rfc3339(),
            completed_at: None,
            success: false,
        }
    }

    fn add_step(&mut self, phase: LoopPhase, success: bool, output: Option<String>) {
        self.steps.push(LoopStep {
            phase,
            started_at: Utc::now().to_rfc3339(),
            completed_at: Some(Utc::now().to_rfc3339()),
            output,
            success,
        });
    }
}

/// Configuration for the engineering loop
#[derive(Debug, Clone)]
pub struct EngLoopConfig {
    pub memory_api_url: String,
    pub project_dir: String,
    pub openhands_path: String,
    pub cargo_args: Vec<String>,
}

impl Default for EngLoopConfig {
    fn default() -> Self {
        Self {
            memory_api_url: std::env::var("MEMORY_API_URL")
                .unwrap_or_else(|_| "http://localhost:3111".to_string()),
            project_dir: std::env::var("TANTRA_PROJECT_DIR").unwrap_or_else(|_| {
                // Default to the parent of the Tantra directory
                let current = std::env::current_dir().unwrap_or_default();
                current.to_string_lossy().to_string()
            }),
            openhands_path: std::env::var("OPENHANDS_PATH")
                .unwrap_or_else(|_| "openhands".to_string()),
            cargo_args: vec!["--workspace".to_string()],
        }
    }
}

// ── Plan Phase ───────────────────────────────────────────────────────────────

/// Generate a plan for the given task by searching memory and analyzing context.
pub fn plan_task(_task: &str, _config: &EngLoopConfig) -> Result<String, String> {
    // In a full implementation, this would:
    // 1. Search agentmemory for similar past tasks
    // 2. Search the codebase for relevant files
    // 3. Use an LLM to create a structured plan
    // For now, we generate a basic plan structure.
    Ok(format!(
        "## Plan: {}\n\n\
         1. **Research** — Understand the codebase context\n\
         2. **Implement** — Write or modify the necessary code\n\
         3. **Build** — Compile the project to verify correctness\n\
         4. **Test** — Run existing tests to check for regressions\n\
         5. **Review** — Check output and learn from results\n",
        _task
    ))
}

// ── Code Phase — Delegate to OpenHands ──────────────────────────────────────

/// Delegate a coding task to OpenHands and return the result.
pub async fn delegate_to_openhands(plan: &str, config: &EngLoopConfig) -> Result<String, String> {
    let now = Utc::now().format("%Y%m%d_%H%M%S_%3f");
    let pid = std::process::id();

    // Build a focused task prompt for OpenHands
    let task_prompt = format!(
        "You are a coding sub-agent for the Tantra engineering loop.\n\
         Your task is to implement the following plan:\n\n\
         {}\n\n\
         Instructions:\n\
         - Work in the project directory: {}\n\
         - After implementing, run `cargo check` to verify compilation\n\
         - Report what files you changed and the result of `cargo check`\n\
         - Be concise and precise",
        plan, config.project_dir
    );

    // Write the task to a temp file for OpenHands (unique: timestamp+pid)
    let task_file = format!("/tmp/tantra_task_{}_{}.md", now, pid);
    std::fs::write(&task_file, &task_prompt)
        .map_err(|e| format!("Failed to write task file: {}", e))?;

    // Run OpenHands with the task file (with 10-minute timeout)
    let cmd_future = tokio::process::Command::new(&config.openhands_path)
        .arg("-f")
        .arg(&task_file)
        .arg("--headless")
        .arg("--always-approve")
        .output();

    let output = tokio::time::timeout(std::time::Duration::from_secs(600), cmd_future)
        .await
        .map_err(|_| "OpenHands timed out after 600s")?
        .map_err(|e| format!("Failed to execute OpenHands: {}", e))?;

    let _ = std::fs::remove_file(&task_file);

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        // Even on non-zero exit, capture output — OpenHands may have partial results
        let combined = format!(
            "OpenHands exit code: {}\nSTDOUT:\n{}\nSTDERR:\n{}",
            output.status.code().unwrap_or(-1),
            truncate(&stdout, 2000),
            truncate(&stderr, 1000)
        );
        return Err(combined);
    }

    Ok(format!(
        "{}\n{}",
        truncate(&stdout, 4000),
        truncate(&stderr, 1000)
    ))
}

// ── Test Phase ───────────────────────────────────────────────────────────────

/// Run cargo check on the project to verify compilation.
pub async fn run_cargo_check(config: &EngLoopConfig) -> Result<String, String> {
    let mut cmd = tokio::process::Command::new("cargo");
    cmd.arg("check");
    for arg in &config.cargo_args {
        cmd.arg(arg);
    }
    cmd.current_dir(&config.project_dir);

    let cmd_future = cmd.output();
    let output = tokio::time::timeout(std::time::Duration::from_secs(180), cmd_future)
        .await
        .map_err(|_| "cargo check timed out after 180s")?
        .map_err(|e| format!("Failed to run cargo check: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(format!(
            "Cargo check FAILED (exit {})\n{}\n{}",
            output.status.code().unwrap_or(-1),
            truncate(&stdout, 2000),
            truncate(&stderr, 2000)
        ));
    }

    Ok("cargo check: PASSED".to_string())
}

/// Run cargo test on the project.
#[allow(dead_code)]
pub async fn run_cargo_test(
    config: &EngLoopConfig,
    test_filter: Option<&str>,
) -> Result<String, String> {
    let mut cmd = tokio::process::Command::new("cargo");
    cmd.arg("test");
    if let Some(filter) = test_filter {
        cmd.arg(filter);
    }
    for arg in &config.cargo_args {
        cmd.arg(arg);
    }
    cmd.current_dir(&config.project_dir);

    let cmd_future = cmd.output();
    let output = tokio::time::timeout(std::time::Duration::from_secs(300), cmd_future)
        .await
        .map_err(|_| "cargo test timed out after 300s")?
        .map_err(|e| format!("Failed to run cargo test: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        // Extract test failure summary
        let failures = stdout
            .lines()
            .filter(|l| l.contains("FAILED") || l.contains("panicked"))
            .collect::<Vec<_>>();
        let fail_summary = if failures.is_empty() {
            "Tests failed (see full output)".to_string()
        } else {
            failures.join("\n")
        };
        return Err(format!(
            "Cargo test FAILED (exit {})\n{}\nSTDERR:\n{}\nFAILURES:\n{}",
            output.status.code().unwrap_or(-1),
            truncate(&stdout, 2000),
            truncate(&stderr, 1000),
            fail_summary
        ));
    }

    // Extract test result summary
    let summary = stdout
        .lines()
        .filter(|l| l.contains("test result") || l.contains("running"))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(if summary.is_empty() {
        "cargo test: PASSED".to_string()
    } else {
        format!("cargo test: PASSED\n{}", summary)
    })
}

// ── Learn Phase ──────────────────────────────────────────────────────────────

/// Store a lesson in the agentmemory system.
pub async fn store_lesson(
    lesson: &str,
    task: &str,
    success: bool,
    config: &EngLoopConfig,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "id": format!("eng-loop-{}", Utc::now().timestamp()),
        "content": lesson,
        "content_type": "engineering_lesson",
        "metadata": {
            "source": "tantra-eng-loop",
            "task": task,
            "success": success,
            "timestamp": Utc::now().to_rfc3339(),
        },
        "tier": "procedural",
        "importance": if success { 0.6 } else { 0.9 },
    });

    let resp = client
        .post(format!("{}/records", config.memory_api_url))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Memory API error: {}", e))?;

    if resp.status().as_u16() == 201 {
        Ok(())
    } else {
        Err(format!("Memory API returned HTTP {}", resp.status()))
    }
}

/// Search memory for past engineering lessons.
#[allow(dead_code)]
pub async fn search_lessons(
    query: &str,
    config: &EngLoopConfig,
) -> Result<Vec<serde_json::Value>, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/search/smart?q={}", config.memory_api_url, query);
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Memory search error: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Memory API returned HTTP {}", resp.status()));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(json.as_array().cloned().unwrap_or_default())
}

// ── Main Loop Runner ────────────────────────────────────────────────────────

/// Run the full engineering loop for a given task.
pub async fn run_engineering_loop(
    task: &str,
    max_iterations: usize,
    config: &EngLoopConfig,
) -> Result<EngLoopRun, String> {
    let mut run = EngLoopRun::new(task.to_string(), max_iterations);
    println!(
        "\n  ⚙️  Engineering Loop — iteration {}/{}\n",
        1, max_iterations
    );

    // Phase 1: Plan
    println!("  📋 [Plan] Researching and planning...");
    match plan_task(task, config) {
        Ok(plan) => {
            run.plan = Some(plan.clone());
            run.add_step(LoopPhase::Plan, true, Some(plan.clone()));
            println!("  ✅ Plan ready\n");
        }
        Err(e) => {
            run.add_step(LoopPhase::Plan, false, Some(e.clone()));
            run.success = false;
            run.completed_at = Some(Utc::now().to_rfc3339());
            return Err(format!("Planning failed: {}", e));
        }
    }

    // Phase 2: Code (delegate to OpenHands)
    println!("  🤖 [Code] Delegating to OpenHands...");
    let plan = run.plan.as_deref().unwrap_or(task);
    match delegate_to_openhands(plan, config).await {
        Ok(result) => {
            run.code_result = Some(result.clone());
            run.add_step(LoopPhase::Code, true, Some(result.clone()));
            println!("  ✅ OpenHands completed\n");
        }
        Err(e) => {
            run.code_result = Some(e.clone());
            run.add_step(LoopPhase::Code, false, Some(e.clone()));
            println!("  ⚠️  OpenHands returned issues, continuing to test...\n");
        }
    }

    // Phase 3: Test (cargo check)
    println!("  🔧 [Test] Running cargo check...");
    match run_cargo_check(config).await {
        Ok(result) => {
            run.test_result = Some(result.clone());
            run.add_step(LoopPhase::Test, true, Some(result.clone()));
            println!("  ✅ Compilation passed\n");
        }
        Err(e) => {
            run.test_result = Some(e.clone());
            run.add_step(LoopPhase::Test, false, Some(e.clone()));
            let first_line = e.lines().next().unwrap_or(&e).to_string();
            println!("  ❌ Compilation failed:\n     {}\n", first_line);
        }
    }

    // Phase 4: Review
    println!("  👁️  [Review] Analyzing results...");
    let trunc = |s: &Option<String>| -> String {
        s.as_ref()
            .map(|s| {
                if s.len() > 100 {
                    format!("{}...", &s[..100])
                } else {
                    s.clone()
                }
            })
            .unwrap_or_else(|| "N/A".to_string())
    };
    let review = format!(
        "Plan: {}\nCode: {}\nTest: {}",
        trunc(&run.plan),
        trunc(&run.code_result),
        trunc(&run.test_result)
    );
    run.review_result = Some(review.clone());
    run.add_step(LoopPhase::Review, true, Some(review));
    println!("  ✅ Review complete\n");

    // Phase 5: Learn
    println!("  🧠 [Learn] Storing lessons in memory...");
    let all_passed = run.steps.iter().all(|s| s.success);
    let lesson = if all_passed {
        format!(
            "Engineering loop SUCCESS for task '{}': all phases completed cleanly.",
            task
        )
    } else {
        let failed_phases: Vec<String> = run
            .steps
            .iter()
            .filter(|s| !s.success)
            .map(|s| format!("{:?}", s.phase))
            .collect();
        format!(
            "Engineering loop PARTIAL for task '{}': phases {:?} had issues. Need human review.",
            task, failed_phases
        )
    };
    run.lessons.push(lesson.clone());
    run.add_step(LoopPhase::Learn, true, Some(lesson.clone()));

    // Store in memory (best-effort)
    match store_lesson(&lesson, task, all_passed, config).await {
        Ok(_) => println!("  ✅ Lesson stored in agentmemory"),
        Err(e) => println!("  ⚠️  Could not store lesson: {}", e),
    }

    run.success = all_passed;
    run.completed_at = Some(Utc::now().to_rfc3339());

    // Summary
    println!("\n  ═══════════════════════════════════════");
    println!("  📊 Engineering Loop Summary");
    println!("  ═══════════════════════════════════════");
    println!("  Task:       {}", task);
    println!(
        "  Status:     {}",
        if all_passed {
            "✅ ALL PASSED"
        } else {
            "⚠️  PARTIAL"
        }
    );
    println!("  Duration:   {} steps", run.steps.len());
    for (i, step) in run.steps.iter().enumerate() {
        let icon = if step.success { "✅" } else { "❌" };
        println!("  {}  {}. {:?}", icon, i + 1, step.phase);
    }
    println!();

    Ok(run)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}... (truncated, was {} chars)", &s[..max], s.len())
    }
}
