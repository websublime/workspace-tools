# AI Agent Research & Specifications

**Date**: January 2025  
**Project**: workspace-node-tools  
**Purpose**: Comprehensive research and technical specifications for implementing an AI-powered agent to enhance CLI usability

---

## 📋 Table of Contents

1. [Vision & Objectives](#vision--objectives)
2. [Architecture Overview](#architecture-overview)
3. [LLM Provider Analysis](#llm-provider-analysis)
4. [Setup & Installation Strategy](#setup--installation-strategy)
5. [Configuration Integration](#configuration-integration)
6. [Component Design](#component-design)
7. [Use Cases & Examples](#use-cases--examples)
8. [Implementation Roadmap](#implementation-roadmap)
9. [Technical Considerations](#technical-considerations)
10. [Key Decisions](#key-decisions)

---

## 🎯 Vision & Objectives

### The Big Idea

Create an AI agent using the [Rig](https://rig.rs/) Rust library that can:
- **Interpret natural language commands** from users
- **Execute operations** through the existing crate APIs (pkg, standard, git)
- **Provide contextual information** about workspace state
- **Suggest actions** based on project patterns and history
- **Reduce learning curve** for new users
- **Automate complex workflows** with intelligent assistance

### Value Proposition

The AI agent transforms the CLI experience from:
```bash
# Traditional approach - requires memorizing commands
$ workspace changeset create --bump minor --env production --packages auth,api
$ workspace bump --dry-run
$ workspace bump --execute --git-tag --git-push
```

To:
```bash
# AI-powered approach - natural language
$ workspace ai "I finished the authentication feature, prepare it for production release"

🤖 AI Agent: I'll help you prepare your authentication work for release.
   
   Analyzing changes...
   ✓ Detected changes in: packages/auth, packages/api
   ✓ Current branch: feature/auth-improvements
   ✓ Found 12 commits since divergence from main
   
   Recommended actions:
   1. Create minor changeset (new features detected)
   2. Add packages: auth, api
   3. Target environment: production
   4. Preview version bump: auth 2.1.0 → 2.2.0
   
   Proceed? [Y/n]: 
```

### Core Benefits

✅ **Reduced Learning Curve** - Natural language instead of command memorization  
✅ **Intelligent Automation** - Complex workflows simplified  
✅ **Error Prevention** - Validation and suggestions before execution  
✅ **Living Documentation** - Agent "knows" all APIs  
✅ **Contextual Analysis** - Understands project state before acting  
✅ **Productivity Boost** - Less time on commands, more on development  

---

## 🏗️ Architecture Overview

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    CLI Entry Point (workspace)                     │
│                                                               │
│  ┌──────────────┐              ┌──────────────────────────┐ │
│  │   Current    │              │      AI Agent Mode       │ │
│  │ CLI Commands │              │     (workspace ai/workspace ask)     │ │
│  └──────────────┘              └──────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                                           │
                    ┌──────────────────────┴──────────────────────┐
                    │                                              │
           ┌────────▼────────┐                          ┌─────────▼─────────┐
           │   Rig Agent     │                          │   Tool Registry   │
           │   (LLM Brain)   │◄─────────────────────────│  (Available Tools)│
           └────────┬────────┘                          └───────────────────┘
                    │                                            │
                    │  Calls Tools Based on Intent               │
                    │                                            │
         ┌──────────┴───────────┬──────────────┬────────────────┴───────┐
         │                      │              │                        │
    ┌────▼─────┐         ┌─────▼──────┐  ┌───▼────────┐      ┌────────▼──────┐
    │   pkg    │         │  standard  │  │    git     │      │  Interactive  │
    │   APIs   │         │    APIs    │  │    APIs    │      │    Prompts    │
    └──────────┘         └────────────┘  └────────────┘      └───────────────┘
```

### Component Stack

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 4: User Interface                                      │
│  - CLI commands (workspace ai)                                     │
│  - Interactive chat mode                                     │
│  - JSON output for CI/CD                                     │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 3: AI Agent (sublime_ai_agent crate)                   │
│  - WorkspaceAgent (orchestrator)                             │
│  - ToolRegistry (available operations)                       │
│  - ConversationManager (history, context)                    │
│  - ProviderRouter (local/cloud selection)                    │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 2: LLM Provider (via Rig library)                      │
│  - Ollama (local, default)                                   │
│  - Anthropic Claude (cloud, premium)                         │
│  - OpenAI GPT-4 (cloud, alternative)                         │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: Core APIs (existing crates)                         │
│  - sublime_pkg_tools (changeset, version, upgrade, audit)    │
│  - sublime_standard_tools (filesystem, project detection)    │
│  - sublime_git_tools (repo operations, history)              │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔍 LLM Provider Analysis

### Comparative Matrix

| Criterion | Ollama (Local) | Anthropic Claude | OpenAI GPT-4 |
|----------|----------------|------------------|--------------|
| **Cost** | ✅ Free | ⚠️ ~$0.015/1K tokens | ⚠️ ~$0.03/1K tokens |
| **Privacy** | ✅ 100% local | ❌ Cloud | ❌ Cloud |
| **Code Quality** | ⚠️ 7/10 | ✅ 9/10 | ✅ 9/10 |
| **Speed** | ✅ Fast (local) | ⚠️ Network dependent | ⚠️ Network dependent |
| **Context Window** | ⚠️ 4K-32K | ✅ 200K | ✅ 128K |
| **Offline Support** | ✅ Yes | ❌ No | ❌ No |
| **Setup Complexity** | ⚠️ Requires install | ✅ API key only | ✅ API key only |
| **Hardware Requirement** | ⚠️ 8GB+ RAM | ✅ None | ✅ None |
| **Maintenance** | ⚠️ Manual updates | ✅ Automatic | ✅ Automatic |

### Recommended Models by Provider

#### 1. Ollama (Local - Default)

**Top Models for Code (2025)**

```yaml
Tier 1 - Production Quality:
  qwen2.5-coder:32b:
    size: ~19GB
    ram: 32GB+
    performance: Excellent
    use_case: Best overall for multi-language projects
    
  deepseek-coder-v2:16b:
    size: ~9GB
    ram: 16GB+
    performance: Excellent
    use_case: Specialized coding tasks, code review
    
  codellama:34b:
    size: ~20GB
    ram: 32GB+
    performance: Very Good
    use_case: Production-quality code generation

Tier 2 - Development (Recommended Default):
  qwen2.5-coder:7b:
    size: ~4.7GB
    ram: 8GB+
    performance: Good
    use_case: General purpose, fast iterations
    
  yi-coder:9b:
    size: ~5.4GB
    ram: 12GB+
    performance: Very Good
    use_case: Full-stack web (JS/TS/Python/Node)
    
  deepseek-coder:6.7b:
    size: ~3.8GB
    ram: 8GB+
    performance: Good
    use_case: Quick iterations, development

Tier 3 - Prototyping:
  codellama:7b:
    size: ~3.8GB
    ram: 8GB
    performance: Adequate
    use_case: Very fast, basic assistance
    
  codegemma:7b:
    size: ~5GB
    ram: 8GB
    performance: Good
    use_case: Lightweight, good for Rust
```

**Performance Benchmarks**

| Model Size | CPU (tok/s) | GPU (tok/s) | RAM Required |
|------------|-------------|-------------|--------------|
| 7B models  | 20-30       | 50-100      | 8GB          |
| 13B models | 10-20       | 30-50       | 16GB         |
| 32B models | 5-10        | 15-30       | 32GB         |

#### 2. Anthropic Claude (Cloud - Premium)

**Models**
- **Claude 3.5 Sonnet** (Current): Best for code, technical reasoning
- **Claude 3 Opus**: Maximum capability, higher cost
- **Claude 3 Haiku**: Fast and economical, less capable

**Key Advantages**
- 200K token context window (ideal for large projects)
- Excellent instruction following
- Superior code analysis and debugging
- Structured, detailed responses

**Pricing**
- Input: ~$3/million tokens
- Output: ~$15/million tokens
- Average query: ~$0.015

#### 3. OpenAI GPT-4 (Cloud - Industry Standard)

**Models**
- **GPT-4 Turbo**: Fast, 128K context
- **GPT-4o**: Latest, optimized
- **GPT-3.5 Turbo**: Economical, less capable

**Key Advantages**
- Most documented and tested
- Strong function calling support
- Mature ecosystem

**Pricing**
- GPT-4 Turbo: ~$0.01 input, ~$0.03 output per 1K tokens
- GPT-4o: Similar pricing
- GPT-3.5: ~$0.0005 input, ~$0.0015 output per 1K tokens

### Hybrid Strategy (Recommended)

```rust
pub enum ProviderTier {
    Local,      // Default: Ollama
    Cloud,      // Premium: Anthropic/OpenAI
    Fallback,   // Auto-failover
}

pub struct ProviderConfig {
    default_tier: ProviderTier,
    local_model: String,        // "qwen2.5-coder:7b"
    cloud_provider: Option<CloudProvider>,
    auto_fallback: bool,
    cost_limits: Option<CostLimits>,
}
```

**Tier 1: Local (Default)**
- ✅ Zero costs
- ✅ Complete privacy
- ✅ Works offline
- ⚠️ Requires initial setup
- ⚠️ Quality slightly lower than cloud

**Use Cases:**
- Frequent operations (list, show, check)
- Basic analysis
- Development/prototyping
- CI/CD (no costs)

**Tier 2: Cloud (Optional Premium)**
- ✅ Maximum quality
- ✅ Zero setup
- ✅ Large context windows
- ⚠️ Usage costs
- ⚠️ Requires internet

**Use Cases:**
- Complex analysis (dependency impact)
- Difficult troubleshooting
- Refactoring suggestions
- Critical operations (release preparation)

**Tier 3: Auto-Fallback**
```
Ollama unavailable/fails → Cloud (if configured)
Cloud fails → Degraded mode (template responses)
```

### Cost Estimation (Cloud)

```
Scenario: Active developer using AI agent

Daily Usage: 50 interactions/day
Average: 500 tokens input + 1000 tokens output per interaction

Local (Ollama):
- Cost: $0.00
- Hardware: Existing RAM

Cloud (Anthropic Claude):
- Cost per interaction: ~$0.015
- Daily cost: ~$0.75
- Monthly cost: ~$22.50

Hybrid (90% local, 10% cloud):
- Monthly cost: ~$2.25
- Best of both worlds! ✨
```

---

## 🚀 Setup & Installation Strategy

### Progressive Enhancement Approach

```
Level 1: Auto-Detection ✅
    ↓
Level 2: Guided Installation 🤝
    ↓
Level 3: Manual Fallback 📖
    ↓
Level 4: Cloud Fallback ☁️
```

### Level 1: Auto-Detection (Always Active)

```rust
pub async fn check_ai_setup() -> AiSetupStatus {
    // 1. Check if Ollama is installed
    if is_ollama_installed().await {
        // 2. Check if it's running
        if is_ollama_running().await {
            // 3. Check if required model exists
            if has_required_model("qwen2.5-coder:7b").await {
                return AiSetupStatus::Ready;
            } else {
                return AiSetupStatus::NeedsModel;
            }
        } else {
            return AiSetupStatus::NeedsStart;
        }
    } else {
        return AiSetupStatus::NeedsInstall;
    }
}

async fn is_ollama_installed() -> bool {
    Command::new("ollama")
        .arg("--version")
        .output()
        .await
        .is_ok()
}

async fn is_ollama_running() -> bool {
    reqwest::get("http://localhost:11434/api/version")
        .await
        .is_ok()
}

async fn has_required_model(model: &str) -> bool {
    // Call Ollama API to list models
    if let Ok(response) = reqwest::get("http://localhost:11434/api/tags").await {
        if let Ok(data) = response.json::<ModelsResponse>().await {
            return data.models.iter().any(|m| m.name == model);
        }
    }
    false
}
```

### Level 2: Guided Installation (Recommended)

**First-time Experience**

```bash
$ workspace ai "list changesets"

🔍 Checking AI setup...
⚠️  Ollama not detected

┌─────────────────────────────────────────────────────────────┐
│ AI Assistant Setup Required                                 │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Ollama (local LLM runtime) is not installed.               │
│ Choose how you'd like to proceed:                          │
│                                                             │
│ 1. 🚀 Quick Setup (Recommended)                            │
│    • Download & install Ollama (~50MB)                     │
│    • Download qwen2.5-coder:7b model (~4.7GB)             │
│    • Configure for workspace use                           │
│    • Estimated time: 5-10 minutes                          │
│                                                             │
│ 2. 📦 Install Ollama Only                                  │
│    • Download & install Ollama                             │
│    • Choose model later                                    │
│    • Estimated time: 2-3 minutes                           │
│                                                             │
│ 3. 📖 Manual Installation                                  │
│    • Show installation instructions                        │
│    • Install yourself                                      │
│                                                             │
│ 4. ☁️  Use Cloud Provider Instead                          │
│    • Setup Anthropic/OpenAI API key                        │
│    • Usage costs apply (~$0.015 per query)                 │
│                                                             │
│ 5. ❌ Skip (disable AI features)                           │
│                                                             │
└─────────────────────────────────────────────────────────────┘

Choice [1-5]: 1

🚀 Starting Ollama quick setup...

📦 Step 1/3: Installing Ollama...
   Running: curl -fsSL https://ollama.com/install.sh
   This will execute the official Ollama install script.
   Proceed? [Y/n]: y
   
   >>> Downloading Ollama...
   >>> Installing to /usr/local/bin...
   
✅ Ollama installed successfully

▶️  Step 2/3: Starting Ollama service...
✅ Ollama is running at http://localhost:11434

📥 Step 3/3: Downloading qwen2.5-coder:7b (~4.7GB)
   This may take a few minutes...
   
   [████████████████████████████████████████] 100% 4.7GB / 4.7GB
   
✅ Model ready!

🧪 Testing setup...
✅ Connection successful

✨ AI Assistant is ready to use!
   Try: workspace ai "what can you help me with?"

Now executing your query: "list changesets"

🤖 AI: Let me check your active changesets...
```

**Installation Implementation (Cross-Platform)**

```rust
async fn quick_setup_ollama(os: Os) -> Result<()> {
    println!("🚀 Starting Ollama quick setup...\n");
    
    // Step 1: Install Ollama
    println!("📦 Step 1/3: Installing Ollama...");
    install_ollama(os).await?;
    println!("✅ Ollama installed successfully\n");
    
    // Step 2: Start service
    println!("▶️  Step 2/3: Starting Ollama service...");
    start_ollama_service(os).await?;
    wait_for_ollama_ready().await?;
    println!("✅ Ollama is running\n");
    
    // Step 3: Download model
    println!("📥 Step 3/3: Downloading qwen2.5-coder:7b (~4.7GB)");
    println!("   This may take a few minutes...");
    download_model_with_progress("qwen2.5-coder:7b").await?;
    println!("\n✅ Model ready!\n");
    
    // Test
    println!("🧪 Testing setup...");
    test_ai_connection().await?;
    println!("✅ Connection successful\n");
    
    println!("✨ AI Assistant is ready to use!");
    Ok(())
}

async fn install_ollama(os: Os) -> Result<()> {
    match os {
        Os::Linux => {
            let script = "curl -fsSL https://ollama.com/install.sh | sh";
            println!("   Running: {}", script);
            
            if !confirm("   Proceed?")? {
                return Err(Error::UserCancelled);
            }
            
            Command::new("sh")
                .arg("-c")
                .arg(script)
                .status()
                .await?
                .success()
                .then_some(())
                .ok_or(Error::InstallFailed)
        }
        Os::MacOs => {
            println!("   Downloading Ollama.app...");
            let url = "https://ollama.com/download/Ollama-darwin.zip";
            download_and_install_macos(url).await
        }
        Os::Windows => {
            println!("   Downloading OllamaSetup.exe...");
            let url = "https://ollama.com/download/OllamaSetup.exe";
            download_and_run_installer_windows(url).await
        }
    }
}

async fn download_model_with_progress(model: &str) -> Result<()> {
    use indicatif::{ProgressBar, ProgressStyle};
    
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("   [{bar:40}] {percent}% {msg}")
            .unwrap()
    );
    
    // Stream download progress from Ollama API
    let client = reqwest::Client::new();
    let mut stream = client
        .post("http://localhost:11434/api/pull")
        .json(&serde_json::json!({
            "name": model,
            "stream": true
        }))
        .send()
        .await?
        .bytes_stream();
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Ok(status) = serde_json::from_slice::<PullStatus>(&chunk) {
            match status {
                PullStatus::Downloading { completed, total } => {
                    let percent = (completed as f64 / total as f64 * 100.0) as u64;
                    pb.set_position(percent);
                    pb.set_message(format!(
                        "{} / {}",
                        humanize_bytes(completed),
                        humanize_bytes(total)
                    ));
                }
                PullStatus::Success => {
                    pb.finish_with_message("Complete");
                    break;
                }
                _ => {}
            }
        }
    }
    
    Ok(())
}
```

### Level 3: Manual Installation

```bash
📖 Manual Ollama Installation

Please install Ollama manually for your platform:

┌─ Linux ─────────────────────────────────────────┐
│ Run in terminal:                                 │
│ curl -fsSL https://ollama.com/install.sh | sh   │
└──────────────────────────────────────────────────┘

┌─ macOS ─────────────────────────────────────────┐
│ 1. Visit: https://ollama.com/download           │
│ 2. Download Ollama-darwin.zip                   │
│ 3. Extract and move to Applications             │
│ 4. Open Ollama.app                              │
└──────────────────────────────────────────────────┘

┌─ Windows ───────────────────────────────────────┐
│ 1. Visit: https://ollama.com/download           │
│ 2. Download OllamaSetup.exe                     │
│ 3. Run the installer                            │
└──────────────────────────────────────────────────┘

After installation, download the model:
  ollama pull qwen2.5-coder:7b

Then try again: workspace ai "list changesets"
```

### Level 4: Cloud Provider Setup

```bash
☁️  Cloud Provider Setup

Which provider would you like to use?

1. Anthropic Claude (Recommended for code)
   • Superior technical understanding
   • 200K context window
   • ~$0.015 per query
   
2. OpenAI GPT-4
   • Industry standard
   • 128K context window
   • ~$0.03 per query

Choice [1-2]: 1

Please enter your Anthropic API key:
(Get one at: https://console.anthropic.com/settings/keys)

API Key: sk-ant-... ████████████

Testing connection... ✅ Valid

✅ AI Assistant ready with Anthropic Claude!
   Configuration saved to repo.config.toml
```

### Smart Status Detection

Different scenarios handled automatically:

```bash
# Scenario 1: Ollama installed but model missing
$ workspace ai "list changesets"
✅ Ollama is running
⚠️  Model 'qwen2.5-coder:7b' not found

Download now? (~4.7GB) [Y/n]: y
[... download progress ...]
✅ Model ready!

# Scenario 2: Ollama installed but not running
$ workspace ai "list changesets"
✅ Ollama is installed
⚠️  Ollama service is not running

Starting Ollama... ✅
Now executing your query...

# Scenario 3: All ready
$ workspace ai "list changesets"
🤖 AI: Let me check your active changesets...
```

---

## ⚙️ Configuration Integration

### Integration with `repo.config.toml`

All AI configuration integrated into the existing workspace configuration file:

```toml
# repo.config.toml (existing file, new [ai] section added)

[ai]
# AI Agent configuration
enabled = true
default_provider = "local"  # "local", "claude", "openai"
auto_fallback = true        # Fallback local → cloud if local fails

[ai.local]
# Ollama configuration (local LLM)
host = "http://localhost:11434"
model = "qwen2.5-coder:7b"
timeout_secs = 30
auto_start = true  # Auto-start Ollama if not running

# Alternative models for different tasks
[ai.local.models]
quick = "codellama:7b"           # Fast responses
analysis = "deepseek-coder:6.7b" # Code analysis
production = "qwen2.5-coder:32b" # Maximum quality (if RAM available)

[ai.cloud]
# Cloud provider configuration
provider = "anthropic"  # "anthropic" or "openai"
model = "claude-3-5-sonnet-20241022"

# API keys from environment variables (security best practice)
api_key_env = "ANTHROPIC_API_KEY"

# Cost controls
[ai.cloud.limits]
max_tokens_per_request = 4000
max_cost_per_day_usd = 5.0
warn_at_cost_usd = 3.0

[ai.cloud.openai]
# OpenAI alternative configuration
model = "gpt-4-turbo-preview"
api_key_env = "OPENAI_API_KEY"

[ai.behavior]
# Operational behavior
require_confirmation = true  # Always confirm before destructive operations
enable_conversation_history = true
max_history_messages = 20

# Logging
log_conversations = true
log_path = ".workspace/ai-logs/"

# Caching (reduces costs and latency)
enable_cache = true
cache_ttl_secs = 3600  # 1 hour

[ai.setup]
# Setup wizard configuration
completed = true
completed_at = "2025-01-15T10:30:00Z"
last_check = "2025-01-15T10:30:00Z"
skip_wizard = false
```

### Environment Variables

```bash
# API Keys (recommended approach - not in config file)
export ANTHROPIC_API_KEY="sk-ant-..."
export OPENAI_API_KEY="sk-..."

# Override default provider
export WORKSPACE_AI_PROVIDER="claude"  # or "local", "openai"

# Override model
export WORKSPACE_AI_MODEL="qwen2.5-coder:32b"

# Disable AI features entirely
export WORKSPACE_AI_ENABLED="false"
```

### Configuration Loading

```rust
pub struct AiConfig {
    pub enabled: bool,
    pub default_provider: ProviderType,
    pub auto_fallback: bool,
    pub local: LocalProviderConfig,
    pub cloud: CloudProviderConfig,
    pub behavior: BehaviorConfig,
    pub setup: SetupConfig,
}

impl AiConfig {
    /// Load from repo.config.toml with environment variable overrides
    pub async fn load(workspace_root: &Path) -> Result<Self> {
        let mut config = Self::load_from_file(workspace_root).await?;
        config.apply_env_overrides()?;
        Ok(config)
    }
    
    async fn load_from_file(workspace_root: &Path) -> Result<Self> {
        // Try different config file formats
        for ext in &["toml", "json", "yaml"] {
            let path = workspace_root.join(format!("repo.config.{}", ext));
            if path.exists() {
                return Self::parse_config_file(&path).await;
            }
        }
        
        // Return defaults if no config found
        Ok(Self::default())
    }
    
    fn apply_env_overrides(&mut self) -> Result<()> {
        if let Ok(enabled) = std::env::var("WORKSPACE_AI_ENABLED") {
            self.enabled = enabled.parse().unwrap_or(true);
        }
        
        if let Ok(provider) = std::env::var("WORKSPACE_AI_PROVIDER") {
            self.default_provider = ProviderType::from_str(&provider)?;
        }
        
        if let Ok(model) = std::env::var("WORKSPACE_AI_MODEL") {
            self.local.model = model;
        }
        
        Ok(())
    }
}
```

---

## 🧩 Component Design

### New Crate: `sublime_ai_agent`

```
crates/ai/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── agent/
│   │   ├── mod.rs           # WorkspaceAgent
│   │   ├── context.rs       # WorkspaceContext
│   │   └── config.rs        # AI configuration
│   ├── tools/
│   │   ├── mod.rs           # ToolRegistry
│   │   ├── changeset.rs     # Changeset operations
│   │   ├── version.rs       # Version/bump operations
│   │   ├── upgrade.rs       # Upgrade operations
│   │   ├── audit.rs         # Audit operations
│   │   ├── git.rs           # Git operations
│   │   └── workspace.rs     # Workspace analysis
│   ├── providers/
│   │   ├── mod.rs
│   │   ├── local.rs         # Ollama provider
│   │   ├── anthropic.rs     # Anthropic Claude
│   │   └── openai.rs        # OpenAI GPT
│   ├── conversation/
│   │   ├── mod.rs
│   │   ├── history.rs       # Conversation history
│   │   └── summarizer.rs    # Context summarization
│   └── error.rs
└── examples/
    └── basic_query.rs
```

### Core Components

#### 1. WorkspaceAgent

```rust
pub struct WorkspaceAgent {
    agent: Agent,                    // Rig Agent
    tool_registry: ToolRegistry,
    context: WorkspaceContext,
    config: AiConfig,
    provider: Box<dyn LlmProvider>,
}

impl WorkspaceAgent {
    pub async fn new(workspace_root: PathBuf) -> Result<Self> {
        let config = AiConfig::load(&workspace_root).await?;
        let context = WorkspaceContext::load(workspace_root, &config).await?;
        
        let provider = match config.default_provider {
            ProviderType::Local => LocalProvider::new(&config.local).await?,
            ProviderType::Anthropic => AnthropicProvider::new(&config.cloud)?,
            ProviderType::OpenAI => OpenAiProvider::new(&config.cloud)?,
        };
        
        let tool_registry = ToolRegistry::new(&context)?;
        let agent = build_rig_agent(provider.as_ref(), &tool_registry)?;
        
        Ok(Self {
            agent,
            tool_registry,
            context,
            config,
            provider,
        })
    }
    
    pub async fn execute_query(&mut self, query: &str) -> Result<Response> {
        // Add query to conversation history
        self.context.add_user_message(query);
        
        // Execute with Rig agent (tool calling enabled)
        let response = self.agent
            .chat(query)
            .with_tools(self.tool_registry.tools())
            .await?;
        
        // Add response to history
        self.context.add_agent_message(&response.content);
        
        Ok(response)
    }
}
```

#### 2. Tool Registry

```rust
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn AgentTool>>,
}

#[async_trait]
pub trait AgentTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value;  // JSON Schema
    async fn execute(&self, params: serde_json::Value) -> Result<ToolResult>;
}

pub enum AgentToolType {
    // Changeset operations
    CreateChangeset,
    UpdateChangeset,
    ListChangesets,
    ShowChangeset,
    
    // Version operations
    PreviewBump,
    ExecuteBump,
    GetVersionInfo,
    
    // Upgrade operations
    CheckUpgrades,
    ApplyUpgrades,
    
    // Audit operations
    RunAudit,
    GetHealthScore,
    
    // Analysis operations
    AnalyzeChanges,
    DetectAffectedPackages,
    GetDependencyGraph,
    
    // Git operations
    GetBranchInfo,
    GetCommitHistory,
    CompareRefs,
    
    // Workspace operations
    DetectProjectType,
    ListPackages,
    GetPackageInfo,
}

impl ToolRegistry {
    pub fn new(context: &WorkspaceContext) -> Result<Self> {
        let mut tools = HashMap::new();
        
        // Register all tools
        tools.insert(
            "create_changeset".to_string(),
            Box::new(CreateChangesetTool::new(context)?) as Box<dyn AgentTool>
        );
        
        // ... register other tools
        
        Ok(Self { tools })
    }
    
    pub fn tools(&self) -> &HashMap<String, Box<dyn AgentTool>> {
        &self.tools
    }
}
```

#### 3. Example Tool Implementation

```rust
pub struct CreateChangesetTool {
    manager: ChangesetManager,
    git_repo: Option<Repo>,
}

impl CreateChangesetTool {
    pub fn new(context: &WorkspaceContext) -> Result<Self> {
        let manager = ChangesetManager::new(
            context.root.clone(),
            FileSystemManager::new(),
            context.config.clone(),
        ).await?;
        
        Ok(Self {
            manager,
            git_repo: context.git_repo.clone(),
        })
    }
}

#[async_trait]
impl AgentTool for CreateChangesetTool {
    fn name(&self) -> &str {
        "create_changeset"
    }
    
    fn description(&self) -> &str {
        "Creates a new changeset for tracking package version changes"
    }
    
    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "branch": {
                    "type": "string",
                    "description": "Branch name (defaults to current branch)"
                },
                "bump": {
                    "type": "string",
                    "enum": ["major", "minor", "patch"],
                    "description": "Version bump type"
                },
                "environments": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Target environments"
                },
                "packages": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Packages to include (auto-detected if empty)"
                }
            },
            "required": ["bump", "environments"]
        })
    }
    
    async fn execute(&self, params: serde_json::Value) -> Result<ToolResult> {
        let branch = params["branch"]
            .as_str()
            .map(String::from)
            .or_else(|| self.git_repo.as_ref()?.get_current_branch().ok())?;
        
        let bump = VersionBump::from_str(params["bump"].as_str()?)?;
        
        let environments: Vec<String> = params["environments"]
            .as_array()?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        
        // Create changeset
        let changeset = self.manager
            .create(&branch, bump, environments)
            .await?;
        
        // Auto-detect packages if not provided
        if params["packages"].is_null() || params["packages"].as_array()?.is_empty() {
            if let Some(repo) = &self.git_repo {
                let detector = PackageDetector::new(...).await?;
                let affected = detector
                    .detect_from_branch(repo, &branch, "main")
                    .await?;
                
                for package in affected {
                    changeset.add_package(&package);
                }
                
                self.manager.update(&changeset).await?;
            }
        }
        
        Ok(ToolResult::Changeset {
            id: changeset.id.clone(),
            branch: changeset.branch.clone(),
            bump: changeset.bump,
            packages: changeset.packages.clone(),
        })
    }
}
```

#### 4. Workspace Context

```rust
pub struct WorkspaceContext {
    pub root: PathBuf,
    pub config: PackageToolsConfig,
    pub project_info: Option<ProjectDescriptor>,
    pub git_repo: Option<Repo>,
    pub conversation_history: Vec<Message>,
}

impl WorkspaceContext {
    pub async fn load(
        root: PathBuf,
        ai_config: &AiConfig,
    ) -> Result<Self> {
        let config = PackageToolsConfig::load(&root).await?;
        
        let project_info = ProjectDetector::new()
            .detect(&root, None)
            .await
            .ok();
        
        let git_repo = Repo::open(root.to_str().unwrap()).ok();
        
        Ok(Self {
            root,
            config,
            project_info,
            git_repo,
            conversation_history: Vec::new(),
        })
    }
    
    pub fn add_user_message(&mut self, content: &str) {
        self.conversation_history.push(Message::User {
            content: content.to_string(),
            timestamp: Utc::now(),
        });
    }
    
    pub fn add_agent_message(&mut self, content: &str) {
        self.conversation_history.push(Message::Agent {
            content: content.to_string(),
            timestamp: Utc::now(),
        });
    }
}
```

---

## 💬 Use Cases & Examples

### Example 1: Intelligent Changeset Creation

```bash
$ workspace ai "I just finished working on authentication, create a changeset for production"

🤖 AI Agent: I'll help you create a changeset for your authentication work.

   Analyzing changes...
   ✓ Detected changes in: packages/auth, packages/api
   ✓ Current branch: feature/auth-improvements
   ✓ Found 12 commits since divergence from main
   ✓ Commit types detected: feat (8), fix (3), refactor (1)
   
   Based on the commits, I recommend:
   - Bump type: MINOR (new features detected from conventional commits)
   - Packages: auth, api (detected from file changes)
   - Environments: production
   
   Creating changeset...
   ✓ Created changeset: feature-auth-improvements.json
   
   Summary:
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   ID:          cs_abc123
   Branch:      feature/auth-improvements
   Bump:        minor
   Packages:    @workspace/auth, @workspace/api
   Commits:     12
   Environments: production
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   
   Next steps:
   • Continue development and commit changes
   • Run 'workspace ai "preview version bump"' to see impact
   • When ready: merge to main to trigger release
```

### Example 2: Dependency Impact Analysis

```bash
$ workspace ai "if I bump the auth package, what else will be affected?"

🤖 AI Agent: Analyzing dependency impact for @workspace/auth...

   Building dependency graph...
   
   @workspace/auth (current: 2.1.0 → proposed: 2.2.0)
   │
   ├─ Direct dependents (will receive automatic updates):
   │  ├─ @workspace/api (1.5.0 → 1.5.1) [uses workspace:*]
   │  ├─ @workspace/web (3.2.1 → 3.2.2) [uses workspace:*]
   │  └─ @workspace/mobile (2.0.0 → 2.0.1) [uses workspace:*]
   │
   ├─ Transitive dependents (indirectly affected):
   │  ├─ @workspace/admin-panel (depends on api)
   │  │  └─ Version: 1.1.0 → 1.1.1
   │  └─ @workspace/customer-portal (depends on web)
   │     └─ Version: 2.3.0 → 2.3.1
   │
   └─ Impact Summary:
      Total packages affected: 5
      Direct updates: 3
      Transitive updates: 2
      Propagation depth: 2 levels
   
   Configuration used:
   • Propagation enabled: Yes
   • Propagation bump: patch
   • Max depth: 5
   
   Would you like me to:
   1. Show full version resolution preview
   2. Create a dry-run bump report
   3. Generate dependency graph visualization
   
   Choice: _
```

### Example 3: Upgrade Safety Analysis

```bash
$ workspace ai "what dependencies need updating and are they safe to upgrade?"

🤖 AI Agent: Let me analyze your dependencies for available upgrades...

   Scanning workspace packages...
   Checking registry for updates...
   
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   UPGRADE ANALYSIS REPORT
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   
   📦 SAFE TO UPGRADE (Patch versions - 12 packages)
   
   High Priority (Security fixes):
   • lodash: 4.17.20 → 4.17.21 
     └─ CVE-2020-28500 (prototype pollution)
   • axios: 1.6.0 → 1.6.7
     └─ Security patch for header parsing
   
   Low Risk:
   • react-dom: 18.2.0 → 18.2.1 (bug fixes)
   • typescript: 5.3.2 → 5.3.3 (compiler fixes)
   ... (8 more)
   
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   
   ⚠️  REVIEW RECOMMENDED (Minor versions - 8 packages)
   
   • react: 18.2.0 → 18.3.0
     └─ New features: useOptimistic hook, Form actions
     └─ Breaking: None documented
     └─ Recommendation: Low risk, test forms
     
   • typescript: 5.3.3 → 5.4.0
     └─ New features: NoInfer utility type, Object.groupBy
     └─ Breaking: Stricter checks may reveal existing issues
     └─ Recommendation: Test build, review new errors
     
   ... (6 more)
   
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   
   🚨 MAJOR VERSIONS (Breaking changes - 3 packages)
   
   • vite: 4.5.0 → 5.1.0
     └─ Breaking: Requires Node 18+, config changes
     └─ Migration guide: https://vitejs.dev/guide/migration
     └─ Recommendation: Plan separate upgrade, test thoroughly
     
   • eslint: 8.57.0 → 9.0.0
     └─ Breaking: Flat config required, some plugins incompatible
     └─ Recommendation: Defer until plugins updated
     
   • vitest: 1.6.0 → 2.0.0
     └─ Breaking: API changes, new test syntax
     └─ Recommendation: Review changelog, update tests
   
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   
   RECOMMENDATIONS:
   
   1. Apply patch upgrades now (especially security fixes)
      → Command: workspace upgrade apply --patch-only
   
   2. Review and test minor upgrades individually
      → Start with: react, typescript
   
   3. Plan major upgrades separately
      → Create separate branches for vite, eslint migrations
   
   Would you like me to:
   A) Apply all patch updates now
   B) Show details for specific upgrade
   C) Create upgrade plan document
   D) Generate changelog for upgrades
   
   Choice: _
