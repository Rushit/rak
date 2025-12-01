# Python ZDK OpenAPI Implementation Analysis

**Date**: 2025-11-19 20:10  
**Purpose**: Analyze Python ZDK's OpenAPI implementation to guide ZDK (Rust) implementation

## Summary

The Python ZDK has a **complete, production-ready OpenAPI tool system** that automatically converts OpenAPI specs into executable tools. This is a **core feature** that enables agents to interact with ANY API that has an OpenAPI specification.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                   Python ZDK OpenAPI System                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  OpenAPIToolset ───┐                                             │
│         │          │                                             │
│         ├─ Parse OpenAPI Spec (JSON/YAML)                        │
│         ├─ Generate RestApiTool for each operation               │
│         └─ Configure authentication                              │
│                                                                   │
│  RestApiTool ───┐                                                │
│         │       │                                                │
│         ├─ Implements BaseTool interface                         │
│         ├─ Builds HTTP requests (params, body, headers)          │
│         ├─ Handles authentication (OAuth2, API Key, Bearer)      │
│         └─ Executes HTTP calls and returns results               │
│                                                                   │
│  OpenApiSpecParser ───┐                                          │
│         │             │                                          │
│         ├─ Parses OpenAPI v3.0+ specs                            │
│         ├─ Resolves $ref references (including circular)         │
│         └─ Extracts operations into ParsedOperation objects      │
│                                                                   │
│  OperationParser ───┐                                            │
│         │           │                                            │
│         ├─ Converts OpenAPI schemas to JSON schemas              │
│         ├─ Maps parameters (path, query, header, cookie, body)   │
│         └─ Generates function names and descriptions             │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. OpenAPIToolset

**Location**: `src/google/adk/tools/openapi_tool/openapi_spec_parser/openapi_toolset.py`

**Purpose**: Main entry point that converts an entire OpenAPI spec into a set of tools.

**Key Features**:
- Accepts OpenAPI spec as string (JSON/YAML) or dict
- Parses spec into individual operations
- Creates a `RestApiTool` for each operation
- Supports authentication configuration
- Supports tool filtering (select subset of operations)

**Usage Example**:
```python
from google.adk.tools.openapi_tool.openapi_spec_parser import OpenAPIToolset

# Load from YAML file
with open("./api/openapi.yaml", "r") as f:
    spec_yaml = f.read()

# Create toolset
toolset = OpenAPIToolset(
    spec_str=spec_yaml,
    spec_str_type="yaml",
    auth_scheme=auth_scheme,      # Optional auth config
    auth_credential=auth_cred,    # Optional auth credentials
    tool_filter=["operation_1", "operation_2"]  # Optional: only these tools
)

# Use in agent
agent = LlmAgent(
    name="api_agent",
    model="gemini-2.5-flash",
    tools=[toolset],  # Pass entire toolset
)
```

**Key Methods**:
- `__init__()` - Parse spec and generate tools
- `get_tools()` - Return all tools (or filtered subset)
- `get_tool(name)` - Get specific tool by name
- `_parse()` - Internal: Parse spec into RestApiTool list
- `_configure_auth_all()` - Apply auth to all tools

### 2. RestApiTool

**Location**: `src/google/adk/tools/openapi_tool/openapi_spec_parser/rest_api_tool.py`

**Purpose**: Represents a single API operation as an executable tool.

**Key Features**:
- Implements `BaseTool` interface (inherits from `BaseTool`)
- Generates function declaration for LLM (name, description, parameters)
- Builds HTTP requests from LLM arguments
- Handles all HTTP methods (GET, POST, PUT, DELETE, PATCH, etc.)
- Supports multiple content types (JSON, form data, multipart, etc.)
- Integrates authentication into requests
- Executes HTTP calls and parses responses

**Key Methods**:
- `from_parsed_operation()` - Factory method from ParsedOperation
- `run_async()` - Execute the tool (async)
- `call()` - Internal: Make HTTP request
- `_get_declaration()` - Generate Gemini function declaration
- `_prepare_request_params()` - Build HTTP request parameters
- `configure_auth_scheme()` - Set authentication scheme
- `configure_auth_credential()` - Set authentication credentials

