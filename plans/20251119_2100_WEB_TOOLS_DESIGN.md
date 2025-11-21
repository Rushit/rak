# Phase 8.3: Web & Search Tools - Design Discussion

**Date**: 2025-11-19 21:00  
**Purpose**: Analyze Python RAK's web tools and discuss implementation options for Rust

## Overview

We need to implement Phase 8.3: Web & Search Tools. This document analyzes how Python RAK implements these features and proposes implementation options for Rust RAK.

---

## Python RAK's Approach

### 1. Google Search Tool

Python RAK has **THREE** different implementations:

#### Option A: `google_search_tool.py` - Gemini Built-in Tool ⭐
```python
class GoogleSearchTool(BaseTool):
    """A built-in tool that is automatically invoked by Gemini 2 models."""
    
    async def process_llm_request(self, *, tool_context, llm_request):
        # Adds Google Search to Gemini API request
        llm_request.config.tools.append(
            types.Tool(google_search=types.GoogleSearch())
        )
```

**How it works**:
- **NO local execution** - search happens inside Gemini API
- Just adds `google_search` to the model config
- Model calls Google Search internally
- Returns results automatically
- **Pros**: Zero API keys needed, zero implementation
- **Cons**: Only works with Gemini 2.0+

#### Option B: `google_search_agent_tool.py` - Sub-agent Wrapper
```python
def create_google_search_agent(model) -> LlmAgent:
    """Create a sub-agent that only uses google_search tool."""
    return LlmAgent(
        name='google_search_agent',
        model=model,
        tools=[google_search],  # Uses Option A internally
    )

class GoogleSearchAgentTool(AgentTool):
    """Wraps google_search agent to use with other tools."""
```