```

### Example 4: Release Preparation

```bash
$ workspace ai "prepare everything for a production release"

🤖 AI Agent: I'll help you prepare for a production release.

   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   PRE-FLIGHT CHECKS
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   
   Repository Status:
   ✅ Current branch: main
   ✅ Branch is up to date with origin/main
   ✅ Working directory clean (no uncommitted changes)
   ✅ No merge conflicts detected
   
   Changeset Status:
   ✅ Active changesets: 3 found
   ✅ All changesets validated
   ✅ Environments: production (all)
   
   Build Status:
   ✅ Last CI build: passed (10 minutes ago)
   ✅ All tests passing
   ✅ No pending review comments
   
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   RELEASE PLAN
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   
   1. Version Bumps (from 3 changesets):
   
   Package                 Current    New        Bump    Reason
   ─────────────────────────────────────────────────────────────
   @workspace/auth         2.1.0   →  2.2.0     minor   New features
   @workspace/api          1.5.0   →  1.6.0     minor   Propagated
   @workspace/web          3.2.1   →  3.3.0     minor   Propagated
   @workspace/mobile       2.0.0   →  2.0.1     patch   Propagated
   @workspace/admin-panel  1.1.0   →  1.1.1     patch   Propagated
   
   2. Changelog Updates:
   
   ✅ Will update CHANGELOG.md for 5 packages
   ✅ Format: Conventional Commits
   ✅ Commits to include: 34 commits since last release
   ✅ Breaking changes: None detected
   
   3. Git Operations:
   
   ✅ Create version tags:
      • auth@2.2.0
      • api@1.6.0
      • web@3.3.0
      • mobile@2.0.1
      • admin-panel@1.1.1
   
   ✅ Create release commit:
      Message: "chore(release): version bump for production"
   
   ✅ Push to origin with tags
   
   4. Changeset Management:
   
   ✅ Archive 3 active changesets
   ✅ Store in: .changesets/history/2025-01-15/
   
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   
   Estimated execution time: ~45 seconds
   
   This will:
   • Modify 5 package.json files
   • Update 5 CHANGELOG.md files
   • Create 5 Git tags
   • Create 1 commit
   • Archive 3 changesets
   • Push to remote
   
   ⚠️  This is a production release. Please review carefully.
   
   Execute release workflow? [y/N]: y
   
   Executing release...
   
   [1/4] Bumping versions... ✅
   [2/4] Updating changelogs... ✅
   [3/4] Creating Git tags and commit... ✅
   [4/4] Archiving changesets... ✅
   
   Pushing to origin... ✅
   
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   ✨ RELEASE COMPLETE
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   
   Published versions:
   • @workspace/auth@2.2.0
   • @workspace/api@1.6.0
   • @workspace/web@3.3.0
   • @workspace/mobile@2.0.1
   • @workspace/admin-panel@1.1.1
   
   Next steps:
   • Monitor CI/CD pipeline
   • Verify deployment to production
   • Update release notes if needed