**HTTP Request Building**:
```python
def _prepare_request_params(self, parameters, kwargs):
    # Extract parameters by location
    path_params = {}    # {id} in /users/{id}
    query_params = {}   # ?search=value
    header_params = {}  # Authorization: Bearer token
    cookie_params = {}  # Cookie: session=xyz
    
    # Build request body (JSON, form data, multipart, etc.)
    body_kwargs = {}
    
    # Construct final URL
    url = f"{base_url}{path.format(**path_params)}"
    
    # Return request parameters
    return {
        "method": method,
        "url": url,
        "params": query_params,
        "headers": header_params,
        "cookies": cookie_params,
        **body_kwargs,
    }
```

**Tool Execution Flow**:
```python
async def call(self, args, tool_context):
    # 1. Prepare authentication
    auth_handler = ToolAuthHandler.from_tool_context(...)
    auth_result = await auth_handler.prepare_auth_credentials()
    
    # 2. Build request parameters
    api_params = self._operation_parser.get_parameters()
    request_params = self._prepare_request_params(api_params, args)
    
    # 3. Execute HTTP request
    response = requests.request(**request_params)
    
    # 4. Parse and return response
    return response.json()
```

### 3. OpenApiSpecParser

**Location**: `src/google/adk/tools/openapi_tool/openapi_spec_parser/openapi_spec_parser.py`

**Purpose**: Parses OpenAPI specification and extracts operations.

**Key Features**:
- Parses OpenAPI v3.0+ specs
- Resolves `$ref` references (including circular references)
- Extracts global and operation-level auth schemes
- Generates operation IDs if missing
- Returns `ParsedOperation` objects for each endpoint/method

**Key Methods**:
- `parse(spec_dict)` - Main entry point
- `_collect_operations()` - Extract all operations from spec
- `_resolve_references()` - Resolve all $ref references

**Reference Resolution**:
```python
def _resolve_references(self, openapi_spec):
    """Recursively resolves all $ref references.
    
    Handles:
    - Internal references (#/components/schemas/User)
    - Circular references (User -> Address -> User)
    - Nested references
    """
    # Uses caching and seen_refs set to handle circularity
    # Returns fully resolved spec with no $ref entries
```

**Operation Collection**:
```python
def _collect_operations(self, spec):
    operations = []
    
    # Get base URL from servers
    base_url = spec["servers"][0]["url"] if spec.get("servers") else ""
    
    # Get global auth scheme
    global_auth = spec.get("security", [{}])[0]
    auth_schemes = spec.get("components", {}).get("securitySchemes", {})
    
    # Iterate through paths and methods
    for path, path_item in spec.get("paths", {}).items():
        for method in ["get", "post", "put", "delete", ...]:
            operation = path_item.get(method)
            if operation:
                # Parse operation
                parsed_op = ParsedOperation(
                    name=operation_id,
                    description=operation.description,
                    endpoint=OperationEndpoint(base_url, path, method),
                    operation=operation,
                    parameters=...,
                    auth_scheme=...,
                )
                operations.append(parsed_op)
    
    return operations
```

### 4. OperationParser

**Location**: `src/google/adk/tools/openapi_tool/openapi_spec_parser/operation_parser.py`

**Purpose**: Parses individual operations and extracts parameters.

**Key Features**:
- Extracts parameters from all locations (path, query, header, cookie, body)
- Converts OpenAPI schemas to JSON schemas
- Generates function names (operationId → snake_case)
- Handles request body schemas
- Extracts return value schema

**Data Structures**:
```python
class ApiParameter:
    original_name: str       # Name in OpenAPI spec
    py_name: str            # Python-friendly name (snake_case)
    param_location: str     # "path", "query", "header", "cookie", "body"
    required: bool
    param_schema: Schema    # OpenAPI schema object

class ParsedOperation:
    name: str                      # Function name (snake_case)
    description: str               # Human-readable description
    endpoint: OperationEndpoint    # base_url, path, method
    operation: Operation           # Full OpenAPI operation object
    parameters: List[ApiParameter] # All parameters
    return_value: ApiParameter     # Response schema
    auth_scheme: Optional[AuthScheme]
    auth_credential: Optional[AuthCredential]
```