**How it works**:
- Wraps Option A in a sub-agent
- Workaround for Gemini 1.x limitation (can't mix google_search with other tools)
- Delegates to sub-agent that only has google_search
- **Pros**: Works around API limitations
- **Cons**: Complex, adds overhead

#### Option C: `discovery_engine_search_tool.py` - Google Cloud API
```python
class DiscoveryEngineSearchTool(FunctionTool):
    """Search using Google Cloud Discovery Engine API."""
    
    async def run_async(self, args, tool_context):
        # Calls Discovery Engine REST API
        # Requires: project_id, location, data_store_id
        # Returns: Search results from indexed data
```

**How it works**:
- Uses Google Cloud Discovery Engine (formerly Enterprise Search)
- Requires GCP project + credentials
- Searches your indexed data stores
- **Pros**: Search your own data
- **Cons**: Requires GCP setup, costs money

#### Option D: Custom Search API (NOT in Python RAK!)
This is what most people expect - Google Custom Search JSON API:
```python
# Not in Python RAK, but common approach
response = requests.get(
    "https://www.googleapis.com/customsearch/v1",
    params={
        "key": api_key,
        "cx": search_engine_id,
        "q": query,
    }
)
```

**How it works**:
- Uses Google Custom Search API
- Requires API key + Search Engine ID
- 100 free queries/day, then paid
- **Pros**: Simple, universal
- **Cons**: Requires setup, limited free tier

---

### 2. URL Context Tool

#### Python's Implementation: `url_context_tool.py` - Gemini Built-in
```python
class UrlContextTool(BaseTool):
    """Gemini 2 model automatically retrieves content from URLs."""
    
    async def process_llm_request(self, *, tool_context, llm_request):
        llm_request.config.tools.append(
            types.Tool(url_context=types.UrlContext())
        )
```

**How it works**:
- **NO local execution** - Gemini fetches URLs internally
- Just enables the capability in model config
- Model fetches and processes URLs automatically
- **Only works with Gemini 2.0+**

---

### 3. Web Scraping Tool

#### Python's Implementation: `load_web_page.py` - Simple Function
```python
def load_web_page(url: str) -> str:
    """Fetches content from URL and returns text."""
    response = requests.get(url)
    soup = BeautifulSoup(response.content, 'lxml')
    text = soup.get_text(separator='\n', strip=True)
    
    # Filter out short lines (< 3 words)
    return '\n'.join(
        line for line in text.splitlines() 
        if len(line.split()) > 3
    )
```

**How it works**:
- Simple HTTP fetch with `requests`
- HTML parsing with BeautifulSoup
- Extracts all text, filters noise
- **32 lines of code total**
- Very simple, no advanced features

---

## Implementation Options for Rust RAK

### Option 1: Gemini Built-in Tools (Easiest) ⭐

**Implement**:
- `GoogleSearchTool` - Adds `google_search` to Gemini config
- `UrlContextTool` - Adds `url_context` to Gemini config

**How**:
```rust
pub struct GoogleSearchTool;

impl GoogleSearchTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GoogleSearchTool {
    fn name(&self) -> &str { "google_search" }
    
    fn description(&self) -> &str {
        "Search the web using Google (Gemini built-in)"
    }
    
    // Key: This tool modifies LLM config, not local execution
    fn is_model_native(&self) -> bool { true }
    
    async fn execute(&self, ctx, params) -> Result<ToolResponse> {
        // This never actually executes locally
        // Model handles it internally
        Ok(ToolResponse::model_native())
    }
}
```

**Pros**:
- ✅ Extremely simple (50-100 lines each)
- ✅ No API keys needed
- ✅ No HTTP client code
- ✅ Matches Python RAK exactly
- ✅ Works perfectly with Gemini 2.0+

**Cons**:
- ❌ Only works with Gemini 2.0+ models
- ❌ Doesn't work with Claude, GPT, etc.
- ❌ Can't customize behavior
- ❌ Relies on Google's implementation

**Implementation Time**: 2-3 hours

---

### Option 2: Custom Search API (Universal)

**Implement**:
- `GoogleCustomSearchTool` - Uses Google Custom Search JSON API
- `WebScraperTool` - HTTP + HTML parsing with scraper crate
- `UrlLoaderTool` - Simple URL content fetcher

**How**:
```rust
pub struct GoogleCustomSearchTool {
    api_key: String,
    search_engine_id: String,
    client: reqwest::Client,
}

impl GoogleCustomSearchTool {
    async fn search(&self, query: &str) -> Result<SearchResults> {
        let response = self.client
            .get("https://www.googleapis.com/customsearch/v1")
            .query(&[
                ("key", &self.api_key),
                ("cx", &self.search_engine_id),
                ("q", query),
            ])
            .send()
            .await?;
            
        Ok(response.json().await?)
    }
}
```

**Pros**:
- ✅ Works with ANY model (Claude, GPT, Gemini, etc.)
- ✅ Full control over implementation
- ✅ Can add features (filters, pagination, etc.)
- ✅ More portable

**Cons**:
- ❌ Requires API key + Search Engine ID setup
- ❌ 100 queries/day free limit
- ❌ More code to maintain (~300 lines each)
- ❌ Need to handle rate limiting, errors, etc.

**Implementation Time**: 1 week

---

### Option 3: Hybrid Approach (Best of Both) ⭐⭐

**Implement BOTH**:
1. Gemini built-in tools (Option 1)
2. Custom API tools (Option 2)

**Structure**:
```rust
// Gemini built-in (simple)
pub struct GeminiGoogleSearchTool;
pub struct GeminiUrlContextTool;

// Universal API-based (full-featured)
pub struct GoogleCustomSearchTool;
pub struct WebScraperTool;
pub struct UrlLoaderTool;
```

**Usage**:
```rust
// With Gemini 2.0+ - use built-in (zero config)
let search = GeminiGoogleSearchTool::new();

// With other models - use custom API
let search = GoogleCustomSearchTool::new(api_key, engine_id)?;

// Web scraping works with all models
let scraper = WebScraperTool::new()?;
```

**Pros**:
- ✅ Best UX for Gemini users (zero config)
- ✅ Universal support for all models
- ✅ Flexibility
- ✅ Users choose based on needs

**Cons**:
- ❌ More code to maintain
- ❌ Two tools with similar names (may confuse users)
- ❌ Longer implementation time

**Implementation Time**: 1.5 weeks

---

## Comparison: What to Build?

| Tool | Option 1 (Gemini) | Option 2 (Custom API) | Option 3 (Hybrid) |
|------|-------------------|----------------------|-------------------|
| **Google Search** | Built-in, Gemini only | Custom Search API, universal | Both |
| **URL Context** | Built-in, Gemini only | Web scraper, universal | Both |
| **Web Scraper** | N/A | HTTP + HTML parsing | Same as Option 2 |
| **Complexity** | Very low | Medium | Medium-high |
| **Model Support** | Gemini 2.0+ only | All models | All models |
| **Setup Required** | None | API keys | Optional |
| **Code Lines** | ~200 total | ~800 total | ~1000 total |
| **Time** | 3 hours | 1 week | 1.5 weeks |

---

## Python RAK's Web Tool Ecosystem

Python RAK has **many** web/search-related tools:

### Search Tools
1. `google_search_tool.py` - Gemini built-in
2. `google_search_agent_tool.py` - Sub-agent wrapper
3. `discovery_engine_search_tool.py` - GCP Discovery Engine
4. `enterprise_search_tool.py` - Enterprise Search (deprecated)
5. `vertex_ai_search_tool.py` - Vertex AI Search

### Retrieval Tools  
6. `retrieval/vertex_ai_rag_retrieval.py` - RAG with Vertex AI
7. `retrieval/llama_index_retrieval.py` - LlamaIndex integration
8. `retrieval/files_retrieval.py` - File-based retrieval

### Web Tools
9. `load_web_page.py` - Simple web scraper
10. `url_context_tool.py` - Gemini URL context

**Observation**: Python RAK heavily leverages **Gemini built-in tools** for simplicity.

---

## Rust-Specific Considerations

### 1. HTML Parsing Options

| Crate | Pros | Cons |
|-------|------|------|
| **scraper** | Best ergonomics, CSS selectors | Heavier dependency |
| **select** | Lightweight, simple | Less feature-rich |
| **html5ever** | Fast, standards-compliant | Lower-level API |

**Recommendation**: Use `scraper` (most similar to BeautifulSoup)

### 2. HTTP Client

We already use `reqwest` in `rak-openapi`. Continue using it:
```rust
let client = reqwest::Client::builder()
    .user_agent("RAK-Web-Tools/0.1.0")
    .timeout(Duration::from_secs(30))
    .build()?;
```

### 3. Rate Limiting

Python RAK doesn't implement rate limiting. We could add:
```rust
use std::time::{Duration, Instant};

struct RateLimiter {
    last_request: Option<Instant>,
    min_interval: Duration,
}
```

---

## Recommendations

### Minimal Viable Product (MVP) - Option 1 ⭐

**Build**: Gemini built-in tools only
- `GeminiGoogleSearchTool`
- `GeminiUrlContextTool`  
- `WebScraperTool` (simple, like Python)

**Rationale**:
- ✅ Matches Python RAK's primary approach
- ✅ Very quick to implement (3-4 hours)
- ✅ Zero configuration for users
- ✅ Works great with Gemini (our primary model)
- ✅ Ship Phase 8.3 fast

**Limitations**:
- Only works with Gemini 2.0+
- Not usable with Claude/GPT

**Next Steps** (Phase 8.3.1 later):
- Add Custom Search API support when needed
- Add more advanced scraping features

---

### Production Ready - Option 3 ⭐⭐

**Build**: Hybrid approach
- Gemini built-in tools (for Gemini users)
- Custom API tools (for universal support)
- Web scraper (universal)

**Rationale**:
- ✅ Best UX for all users
- ✅ Universal model support
- ✅ More complete offering
- ✅ Future-proof

**Tradeoff**:
- Takes longer (1.5 weeks)
- More code to maintain

---

## Proposed Architecture

### Structure
```
rak-web-tools/
├── Cargo.toml
├── src/
│   ├── lib.rs                          # Public API
│   │
│   ├── gemini/                         # Gemini built-in tools
│   │   ├── mod.rs
│   │   ├── google_search.rs           # GeminiGoogleSearchTool
│   │   └── url_context.rs              # GeminiUrlContextTool
│   │
│   ├── custom/                         # Custom API-based tools
│   │   ├── mod.rs
│   │   ├── google_custom_search.rs    # GoogleCustomSearchTool
│   │   └── web_scraper.rs              # WebScraperTool
│   │
│   └── common/                         # Shared utilities
│       ├── mod.rs
│       ├── html_parser.rs
│       └── rate_limiter.rs
```

### Dependencies
```toml
[dependencies]
rak-core = { path = "../rak-core" }
reqwest = { workspace = true }
scraper = "0.20"          # HTML parsing
url = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true }
anyhow = { workspace = true }
```

---

## Decision Matrix

| Scenario | Recommended Option |
|----------|-------------------|
| **Ship Phase 8.3 ASAP** | Option 1 (MVP) |
| **Only use Gemini** | Option 1 (MVP) |
| **Need multi-model support** | Option 2 or 3 |
| **Want future flexibility** | Option 3 (Hybrid) |
| **Limited time** | Option 1 (MVP) |
| **Want production-ready** | Option 3 (Hybrid) |

---

## Questions for Discussion

1. **Which option do we want?**
   - Option 1: MVP with Gemini built-ins (3 hours)
   - Option 2: Custom API only (1 week)
   - Option 3: Hybrid both (1.5 weeks)

2. **Multi-model priority?**
   - Do we need Claude/GPT support now?
   - Or focus on Gemini first?

3. **Feature scope?**
   - Just basic search + scraping?
   - Or advanced features (filters, pagination, rate limiting)?

4. **Python parity vs Rust best practices?**
   - Match Python exactly?
   - Or improve with better APIs?

---

## My Recommendation 🎯

**Start with Option 1 (MVP)**, then evolve:

### Phase 8.3.0 - NOW (3-4 hours)
- `GeminiGoogleSearchTool` (built-in)
- `GeminiUrlContextTool` (built-in)
- `WebScraperTool` (simple, like Python)

### Phase 8.3.1 - LATER (when needed)
- `GoogleCustomSearchTool` (Custom API)
- Advanced scraping (CSS selectors, link extraction)
- Rate limiting

This matches Python RAK's philosophy: **Leverage model capabilities first, add custom implementations when needed.**

---

**Next Step**: Discuss and decide which option to implement!