```

### Example 5: Troubleshooting

```bash
$ workspace ai "why is my bump command failing?"

🤖 AI Agent: Let me diagnose the issue...

   Running diagnostics...
   
   Checking configuration... ✅
   Checking Git repository... ✅
   Checking active changesets... ✅
   Analyzing dependency graph... ❌
   
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   ISSUE DETECTED: Circular Dependency
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   
   The version resolution is failing due to a circular dependency
   in your workspace packages:
   
   Cycle detected:
   @workspace/utils
      ↓ depends on
   @workspace/core
      ↓ depends on
   @workspace/utils  ← creates infinite loop!
   
   This prevents the dependency propagation algorithm from determining
   the correct order of version updates.
   
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   RESOLUTION OPTIONS
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   
   Option 1: Refactor to Break Cycle (Recommended)
   
   Create a new shared package to break the dependency:
   
   @workspace/shared
      ↑                ↑
   @workspace/utils   @workspace/core
   
   Steps:
   1. Create new package: @workspace/shared
   2. Move common code from utils/core to shared
   3. Update utils and core to depend on shared
   
   Would you like me to generate a refactoring plan?
   
   ───────────────────────────────────────────────────────────
   
   Option 2: Configure to Allow Circular Dependencies
   
   Update repo.config.toml:
   
   [dependency]
   fail_on_circular = false
   
   ⚠️  Warning: This may cause:
   • Unpredictable version resolution
   • Potential version mismatches
   • Build order issues
   
   Not recommended for production.
   
   ───────────────────────────────────────────────────────────
   
   Option 3: Skip Dependency Propagation
   
   Run bump with --no-propagation flag:
   
   workspace bump --execute --no-propagation
   
   ⚠️  Warning: You'll need to manually update dependent packages.
   
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   
   Which option would you like?
   A) Generate refactoring plan (recommended)
   B) Update config to allow circular dependencies
   C) Show me how to skip propagation
   D) Show full dependency graph
   
   Choice: _