### 5. Authentication System

**Location**: `src/google/adk/tools/openapi_tool/auth/`

**Components**:
- `ToolAuthHandler` - Manages auth for tool execution
- `AuthScheme` - Defines auth method (OAuth2, API Key, Bearer, Basic)
- `AuthCredential` - Stores credentials
- `AutoAuthCredentialExchanger` - Handles OAuth2 flows
- `auth_helpers.py` - Helper functions for common auth patterns

**Supported Auth Methods**:

1. **API Key** (in header or query)
   ```python
   auth_scheme = AuthScheme(
       type="apiKey",
       name="X-API-Key",
       in_="header"
   )
   auth_credential = AuthCredential(api_key="your_key")
   ```

2. **Bearer Token**
   ```python
   auth_scheme = AuthScheme(type="http", scheme="bearer")
   auth_credential = AuthCredential(access_token="your_token")
   ```

3. **Basic Auth**
   ```python
   auth_scheme = AuthScheme(type="http", scheme="basic")
   auth_credential = AuthCredential(
       username="user",
       password="pass"
   )
   ```

4. **OAuth 2.0**
   ```python
   auth_scheme = AuthScheme(
       type="oauth2",
       flows={
           "clientCredentials": {
               "tokenUrl": "https://auth.example.com/token",
               "scopes": {"read": "Read access"}
           }
       }
   )
   auth_credential = AuthCredential(
       client_id="client_id",
       client_secret="client_secret"
   )
   ```

**Auth Flow in Tool Execution**:
```python
async def call(self, args, tool_context):
    # 1. Prepare auth credentials (may involve OAuth2 exchange)
    auth_handler = ToolAuthHandler.from_tool_context(
        tool_context, 
        self.auth_scheme, 
        self.auth_credential
    )
    auth_result = await auth_handler.prepare_auth_credentials()
    
    # 2. Check if auth is ready
    if auth_result.state == "pending":
        return {"pending": True, "message": "Auth required"}
    
    # 3. Inject auth into request
    auth_param, auth_args = self._prepare_auth_request_params(
        auth_result.auth_scheme, 
        auth_result.auth_credential
    )
    
    # 4. Execute request with auth
    request_params = self._prepare_request_params(...)
    response = requests.request(**request_params)
```

## Real-World Usage Example

**Scenario**: Hotel booking API with OAuth2 authentication

```python
# 1. Set up authentication
credential_dict = {
    "client_id": os.environ["OAUTH_CLIENT_ID"],
    "client_secret": os.environ["OAUTH_CLIENT_SECRET"],
}

auth_scheme, auth_credential = openid_url_to_scheme_credential(
    openid_url="https://auth.example.com/.well-known/openid-configuration",
    credential_dict=credential_dict,
    scopes=["read", "write"],
)

# 2. Load OpenAPI spec
with open("./hotel_api/openapi.yaml", "r") as f:
    spec_yaml = f.read()

# 3. Create toolset
openapi_toolset = OpenAPIToolset(
    spec_str=spec_yaml,
    spec_str_type="yaml",
    auth_scheme=auth_scheme,
    auth_credential=auth_credential,
)

# 4. Create agent with toolset
agent = LlmAgent(
    name="hotel_agent",
    instruction="Help users find and book hotels using the provided tools.",
    model="gemini-2.5-flash",
    tools=[openapi_toolset],  # All API operations now available as tools!
)

# 5. Agent can now call ANY operation from the API:
# - search_hotels(location, check_in, check_out)
# - get_hotel_details(hotel_id)
# - create_booking(hotel_id, guest_info)
# - get_user_bookings()
# - cancel_booking(booking_id)
```

## Key Implementation Insights for ZDK

### 1. **Runtime Generation Approach**

Python ZDK uses **runtime generation**:
- Parse OpenAPI spec at runtime
- Dynamically create tool instances
- No code generation or macros needed

**Pros**:
- Simple implementation
- Works with any spec immediately
- Easy to debug

**Cons**:
- No compile-time type checking
- Spec parsing happens at startup

**For ZDK**: Start with runtime generation, add compile-time macros later.

### 2. **Tool Interface Design**

