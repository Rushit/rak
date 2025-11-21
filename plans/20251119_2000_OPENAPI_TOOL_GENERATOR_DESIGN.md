# OpenAPI Tool Generator Design Document

**Phase**: 8.1  
**Created**: 2025-11-19 20:00  
**Status**: Planning

## Purpose

This document details the design and requirements for RAK's OpenAPI Tool Generator - a system that automatically generates RAK tools from OpenAPI specifications.

## Problem Statement

**Current State**: 
- RAK has only 2 built-in tools (Calculator, Echo)
- Python RAK has 100+ tools
- Writing each tool manually is time-consuming and error-prone

**Goal**: 
- Automatically generate tools from OpenAPI specs
- Enable instant integration with any API that has an OpenAPI spec
- Reduce tool creation from days to minutes

## What is OpenAPI?

OpenAPI (formerly Swagger) is an industry-standard specification for describing REST APIs. It defines:
- Endpoints and HTTP methods
- Request/response schemas
- Authentication requirements
- Parameter types and validation

### Example OpenAPI Spec

```yaml
openapi: 3.0.0
info:
  title: Weather API
  version: 1.0.0
servers:
  - url: https://api.weather.com
paths:
  /weather:
    get:
      operationId: getWeather
      summary: Get current weather for a city
      parameters:
        - name: city
          in: query
          required: true
          schema:
            type: string
        - name: units
          in: query
          required: false
          schema:
            type: string
            enum: [celsius, fahrenheit]
      responses:
        '200':
          description: Weather data
          content:
            application/json:
              schema:
                type: object
                properties:
                  temperature:
                    type: number
                  condition:
                    type: string
                  humidity:
                    type: number
```

## Desired User Experience

### For End Users (Agent Developers)

```rust
use rak_openapi::openapi_toolset;

// Option 1: Macro-based (compile-time generation)
#[openapi_toolset("https://api.weather.com/openapi.json")]
struct WeatherTools;

// Option 2: Runtime loading
let weather_tools = OpenApiToolset::from_url("https://api.weather.com/openapi.json").await?;

// Use in agent
let agent = LLMAgent::builder()
    .name("weather_assistant")
    .model(model)
    .toolset(Arc::new(WeatherTools::new()?))  // All tools available!
    .build()?;
```

### Generated Tool Usage

The LLM will see each endpoint as a tool:

```json
{
  "name": "getWeather",
  "description": "Get current weather for a city",
  "parameters": {
    "type": "object",
    "properties": {
      "city": {
        "type": "string",
        "description": "City name"
      },
      "units": {
        "type": "string",
        "enum": ["celsius", "fahrenheit"],
        "description": "Temperature units"
      }
    },
    "required": ["city"]
  }
}
```

## Architecture

### Components

```
┌─────────────────────────────────────────────────────────────┐
│                    OpenAPI Tool Generator                    │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐    ┌─────────────┐    ┌───────────────┐  │
│  │   OpenAPI    │───▶│   Parser    │───▶│  Code Gen     │  │
│  │     Spec     │    │             │    │               │  │
│  │ (JSON/YAML)  │    │ ▪ Validate  │    │ ▪ Rust types  │  │
│  └──────────────┘    │ ▪ Parse     │    │ ▪ HTTP client │  │
│                      │ ▪ Extract    │    │ ▪ Tool trait  │  │
│                      └─────────────┘    └───────────────┘  │
│                             │                     │          │
│                             ▼                     ▼          │
│                      ┌─────────────┐    ┌───────────────┐  │
│                      │  Schema to  │    │  Generated    │  │
│                      │  Rust Types │    │    Tools      │  │
│                      └─────────────┘    └───────────────┘  │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### Crate Structure

```
rak-openapi/
├── src/
│   ├── lib.rs              # Public API
│   ├── parser.rs           # OpenAPI spec parser
│   ├── schema.rs           # Schema type definitions
│   ├── codegen.rs          # Code generation
│   ├── http_client.rs      # HTTP request builder
│   ├── auth.rs             # Authentication handling
│   └── macro.rs            # Procedural macro (optional)
└── Cargo.toml
```

## Requirements

### Phase 8.1.1: Core Parser

Parse OpenAPI v3.0+ specifications:

```rust
pub struct OpenApiSpec {
    pub info: ApiInfo,
    pub servers: Vec<Server>,
    pub paths: HashMap<String, PathItem>,
    pub components: Option<Components>,
}

pub struct PathItem {
    pub get: Option<Operation>,
    pub post: Option<Operation>,
    pub put: Option<Operation>,
    pub delete: Option<Operation>,
    // ... other methods
}

