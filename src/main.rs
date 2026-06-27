//! # Tantra — Intelligent Coworker & Engineering Loop Orchestrator
//!
//! Connects to agentic-memory, delegates coding to OpenHands, and runs
//! the engineering loop: Plan → Code → Test → Review → Learn.
//!
//! ## Commands
//! ```bash
//! tantra status          — Show memory stats
//! tantra health          — Show system health
//! tantra test            — Run integration test against memory API
//! tantra search <query>  — Smart search memory
//! tantra eng-loop <task> — Run full engineering loop
//! tantra plan <task>     — Generate a plan for a task
//! tantra seed-design     — Seed design system into memory
//! ```

use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::process::Command;

mod eng_loop;

#[derive(Debug, Deserialize)]
struct MemoryStats {
    total_records: u64,
    total_with_embeddings: u64,
    storage_bytes: u64,
    tier_breakdown: HashMap<String, TierBreakdown>,
}

#[derive(Debug, Deserialize)]
struct TierBreakdown {
    tier: String,
    total_records: u64,
    average_importance: f64,
    total_accesses: u64,
}

use serde::Deserialize;

#[derive(Parser)]
#[command(name = "tantra", version = "0.2.0", about = "🧠 Tantra — Engineering Loop Orchestrator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show memory stats from agentmemory
    Status,
    /// Show system health
    Health,
    /// Run integration test against memory API
    Test,
    /// Smart search across memory
    Search { query: Vec<String> },
    /// Run the full engineering loop: Plan → Code(OpenHands) → Test → Review → Learn
    #[command(name = "eng-loop")]
    EngLoop {
        task: Vec<String>,
        #[arg(short, long, default_value = "3")]
        max_iterations: usize,
    },
    /// Generate a plan for a task (dry-run the planning phase)
    Plan { task: Vec<String> },
    /// Seed the Tredo design system into agentmemory
    #[command(name = "seed-design")]
    SeedDesign,
    /// Check if OpenHands is available
    #[command(name = "check-openhands")]
    CheckOpenHands,
}

#[tokio::main]
async fn main() {
    println!("  ╔═══════════════════════════════════════╗");
    println!("  ║   🧠 Tantra — Engineering Loop Agent   ║");
    println!("  ║   v0.2.0  |  Plan → Code → Learn       ║");
    println!("  ╚═══════════════════════════════════════╝\n");

    let cli = Cli::parse();

    match &cli.command {
        Commands::Status => cmd_status().await,
        Commands::Health => cmd_health().await,
        Commands::Test => cmd_test().await,
        Commands::Search { query } => cmd_search(&query.join(" ")).await,
        Commands::EngLoop { task, max_iterations } => {
            cmd_eng_loop(&task.join(" "), *max_iterations).await
        }
        Commands::Plan { task } => cmd_plan(&task.join(" ")).await,
        Commands::SeedDesign => cmd_seed_design().await,
        Commands::CheckOpenHands => cmd_check_openhands(),
    }
}

// ── OpenHands Availability Check ────────────────────────────────────────────

fn cmd_check_openhands() {
    println!("  🔍 Checking OpenHands...\n");
    let config = eng_loop::EngLoopConfig::default();
    match Command::new(&config.openhands_path)
        .arg("--version")
        .output()
    {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("  ✅ OpenHands is available");
            println!("  Path:    {}", config.openhands_path);
            println!("  Version: {}", version);
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("  ❌ OpenHands found but returned error:\n     {}", stderr.trim());
        }
        Err(e) => {
            println!("  ❌ OpenHands not available: {}", e);
            println!();
            println!("  Install with: pip install openhands");
            println!("  Or set:       OPENHANDS_PATH=/path/to/openhands");
        }
    }
}

// ── Engineering Loop Commands ───────────────────────────────────────────────

async fn cmd_eng_loop(task: &str, max_iterations: usize) {
    let config = eng_loop::EngLoopConfig::default();

    println!("  Task:            {}", task);
    println!("  Max iterations:  {}", max_iterations);
    println!("  Project dir:     {}", config.project_dir);
    println!("  Memory API:      {}", config.memory_api_url);
    println!("  OpenHands:       {}\n", config.openhands_path);

    match eng_loop::run_engineering_loop(task, max_iterations, &config).await {
        Ok(run) => {
            if run.success {
                println!("  ✅ Engineering loop completed successfully!\n");
            } else {
                println!(
                    "  ⚠️  Engineering loop completed with {}/{} steps passing.\n",
                    run.steps.iter().filter(|s| s.success).count(),
                    run.steps.len()
                );
            }
            // Print lessons
            if !run.lessons.is_empty() {
                println!("  📚 Lessons learned:");
                for lesson in &run.lessons {
                    println!("     • {}", lesson);
                }
                println!();
            }
        }
        Err(e) => {
            eprintln!("  ❌ Engineering loop failed: {}", e);
        }
    }
}