Python ZDK's `RestApiTool`:
- Implements `BaseTool` interface
- Returns function declaration (name, description, schema)
- Executes with arbitrary JSON args
- Returns JSON response

**ZDK Equivalent**:
```rust
pub struct RestApiTool {
    name: String,
    description: String,
    endpoint: OperationEndpoint,
    operation: Operation,
    parameters: Vec<ApiParameter>,
    auth_scheme: Option<AuthScheme>,
}

#[async_trait]
impl Tool for RestApiTool {
    fn name(&self) -> &str { &self.name }
    fn description(&self) -> &str { &self.description }
    fn schema(&self) -> ToolSchema { /* Generated from operation */ }
    
    async fn execute(
        &self,
        ctx: Arc<dyn ToolContext>,
        params: Value,
    ) -> Result<ToolResponse> {
        // Build and execute HTTP request
    }
}
```

### 3. **HTTP Request Building**

Python ZDK separates parameter handling by location:
- Path parameters: `{id}` in URL
- Query parameters: `?key=value`
- Header parameters: `Authorization: Bearer token`
- Cookie parameters: `Cookie: session=xyz`
- Body: JSON, form data, multipart, etc.

**Key Pattern**:
```python
# 1. Extract parameters by location
for param_name, value in kwargs.items():
    param = params_map[param_name]
    if param.param_location == "path":
        path_params[param.original_name] = value
    elif param.param_location == "query":
        query_params[param.original_name] = value
    # ... etc
    
# 2. Format URL with path params
url = f"{base_url}{path.format(**path_params)}"

# 3. Build request
requests.request(
    method=method,
    url=url,
    params=query_params,
    headers=header_params,
    json=body_data,
)
```

**For ZDK**: Use `reqwest::RequestBuilder` with similar pattern.

### 4. **Schema Conversion**

Python ZDK converts OpenAPI schemas to Gemini function declarations:
- OpenAPI Schema → JSON Schema → Gemini Schema
- Uses `_to_gemini_schema()` utility
- Handles nested objects, arrays, enums, etc.

**For ZDK**: We already have `schemars` and JSON schema support, so this should be straightforward.

### 5. **Authentication Integration**

Python ZDK has sophisticated auth handling:
- Separate `AuthScheme` (how to auth) and `AuthCredential` (what to auth with)
- `ToolAuthHandler` manages auth state
- `AutoAuthCredentialExchanger` handles OAuth2 flows
- Auth credentials can come from tool context (user-specific)

**Key Pattern**:
```python
# Auth is prepared at execution time, not tool creation time
async def call(self, args, tool_context):
    # 1. Get auth from tool or context
    auth_handler = ToolAuthHandler.from_tool_context(
        tool_context,
        self.auth_scheme,
        self.auth_credential
    )
    
    # 2. Prepare/exchange credentials if needed
    auth_result = await auth_handler.prepare_auth_credentials()
    
    # 3. Inject into request
    if auth_result.auth_credential:
        # Add to headers, query params, etc.
```

**For ZDK**: Start simple (API key, Bearer), add OAuth2 later.

### 6. **Error Handling**

Python ZDK returns errors as tool results:
```python
try:
    response.raise_for_status()
    return response.json()
except HTTPError:
    return {
        "error": f"Tool {self.name} execution failed. "
                f"Status Code: {response.status_code}, "
                f"{error_details}"
    }
```

**Why**: LLM can see the error and retry with corrections.

**For ZDK**: Similar approach - return errors as `ToolResponse` with error field.

### 7. **Reference Resolution**

Python ZDK resolves `$ref` recursively with circular reference handling:
```python
def _resolve_references(openapi_spec):
    resolved_cache = {}
    
    def recursive_resolve(obj, seen_refs=None):
        if "$ref" in obj:
            ref = obj["$ref"]
            if ref in seen_refs:
                # Circular ref - return object without $ref
                return {k: v for k, v in obj.items() if k != "$ref"}
            
            seen_refs.add(ref)
            resolved = resolve_ref(ref)
            resolved_cache[ref] = resolved
            return recursive_resolve(resolved, seen_refs)
        # ... handle dicts and lists recursively
```

**For ZDK**: Use `openapiv3` crate which may already handle this, or implement similar logic.