```

### Example 6: Chat Mode

```bash
$ workspace ai

🤖 Workspace AI Assistant

I can help you with:
• Creating and managing changesets
• Version bumping and releases
• Dependency upgrades
• Project health audits
• Git operations and history
• Workspace analysis

Type 'help' for examples, 'exit' to quit.

> what changesets exist?

You have 2 active changesets:

1. feature/auth-improvements (minor)
   • Packages: auth, api
   • Commits: 12
   • Created: 2 days ago

2. feature/ui-redesign (major)
   • Packages: web, mobile
   • Commits: 28
   • Created: 1 week ago

> show me the auth one in detail

Changeset: feature/auth-improvements
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

ID:           cs_abc123
Branch:       feature/auth-improvements
Bump:         minor
Environments: production
Packages:     @workspace/auth, @workspace/api

Commits (12):
• abc1234 - feat: add OAuth2 support
• def5678 - feat: implement JWT refresh tokens
• ghi9012 - fix: handle token expiration
... (9 more)

Created:  2025-01-13 14:30:00
Updated:  2025-01-15 09:15:00

> what would happen if I bump this?

Let me preview the version bump...

@workspace/auth: 2.1.0 → 2.2.0 (minor)

Dependency propagation:
• @workspace/api: 1.5.0 → 1.5.1 (patch, depends on auth)
• @workspace/web: 3.2.1 → 3.2.2 (patch, depends on api)