async fn cmd_plan(task: &str) {
    let config = eng_loop::EngLoopConfig::default();
    println!("  📋 Generating plan for: '{}'\n", task);

    match eng_loop::plan_task(task, &config) {
        Ok(plan) => {
            println!("{}", plan);
        }
        Err(e) => {
            eprintln!("  ❌ Planning failed: {}", e);
        }
    }
}

// ── Memory API Commands (existing) ─────────────────────────────────────────

fn get_base_url() -> String {
    std::env::var("MEMORY_API_URL").unwrap_or_else(|_| "http://localhost:3111".to_string())
}

async fn fetch_json(client: &reqwest::Client, url: &str) -> Result<serde_json::Value, String> {
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("HTTP {}", status));
    }
    resp.json()
        .await
        .map_err(|e| format!("Parse failed: {}", e))
}

async fn cmd_status() {
    let base_url = get_base_url();
    let client = reqwest::Client::new();

    println!("  📊 Memory System Status\n");
    match fetch_json(&client, &format!("{}/stats", base_url)).await {
        Ok(json) => {
            let stats: MemoryStats = serde_json::from_value(json).unwrap_or_else(|_| MemoryStats {
                total_records: 0,
                total_with_embeddings: 0,
                storage_bytes: 0,
                tier_breakdown: HashMap::new(),
            });
            println!("  Total records:      {}", stats.total_records);
            println!("  With embeddings:    {}", stats.total_with_embeddings);
            println!("  Storage:            {} bytes", stats.storage_bytes);
            println!("\n  Tier Breakdown:");
            for (name, tier) in &stats.tier_breakdown {
                println!(
                    "    • {} [{}] — {} records, importance {:.2}, {} accesses",
                    name, tier.tier, tier.total_records, tier.average_importance, tier.total_accesses
                );
            }
        }
        Err(e) => {
            eprintln!(
                "  ❌ {}\n  Make sure agentic-memory is running on {}",
                e, base_url
            );
        }
    }
}

async fn cmd_health() {
    let base_url = get_base_url();
    let client = reqwest::Client::new();

    println!("  🏥 System Health\n");
    match fetch_json(&client, &format!("{}/health", base_url)).await {
        Ok(json) => {
            println!("  Total records:    {}", json["total_records"]);
            println!("  Across tiers:     {}", json["total_across_tiers"]);
            println!("  Graph edges:      {}", json["graph_edges"]);
            println!("\n  Recommendations:");
            if let Some(recommendations) = json["recommendations"].as_array() {
                for rec in recommendations {
                    println!("    • {}", rec.as_str().unwrap_or("?"));
                }
            }
            // Also show Tantra's own status
            println!("\n  Tantra:");
            println!("    OpenHands: {}", if check_openhands_available() { "✅ Available" } else { "❌ Not found" });
        }
        Err(e) => eprintln!("  ❌ {}", e),
    }
}