## Comparison: Python ZDK vs ZDK Design

| Aspect | Python ZDK | ZDK (Proposed) |
|--------|-----------|----------------|
| **Generation** | Runtime | Runtime (Phase 1), Macro (Phase 2) |
| **Spec Parsing** | Custom parser | `openapiv3` crate |
| **HTTP Client** | `requests` library | `reqwest` crate |
| **Auth** | Full OAuth2 + more | Start simple, expand |
| **Type Safety** | Dynamic (JSON) | Static where possible |
| **Toolset** | `OpenAPIToolset` class | `OpenApiToolset` struct |
| **Individual Tool** | `RestApiTool` class | `RestApiTool` struct |
| **Schema** | Python types + Gemini | `schemars` + `serde_json` |
| **Testing** | Mock HTTP, unit tests | Similar approach |

## Recommended Implementation Plan for ZDK

### Phase 8.1.1: Core Parser (Week 1)
- Use `openapiv3` crate for parsing
- Create `OpenApiSpec`, `ParsedOperation` structs
- Handle `$ref` resolution (may be built into `openapiv3`)
- Load from URL and file

### Phase 8.1.2: Type Generation (Week 1-2)
- Map OpenAPI types to Rust types
- Generate `ToolSchema` from OpenAPI operation
- Handle parameters (path, query, header, body)

### Phase 8.1.3: HTTP Client (Week 2)
- Use `reqwest` for HTTP calls
- Build requests from `ParsedOperation`
- Handle all HTTP methods
- Parse responses (JSON, text, binary)

### Phase 8.1.4: Authentication (Week 2-3)
- Start with API Key (header/query)
- Add Bearer token
- Add Basic auth
- OAuth2 later (Phase 8.1.7)

### Phase 8.1.5: RestApiTool (Week 3)
- Implement `Tool` trait
- Execute HTTP requests
- Error handling
- Integration with ZDK tool system

### Phase 8.1.6: OpenApiToolset (Week 3-4)
- Container for all tools
- Parse spec → generate tools
- Filter support
- Builder pattern API

### Phase 8.1.7: OAuth2 Support (Post-MVP)
- Client credentials flow
- Authorization code flow
- Token refresh
- Integration with credential services

## Dependencies

```toml
[dependencies]
# OpenAPI parsing
openapiv3 = "2.0"           # OpenAPI v3.0+ parser

# HTTP client
reqwest = { version = "0.11", features = ["json", "multipart"] }

# JSON handling
serde_json = "1.0"

# URL manipulation
url = "2.5"

# Async
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"
```

## Success Criteria (Matching Python ZDK)

1. ✅ Parse any valid OpenAPI v3.0+ spec
2. ✅ Generate tools for all operations
3. ✅ Support all HTTP methods (GET, POST, PUT, DELETE, etc.)
4. ✅ Handle parameters in all locations (path, query, header, cookie, body)
5. ✅ Support common content types (JSON, form data, multipart)
6. ✅ Integrate with ZDK's `Tool` trait
7. ✅ Support API Key and Bearer token auth
8. ✅ Return errors as tool responses (for LLM retry)
9. ✅ Work with real public APIs (httpbin.org, OpenWeatherMap, etc.)
10. ✅ Complete documentation and examples

## Conclusion

Python ZDK's OpenAPI implementation is **comprehensive and production-ready**. It demonstrates that:

1. **OpenAPI tool generation is critical** - It's not a "nice-to-have" but a core feature
2. **Runtime generation works well** - No need for complex macros initially
3. **Authentication is complex** - Need proper abstraction for auth schemes
4. **Error handling matters** - Errors should be tool responses, not exceptions
5. **Real APIs have edge cases** - Need robust parsing and handling

For ZDK, we should:
- **Start with runtime generation** (simpler, matches Python)
- **Use existing crates** (`openapiv3`, `reqwest`)
- **Focus on common cases first** (GET/POST, API Key auth)
- **Match Python's API design** (OpenAPIToolset, RestApiTool)
- **Add Rust-specific improvements** later (compile-time generation, type safety)

This will give ZDK feature parity with Python ZDK for OpenAPI support, enabling instant integration with 1000s of APIs.