Total packages affected: 3
Changelogs will be updated for all 3 packages

> exit

Goodbye! 👋
```

---

## 📅 Implementation Roadmap

### Phase 1: Foundation (MVP) - Weeks 1-3

**Goal**: Basic AI agent working with Ollama local provider

**Deliverables:**
- ✅ New crate `sublime_ai_agent` scaffolding
- ✅ Basic Rig integration with Ollama
- ✅ 5 essential tools:
  - `create_changeset`
  - `list_changesets`
  - `show_changeset`
  - `preview_bump`
  - `get_project_info`
- ✅ CLI command `workspace ai <query>` (single-shot mode)
- ✅ Configuration integration in `repo.config.toml`
- ✅ Basic setup wizard (manual instructions)

**Success Criteria:**
```bash
$ workspace ai "create a minor changeset for production"
# Works and creates changeset correctly
```

### Phase 2: Setup & Expansion - Weeks 4-6

**Goal**: Automated setup and expanded tool coverage

**Deliverables:**
- ✅ Guided installation wizard (Level 2 setup)
- ✅ Auto-detection and smart status checking
- ✅ Model download with progress bars
- ✅ 15+ additional tools covering:
  - Version operations (bump, rollback)
  - Upgrade operations (check, apply)
  - Audit operations (run, analyze)
  - Git operations (history, compare)
  - Workspace operations (detect, analyze)
- ✅ Conversation history management
- ✅ Error handling and fallbacks

**Success Criteria:**
```bash
$ workspace ai "analyze my project health"
# Comprehensive analysis using multiple tools
```

### Phase 3: Cloud & Intelligence - Weeks 7-9

**Goal**: Cloud providers and advanced features

**Deliverables:**
- ✅ Anthropic Claude integration
- ✅ OpenAI GPT-4 integration
- ✅ Smart provider routing (local vs cloud)
- ✅ Cost tracking and limits
- ✅ Interactive chat mode (`workspace ai` with no query)
- ✅ Context summarization for long conversations
- ✅ Caching layer for common queries
- ✅ JSON output mode for CI/CD

**Success Criteria:**
```bash
$ workspace ai
> help me prepare for release
# Multi-turn conversation with context awareness
```

### Phase 4: Production Ready - Weeks 10-12

**Goal**: Polish, optimization, and documentation

**Deliverables:**
- ✅ Comprehensive error messages and recovery
- ✅ Performance optimization (tool execution, caching)
- ✅ Security audit (input validation, injection prevention)
- ✅ Telemetry (optional, privacy-first)
- ✅ Complete documentation:
  - User guide
  - Tool development guide
  - Configuration reference
  - Examples and recipes
- ✅ CI/CD integration examples
- ✅ Video demos

**Success Criteria:**
- 95%+ of common operations work via AI
- <2s response time for simple queries
- <10s for complex analysis
- Zero security vulnerabilities
- 100% documentation coverage

### Timeline Summary

```
Week 1-3:   Phase 1 - MVP Foundation
Week 4-6:   Phase 2 - Setup & Expansion
Week 7-9:   Phase 3 - Cloud & Intelligence
Week 10-12: Phase 4 - Production Ready
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total: 12 weeks (~3 months)
```

### Post-Launch (Continuous)

- 🔄 Model updates (new Ollama models)
- 🔄 New tools as crate APIs expand
- 🔄 User feedback integration
- 🔄 Performance optimizations
- 🔄 Advanced features:
  - Proactive suggestions
  - Learning from project patterns
  - Workflow templates
  - Multi-project context

---

## 🔒 Technical Considerations

### Security

**Input Validation**
```rust
// Sanitize user input before sending to LLM
fn sanitize_input(input: &str) -> String {
    // Remove potential injection attempts
    // Limit length
    // Escape special characters
}