pub struct Operation {
    pub operation_id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub parameters: Vec<Parameter>,
    pub request_body: Option<RequestBody>,
    pub responses: HashMap<String, Response>,
}
```

**Features**:
- Load from URL or file path
- Support JSON and YAML formats
- Validate spec structure
- Handle `$ref` references

### Phase 8.1.2: Schema to Rust Type Conversion

Convert OpenAPI schemas to Rust types:

**OpenAPI Type → Rust Type Mapping**:
- `string` → `String`
- `integer` → `i32` or `i64`
- `number` → `f64`
- `boolean` → `bool`
- `array` → `Vec<T>`
- `object` → `HashMap<String, Value>` or custom struct
- `enum` → Rust enum

**Example**:
```rust
// OpenAPI schema
{
  "type": "object",
  "properties": {
    "name": { "type": "string" },
    "age": { "type": "integer" }
  }
}

// Generated Rust type
#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratedType {
    pub name: String,
    pub age: i32,
}
```

### Phase 8.1.3: HTTP Client Generation

Generate HTTP request code for each operation:

```rust
impl GetWeatherTool {
    async fn execute_request(
        &self,
        city: String,
        units: Option<String>,
    ) -> Result<WeatherResponse> {
        let mut url = self.base_url.clone();
        url.set_path("/weather");
        
        let mut query_params = vec![("city", city)];
        if let Some(u) = units {
            query_params.push(("units", u));
        }
        
        let response = self.client
            .get(url)
            .query(&query_params)
            .send()
            .await?;
        
        let data: WeatherResponse = response.json().await?;
        Ok(data)
    }
}
```

**Features**:
- Path parameters
- Query parameters
- Request body (JSON)
- Headers
- Response parsing
- Error handling

### Phase 8.1.4: Authentication Handling

Support common authentication schemes:

1. **API Key**
   - Header-based: `X-API-Key`
   - Query parameter: `?api_key=xxx`

2. **Bearer Token**
   - `Authorization: Bearer TOKEN`

3. **Basic Auth**
   - `Authorization: Basic base64(user:pass)`

4. **OAuth 2.0** (basic)
   - Client credentials flow

**Example**:
```rust
pub enum AuthConfig {
    ApiKey { header_name: String, key: String },
    Bearer { token: String },
    Basic { username: String, password: String },
}

impl OpenApiToolset {
    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        self.auth = Some(auth);
        self
    }
}
```

### Phase 8.1.5: Tool Integration

Generate tools that implement RAK's `Tool` trait:

```rust
// Generated tool
pub struct GetWeatherTool {
    client: reqwest::Client,
    base_url: Url,
    auth: Option<AuthConfig>,
}

#[async_trait]
impl Tool for GetWeatherTool {
    fn name(&self) -> &str {
        "getWeather"
    }
    
    fn description(&self) -> &str {
        "Get current weather for a city"
    }
    
    fn schema(&self) -> ToolSchema {
        // Generated from OpenAPI schema
        ToolSchema { /* ... */ }
    }
    
    async fn execute(
        &self,
        ctx: Arc<dyn ToolContext>,
        params: Value,
    ) -> Result<ToolResponse> {
        // Parse parameters
        let city: String = params["city"]
            .as_str()
            .ok_or_else(|| Error::MissingParameter("city"))?
            .to_string();
        
        let units: Option<String> = params["units"]
            .as_str()
            .map(String::from);
        
        // Execute HTTP request
        let result = self.execute_request(city, units).await?;
        
        // Return response
        Ok(ToolResponse {
            result: serde_json::to_value(result)?,
        })
    }
}
```

### Phase 8.1.6: Toolset Container

Group all generated tools:

```rust
pub struct OpenApiToolset {
    tools: Vec<Arc<dyn Tool>>,
    base_url: Url,
    auth: Option<AuthConfig>,
}

impl OpenApiToolset {
    pub fn from_spec(spec: OpenApiSpec) -> Result<Self> {
        let mut tools = Vec::new();
        
        for (path, path_item) in spec.paths {
            if let Some(operation) = path_item.get {
                tools.push(Arc::new(
                    generate_tool(&operation, &path, Method::GET)?
                ) as Arc<dyn Tool>);
            }
            // ... other methods
        }
        
        Ok(Self {
            tools,
            base_url: spec.servers[0].url.parse()?,
            auth: None,
        })
    }
    
    pub fn tools(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.clone()
    }
}
```

## Example Usage Scenarios

### Scenario 1: Google Cloud Storage API

```rust
// Generate tools from Google Cloud Storage API
#[openapi_toolset("https://storage.googleapis.com/$discovery/rest?version=v1")]
struct GcsTools;

let gcs = GcsTools::new()?
    .with_auth(AuthConfig::Bearer {
        token: get_gcp_token().await?,
    });