fn check_openhands_available() -> bool {
    Command::new(eng_loop::EngLoopConfig::default().openhands_path)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

async fn cmd_test() {
    let base_url = get_base_url();
    let client = reqwest::Client::new();

    println!("  🧪 Running integration test...\n");

    // Test 1: Stats
    println!("  1. GET /stats ...");
    match fetch_json(&client, &format!("{}/stats", base_url)).await {
        Ok(j) => println!("     ✅ Stats OK ({} records)", j["total_records"]),
        Err(e) => {
            eprintln!("     ❌ {}", e);
            return;
        }
    }

    // Test 2: Insert record
    println!("  2. POST /records ...");
    let body = serde_json::json!({
        "id": format!("test-tantra-{}", chrono::Utc::now().timestamp()),
        "content": "Engineering loop test: Tantra orchestrator verified working",
        "content_type": "test",
        "metadata": {"source": "tantra-test"},
        "tier": "episodic",
        "importance": 0.5
    });
    let resp = client
        .post(format!("{}/records", base_url))
        .json(&body)
        .send()
        .await;
    match resp {
        Ok(r) if r.status().as_u16() == 201 => println!("     ✅ Record inserted"),
        Ok(r) => {
            eprintln!("     ❌ HTTP {}", r.status());
            return;
        }
        Err(e) => {
            eprintln!("     ❌ {}", e);
            return;
        }
    }

    // Test 3: Search
    println!("  3. GET /search?q=engineering+loop ...");
    match fetch_json(&client, &format!("{}/search?q=engineering+loop", base_url)).await {
        Ok(j) => println!(
            "     ✅ Search: {} results",
            j.as_array().map(|a| a.len()).unwrap_or(0)
        ),
        Err(e) => {
            eprintln!("     ❌ {}", e);
        }
    }

    // Test 4: Health
    println!("  4. GET /health ...");
    match fetch_json(&client, &format!("{}/health", base_url)).await {
        Ok(j) => println!(
            "     ✅ Health OK ({} recommendations)",
            j["recommendations"]
                .as_array()
                .map(|a| a.len())
                .unwrap_or(0)
        ),
        Err(e) => {
            eprintln!("     ❌ {}", e);
        }
    }

    // Test 5: Check OpenHands
    println!("  5. OpenHands availability ...");
    if check_openhands_available() {
        let version = Command::new(eng_loop::EngLoopConfig::default().openhands_path)
            .arg("--version")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        println!("     ✅ OpenHands available (version: {})", version);
    } else {
        println!("     ⚠️  OpenHands not found — install with: pip install openhands");
    }

    println!("\n  ✅ All integration tests passed!\n");
}async fn cmd_search(query: &str) {
    let base_url = get_base_url();
    let client = reqwest::Client::new();

    println!("  🔍 Searching for: '{}'\n", query);
    let url = format!("{}/search/smart", base_url);
    let resp = client
        .get(&url)
        .query(&[("q", query)])
        .send()
        .await;

    match resp {
        Ok(r) => {
            if !r.status().is_success() {
                eprintln!("  ❌ Search API returned HTTP {}", r.status());
                return;
            }
            match r.json::<serde_json::Value>().await {
                Ok(json) => {
                    let results = json.as_array().cloned().unwrap_or_default();
                    if results.is_empty() {
                        println!("  No results found.");
                    }
                    for (i, r) in results.iter().enumerate() {
                        let score = r["score"].as_f64().unwrap_or(0.0);
                        let content = r["record"]["content"].as_str().unwrap_or("?");
                        let ct = r["record"]["content_type"].as_str().unwrap_or("?");
                        println!("  {}. [{}] (score: {:.2})", i + 1, ct, score);
                        println!(
                            "     {}",
                            &content.chars().take(120).collect::<String>()
                        );
                        println!();
                    }
                }
                Err(e) => eprintln!("  ❌ JSON parse error: {}", e),
            }
        }
        Err(e) => eprintln!("  ❌ Request failed: {}", e),
    }
}

// ── Seed Design System ─────────────────────────────────────────────────────

async fn cmd_seed_design() {
    let base_url = get_base_url();
    let client = reqwest::Client::new();

    println!("  🎨 Seeding Tredo Design System into agentmemory...\n");

    let design_records: Vec<(&str, &str, &str, f64)> = vec![
        (
            "design-color-brand",
            "Tredo Exchange Design System — Brand & Colors:\n- Tredo Yellow (primary): #FCD535\n- Canvas Dark: #0b0e11\n- Surface Card: #1e2329\n- Surface Elevated: #2b3139",
            "design-color-brand",
            0.8,
        ),
        (
            "design-trading-semantics",
            "Tredo Exchange Design System — Trading Semantics:\n- Trading Up: #0ecb81 (green, price increase)\n- Trading Down: #f6465d (red, price decrease)\n- These are SEMANTIC price tokens — do not repurpose for success/error states",
            "design-trading-semantics",
            0.7,
        ),
        (
            "design-typography",
            "Tredo Exchange Design System — Typography:\n- Font stack: Inter (sans-serif) for body, JetBrains Mono (monospace) for numbers\n- Body MD: 14px/400/1.5\n- Button: 14px/600/1",
            "design-typography",
            0.75,
        ),
        (
            "design-components",
            "Tredo Exchange Design System — Components:\n- Primary Button: Yellow (#FCD535) bg, Black text, 6px radius\n- Trading Up: Green (#0ecb81) for Buy/Long\n- Trading Down: Red (#f6465d) for Sell/Short",
            "design-components",
            0.7,
        ),
        (
            "design-trading-desk",
            "Tredo Exchange Design System — Trading Desk:\n- 3-col layout: Order Book | Chart | Trade Form + AI Pre-Trade\n- Side Switch: Buy/Long (green) / Sell/Short (red)\n- AI Pre-Trade: Edge score, confluence, debate, kronos, guardian",
            "design-trading-desk",
            0.65,
        ),
    ];

    let mut success = 0;
    let mut failed = 0;

    for (id, content, content_type, importance) in &design_records {
        print!("  {} ... ", id);
        let body = serde_json::json!({
            "id": id,
            "content": content,
            "content_type": content_type,
            "metadata": {"source": "tredo-design-system"},
            "tier": "semantic",
            "importance": importance,
        });
        match client
            .post(format!("{}/records", base_url))
            .json(&body)
            .send()
            .await
        {
            Ok(r) if r.status().as_u16() == 201 => {
                println!("✅");
                success += 1;
            }
            Ok(r) => {
                println!("❌ HTTP {}", r.status());
                failed += 1;
            }
            Err(e) => {
                println!("❌ {}", e);
                failed += 1;
            }
        }
    }

    println!();
    if failed == 0 {
        println!("  ✅ Design system seeded: {} records written", success);
        println!("  Search: tantra search 'tredo design colors'");
    } else {
        eprintln!(
            "  ⚠️  {} succeeded, {} failed. Is agentmemory running on {}?",
            success, failed, base_url
        );
    }
}