// Validate tool parameters
fn validate_tool_params(params: &Value, schema: &Value) -> Result<()> {
    // JSON schema validation
    // Type checking
    // Range validation
}
```

**Confirmation for Destructive Operations**
```rust
async fn execute_tool(tool: &dyn AgentTool, params: Value) -> Result<ToolResult> {
    if tool.is_destructive() {
        // Always confirm before executing
        let preview = tool.preview(params.clone()).await?;
        if !confirm_with_preview(preview)? {
            return Err(Error::UserCancelled);
        }
    }
    
    tool.execute(params).await
}
```

**API Key Protection**
- Never log API keys
- Store in environment variables only
- Validate key format before use
- Rotate keys regularly (documented)

### Performance

**Caching Strategy**
```rust
pub struct ResponseCache {
    cache: HashMap<CacheKey, CachedResponse>,
    ttl: Duration,
}

#[derive(Hash, Eq, PartialEq)]
struct CacheKey {
    query_hash: String,
    context_hash: String,
}

impl ResponseCache {
    pub fn get(&self, query: &str, context: &Context) -> Option<&CachedResponse> {
        let key = self.compute_key(query, context);
        self.cache.get(&key).filter(|r| !r.is_expired())
    }
    
    pub fn set(&mut self, query: &str, context: &Context, response: Response) {
        let key = self.compute_key(query, context);
        self.cache.insert(key, CachedResponse {
            response,
            cached_at: Utc::now(),
        });
    }
}
```

**Async/Concurrent Tool Execution**
```rust
// Execute independent tools concurrently
async fn execute_tools_parallel(tools: Vec<ToolCall>) -> Result<Vec<ToolResult>> {
    let futures: Vec<_> = tools
        .into_iter()
        .map(|call| async move {
            let tool = registry.get(&call.name)?;
            tool.execute(call.params).await
        })
        .collect();
    
    futures::future::try_join_all(futures).await
}
```

**Response Streaming**
```rust
// Stream LLM responses for better UX
async fn stream_response(agent: &Agent, query: &str) -> impl Stream<Item = String> {
    agent
        .chat_stream(query)
        .await
        .map(|chunk| chunk.content)
}
```

### Context Management

**Conversation History Limits**
```rust
const MAX_HISTORY_MESSAGES: usize = 20;
const MAX_CONTEXT_TOKENS: usize = 8000;