let agent = LLMAgent::builder()
    .name("storage_manager")
    .model(model)
    .tools(gcs.tools())  // All GCS operations available!
    .build()?;
```

### Scenario 2: Stripe API

```rust
let stripe_tools = OpenApiToolset::from_url(
    "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json"
).await?
    .with_auth(AuthConfig::Bearer {
        token: env::var("STRIPE_API_KEY")?,
    });

let agent = LLMAgent::builder()
    .name("payment_agent")
    .model(model)
    .tools(stripe_tools.tools())
    .build()?;

// Agent can now:
// - Create charges
// - List customers
// - Create subscriptions
// - Process refunds
// etc. (100+ operations)
```

### Scenario 3: Internal Company API

```rust
// Load your company's internal API
let internal_tools = OpenApiToolset::from_file("./api/openapi.yaml")?
    .with_auth(AuthConfig::ApiKey {
        header_name: "X-Company-Key".into(),
        key: env::var("COMPANY_API_KEY")?,
    });

let agent = LLMAgent::builder()
    .name("internal_agent")
    .model(model)
    .tools(internal_tools.tools())
    .build()?;
```

## Implementation Phases

### Phase 8.1 (Current): Core OpenAPI Tool Generator with Simple Auth

This phase implements the essential OpenAPI tool system with API Key and Bearer Token authentication. OAuth2 and advanced auth features are deferred to Phase 8.7.

#### Phase 8.1.1: Parser (Week 1)
- [ ] Create `rak-openapi` crate
- [ ] Use `openapiv3` crate for parsing
- [ ] Load specs from URL and file
- [ ] Support JSON and YAML formats
- [ ] Parse operations into `ParsedOperation` structs
- [ ] Basic validation

#### Phase 8.1.2: Type Generation (Week 1-2)
- [ ] Schema to Rust type mapping
- [ ] Generate `ToolSchema` from OpenAPI operation
- [ ] Handle parameters (path, query, header, body)
- [ ] Map OpenAPI types to JSON schema types

#### Phase 8.1.3: HTTP Client (Week 2)
- [ ] Use `reqwest` for HTTP calls
- [ ] Build requests from `ParsedOperation`
- [ ] Handle GET, POST, PUT, DELETE methods
- [ ] Parameter extraction (path, query, header)
- [ ] Request body handling (JSON)
- [ ] Response parsing (JSON, text)
- [ ] Error handling (return as tool response)

#### Phase 8.1.4: Simple Authentication (Week 2-3)
**Scope: API Key and Bearer Token only**
- [ ] `AuthConfig` enum (ApiKey, Bearer, Basic)
- [ ] API Key in header (e.g., `X-API-Key: xxx`)
- [ ] API Key in query (e.g., `?api_key=xxx`)
- [ ] Bearer token (`Authorization: Bearer xxx`)
- [ ] Basic auth (`Authorization: Basic xxx`)
- [ ] Auth injection into HTTP requests
- [ ] Environment variable support

**Out of Scope for Phase 8.1**:
- ❌ OAuth2 flows (see Phase 8.7)
- ❌ Dynamic credential exchange
- ❌ Token refresh
- ❌ User-specific credentials from context

#### Phase 8.1.5: RestApiTool (Week 3)
- [ ] Implement `Tool` trait
- [ ] Generate tool schema from operation
- [ ] Execute HTTP requests
- [ ] Map responses to `ToolResponse`
- [ ] Error handling and retry hints
- [ ] Integration with RAK tool system

#### Phase 8.1.6: OpenApiToolset (Week 3-4)
- [ ] Container for all tools
- [ ] Parse spec → generate tools
- [ ] Builder pattern API
- [ ] Tool filtering support
- [ ] Integration tests
- [ ] Documentation
- [ ] Examples (httpbin.org, public APIs)

### Phase 8.7 (Future): Advanced Authentication & OAuth2

This phase will be implemented after Phase 8.1 is complete and stable. Documentation is provided here for planning purposes.

#### Phase 8.7.1: OAuth2 Foundation
**Goal**: Add OAuth2 authentication support matching Python RAK's capabilities.

**Components**:
- [ ] `OAuth2Config` struct
  - Client credentials flow
  - Authorization code flow
  - Token endpoint configuration
  - Scope management
- [ ] `TokenManager` for token lifecycle
  - Token storage
  - Automatic token refresh
  - Expiration tracking
- [ ] `CredentialExchanger` trait
  - Different exchange strategies
  - Auto-detection from OpenID Connect

**Example Usage** (Phase 8.7):
```rust
// OAuth2 with client credentials
let oauth_config = OAuth2Config {
    client_id: env::var("CLIENT_ID")?,
    client_secret: env::var("CLIENT_SECRET")?,
    token_url: "https://auth.example.com/token".into(),
    scopes: vec!["read".into(), "write".into()],
    flow: OAuth2Flow::ClientCredentials,
};