impl WorkspaceContext {
    fn truncate_history(&mut self) {
        if self.conversation_history.len() > MAX_HISTORY_MESSAGES {
            // Keep system message + recent messages
            let to_remove = self.conversation_history.len() - MAX_HISTORY_MESSAGES;
            self.conversation_history.drain(1..to_remove + 1);
        }
    }
    
    async fn summarize_if_needed(&mut self) {
        let total_tokens = estimate_tokens(&self.conversation_history);
        
        if total_tokens > MAX_CONTEXT_TOKENS {
            // Summarize old messages
            let summary = self.summarizer
                .summarize(&self.conversation_history[1..10])
                .await?;
            
            // Replace old messages with summary
            self.conversation_history.drain(1..10);
            self.conversation_history.insert(1, Message::Summary(summary));
        }
    }
}
```

### Error Handling

**Graceful Degradation**
```rust
async fn execute_with_fallback(query: &str) -> Result<Response> {
    // Try local provider first
    match local_provider.execute(query).await {
        Ok(response) => Ok(response),
        Err(e) if e.is_model_not_available() => {
            // Fallback to cloud if configured
            if let Some(cloud) = &cloud_provider {
                warn!("Local model unavailable, using cloud provider");
                cloud.execute(query).await
            } else {
                // Fallback to template responses
                template_response(query)
            }
        }
        Err(e) => Err(e),
    }
}
```

**User-Friendly Error Messages**
```rust
pub enum AiError {
    ModelNotAvailable {
        model: String,
        suggestion: String,
    },
    ApiKeyInvalid {
        provider: String,
        help_url: String,
    },
    NetworkError {
        operation: String,
        retry_in: Duration,
    },
    // ...
}

impl Display for AiError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::ModelNotAvailable { model, suggestion } => {
                write!(f, "Model '{}' not available.\n\n{}", model, suggestion)
            }
            Self::ApiKeyInvalid { provider, help_url } => {
                write!(f,
                    "Invalid API key for {}.\n\
                     Get a valid key at: {}",
                    provider, help_url
                )
            }
            // ...
        }
    }
}
```

### Cost Controls

**Budget Tracking**
```rust
pub struct CostTracker {
    daily_usage: HashMap<Date, f64>,
    limits: CostLimits,
}

impl CostTracker {
    pub async fn check_before_request(
        &self,
        estimated_cost: f64,
    ) -> Result<(), CostLimitError> {
        let today = Utc::now().date_naive();
        let current = self.daily_usage.get(&today).copied().unwrap_or(0.0);
        
        if current + estimated_cost > self.limits.max_cost_per_day {
            return Err(CostLimitError::DailyLimitReached {
                current,
                limit: self.limits.max_cost_per_day,
            });
        }
        
        if current + estimated_cost > self.limits.warn_at_cost {
            warn!(
                "Approaching daily cost limit: ${:.2} / ${:.2}",
                current + estimated_cost,
                self.limits.max_cost_per_day
            );
        }
        
        Ok(())
    }
    
    pub fn record_usage(&mut self, cost: f64) {
        let today = Utc::now().date_naive();
        *self.daily_usage.entry(today).or_insert(0.0) += cost;
    }
}
```

---

## ✅ Key Decisions

### 1. Default Provider: Ollama (Local)

**Decision**: Use Ollama with `qwen2.5-coder:7b` as the default provider

**Rationale**:
- Zero cost for users
- Complete privacy (all local)
- Works offline
- Good quality for most use cases
- Easy to upgrade to cloud when needed

**Trade-offs**:
- Requires initial setup (~5-10 minutes)
- Slightly lower quality than cloud models
- Requires 8GB+ RAM

### 2. Premium Option: Anthropic Claude 3.5 Sonnet

**Decision**: Anthropic Claude as the primary cloud provider

**Rationale**:
- Superior code understanding and technical reasoning
- 200K context window (best for large projects)
- Excellent instruction following
- Detailed, structured responses
- Competitive pricing

**Alternative**: OpenAI GPT-4 also supported

### 3. Setup Strategy: Guided Installation

**Decision**: Implement guided wizard with auto-installation option

**Rationale**:
- Best UX - works "out of the box" with confirmation
- Cross-platform support (Linux, macOS, Windows)
- Safe - user confirms before installing
- Fallbacks for manual installation and cloud

**Alternatives Considered**:
- ❌ Manual only - poor UX, high friction
- ❌ Fully automatic - security concerns, less control

### 4. Configuration: Integrated in `repo.config.toml`

**Decision**: Add `[ai]` section to existing `repo.config.toml`

**Rationale**:
- Single source of truth for all workspace config
- Already familiar to users
- Supports multiple formats (TOML, JSON, YAML)
- Can be committed to repo for team consistency

**Alternatives Considered**:
- ❌ Separate AI config file - fragmentation
- ❌ Global config only - not project-specific

### 5. Architecture: Rig Library

**Decision**: Use [Rig](https://rig.rs/) for LLM abstraction

**Rationale**:
- Native Rust library (no FFI overhead)
- Provider-agnostic (Ollama, Anthropic, OpenAI)
- Built-in tool calling support
- Active development (2025)
- Clean, idiomatic Rust API

### 6. Tool Execution: Direct API Calls

**Decision**: Tools call crate APIs directly (not CLI commands)

**Rationale**:
- Better performance (no subprocess overhead)
- Type safety
- Direct access to rich return types
- Better error handling
- Easier to test

### 7. Mode of Operation: Hybrid (Single-shot + Chat)

**Decision**: Support both `workspace ai "query"` and `workspace ai` (chat mode)

**Rationale**:
- Single-shot for quick queries and CI/CD
- Chat mode for complex, multi-step tasks
- Flexibility for different user preferences

### 8. Cost Strategy: Hybrid with Controls

**Decision**: Default to free local, optional cloud with cost limits

**Rationale**:
- 90% of queries work well with local models
- 10% complex queries benefit from cloud quality
- Cost controls prevent surprise bills
- Transparent to user (always show which provider used)

### 9. Security: Confirm Before Destruction

**Decision**: Always require confirmation for destructive operations

**Rationale**:
- Safety first - prevent accidental data loss
- Clear preview of what will change
- User maintains control
- Can be overridden with `--force` flag for automation

### 10. Rollout: Phased Implementation

**Decision**: 4-phase rollout over 12 weeks

**Rationale**:
- MVP validates concept quickly (3 weeks)
- Each phase adds value incrementally
- Time for user feedback between phases
- Reduces risk of large rewrites

---

## 🎓 Learning Resources

### Rig Library
- **Documentation**: https://docs.rig.rs/
- **GitHub**: https://github.com/0xPlaygrounds/rig
- **Tutorial**: Building an arXiv Agent with Rig (January 2025)

### Ollama
- **Official Site**: https://ollama.com/
- **Models Library**: https://ollama.com/library
- **API Docs**: https://github.com/ollama/ollama/blob/main/docs/api.md

### LLM Models
- **Qwen2.5-Coder**: https://ollama.com/library/qwen2.5-coder
- **DeepSeek-Coder**: https://ollama.com/library/deepseek-coder
- **CodeLlama**: https://ollama.com/library/codellama

### Cloud Providers
- **Anthropic Claude**: https://console.anthropic.com/
- **OpenAI**: https://platform.openai.com/

---

## 📝 Appendix

### Estimated Costs (3-Month Development)

**Development Resources**:
- Developer time: ~480 hours (12 weeks × 40 hours)
- Cloud API testing: ~$100 (testing with real APIs)
- **Total**: ~$100 out-of-pocket (dev time excluded)

**Post-Launch User Costs**:
- Local (default): $0/month
- Hybrid (recommended): $2-5/month
- Cloud-only: $20-50/month (heavy usage)

### Success Metrics

**Phase 1 (MVP)**:
- ✅ 5+ tools working
- ✅ 100% of simple queries work
- ✅ <5s response time

**Phase 2 (Expansion)**:
- ✅ 20+ tools working
- ✅ 90% of queries work without cloud
- ✅ Setup wizard <10 minutes

**Phase 3 (Cloud)**:
- ✅ Multi-provider support
- ✅ Chat mode functional
- ✅ Cost tracking accurate

**Phase 4 (Production)**:
- ✅ 95%+ success rate
- ✅ <2s simple queries, <10s complex
- ✅ Zero critical bugs
- ✅ Complete documentation

### Future Enhancements (Post-MVP)

**Advanced Intelligence**:
- Learn from project patterns
- Proactive suggestions ("You usually create a changeset after this")
- Workflow templates (common multi-step operations)
- Multi-project context awareness

**Integrations**:
- GitHub Actions integration
- GitLab CI integration
- VSCode extension
- Web UI (optional)

**Observability**:
- Usage analytics (privacy-first)
- Performance monitoring
- Cost optimization suggestions
- Quality improvements from feedback

---

## 📚 References

This research document compiled from:
- Rig library documentation and examples (January 2025)
- Ollama model benchmarks (2025)
- Anthropic Claude API documentation
- OpenAI GPT-4 API documentation
- Community best practices for CLI AI integration
- Rust async patterns and performance optimization
- Existing workspace-node-tools codebase analysis

**Last Updated**: January 2025  
**Status**: Research Complete - Ready for Implementation  
**Next Step**: Review and approve implementation plan