let toolset = OpenApiToolset::from_file("api.yaml")?
    .with_oauth2(oauth_config);
```

#### Phase 8.7.2: Dynamic Credential Management
**Goal**: Per-user, context-aware authentication.

**Components**:
- [ ] `CredentialService` integration
  - Store user-specific credentials
  - Retrieve from session/context
- [ ] `ToolAuthHandler` for context-aware auth
  - Check if auth is required
  - Return "pending" state if user auth needed
  - Integrate with RAK's context system

**Example Usage** (Phase 8.7):
```rust
// Credentials come from user's context
let toolset = OpenApiToolset::from_file("api.yaml")?
    .with_context_auth(AuthScheme::OAuth2 {
        // Config from OpenAPI spec
    });

// When tool executes:
// 1. Check if user has credentials in context
// 2. If not, return {"pending": true} to trigger user auth flow
// 3. If yes, use credentials for API call
```

#### Phase 8.7.3: Advanced Auth Features
**Goal**: Match Python RAK's full authentication capabilities.

**Components**:
- [ ] OpenID Connect discovery
  - Auto-configure from `.well-known/openid-configuration`
- [ ] Token refresh
  - Automatic refresh on expiration
  - Refresh token management
- [ ] Multi-auth support
  - Multiple auth schemes per spec
  - Per-operation auth override
- [ ] Credential helpers
  - `openid_url_to_scheme_credential()` equivalent
  - Common auth pattern shortcuts

**Example Usage** (Phase 8.7):
```rust
// Auto-configure from OpenID Connect
let (auth_scheme, auth_credential) = 
    openid_url_to_scheme_credential(
        "https://auth.example.com/.well-known/openid-configuration",
        CredentialDict {
            client_id: env::var("CLIENT_ID")?,
            client_secret: env::var("CLIENT_SECRET")?,
        },
        vec!["read", "write"],
    ).await?;

let toolset = OpenApiToolset::from_file("api.yaml")?
    .with_auth(auth_scheme, auth_credential);
```

#### Phase 8.7.4: Testing & Documentation
- [ ] OAuth2 mock server for testing
- [ ] Integration tests with real OAuth2 providers
- [ ] Comprehensive documentation
- [ ] Examples for common OAuth2 scenarios
- [ ] Migration guide from simple auth to OAuth2

**Timeline**: Phase 8.7 to be scheduled after Phase 8.1 completion.

**Dependencies**:
```toml
# Phase 8.7 additional dependencies
oauth2 = "4.4"              # OAuth2 client
openidconnect = "3.0"       # OpenID Connect support
```

## Testing Strategy

### Unit Tests
- Parser tests with sample OpenAPI specs
- Type generation tests
- HTTP client tests (mocked)
- Authentication tests

### Integration Tests
- Test against real public APIs (httpbin.org)
- Test generated tools in actual agents
- End-to-end workflow tests

### Example APIs for Testing
- httpbin.org (simple testing API)
- JSONPlaceholder (fake REST API)
- OpenWeatherMap (real API with free tier)

## Success Criteria

1. **Can generate tools from any valid OpenAPI v3.0 spec**
2. **Generated tools work in RAK agents without modification**
3. **Support for common authentication methods**
4. **Clear error messages for invalid specs**
5. **Complete documentation and examples**

## Benefits

### Immediate
- Instantly support 100+ APIs with OpenAPI specs
- Reduce tool development time by 90%
- Standardized tool generation process

### Strategic
- Foundation for Phase 8.2 (GCP Tools)
- Enables rapid integration with any API
- Community can contribute API integrations easily

### Ecosystem
- Matches Python RAK's API tool ecosystem
- Enables enterprise adoption (internal APIs)
- Future-proof as more APIs adopt OpenAPI

## Dependencies

**Rust Crates**:
- `serde` + `serde_json` - JSON handling
- `serde_yaml` - YAML parsing
- `reqwest` - HTTP client
- `url` - URL parsing
- `openapi` or `openapiv3` - OpenAPI parsing (may need to fork/extend)

## Related Work

- **Python RAK**: Has OpenAPI tool support
- **LangChain**: Has OpenAPI agent toolkit
- **OpenAPI Generator**: Generates clients (not tools)

## Future Enhancements (Post-MVP)

- OpenAPI v2.0 (Swagger) support
- GraphQL schema support
- Streaming responses
- Webhook support
- Custom code templates
- CLI tool for offline generation

---

**Next Steps**: Review requirements, discuss approach, begin implementation.

