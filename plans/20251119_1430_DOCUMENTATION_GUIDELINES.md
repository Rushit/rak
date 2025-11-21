# Documentation Guidelines

**Created:** 2025-11-19 14:30  
**Last Updated:** 2025-11-19 14:30

## Purpose

This document establishes guidelines for creating and maintaining documentation in the rak project.

## Document Naming Convention

All documentation files must follow this naming pattern:

```
YYYYMMDD_HHmm_FEATURE_DOCTYPE.md
```

### Format Breakdown

- **YYYYMMDD**: Date in year-month-day format (e.g., 20251119)
- **HHmm**: Time in 24-hour format (e.g., 1430 for 2:30 PM)
- **FEATURE**: Short descriptor of the feature or topic (e.g., AUTHENTICATION, API, ARCHITECTURE)
- **DOCTYPE**: Type of document (e.g., SPEC, DESIGN, GUIDE, RFC, GUIDELINES)

### Examples

- `20251119_1430_AUTHENTICATION_DESIGN.md` - Authentication design document
- `20251119_0900_API_SPEC.md` - API specification
- `20251120_1600_STREAMING_RFC.md` - RFC for streaming implementation
- `20251121_1000_DEPLOYMENT_GUIDE.md` - Deployment guide

## Document Structure

Every documentation file should include:

### 1. Header Section

```markdown
# [Title]

**Created:** YYYY-MM-DD HH:mm
**Last Updated:** YYYY-MM-DD HH:mm
**Status:** [Draft|In Review|Approved|Deprecated]
**Author(s):** [Names or handles]

## Purpose

Brief description of what this document covers and why it exists.
```

### 2. Main Content

Organize content with clear sections using headers:

- Use `##` for major sections
- Use `###` for subsections
- Use `####` sparingly for sub-subsections

### 3. Code Examples

- Use proper syntax highlighting
- Include comments for clarity
- Ensure examples are runnable when possible
- Show both correct usage and common pitfalls

### 4. References (if applicable)

Link to related documents, external resources, or codebase locations.

## Document Types

### SPEC (Specification)

Technical specifications for features or APIs. Should be precise and comprehensive.

**Required Sections:**
- Purpose
- Technical Requirements
- API/Interface Definition
- Examples
- Testing Considerations

### DESIGN (Design Document)

Architecture and design decisions for features or components.

**Required Sections:**
- Problem Statement
- Proposed Solution
- Alternatives Considered
- Implementation Plan
- Trade-offs

### GUIDE (Guide/Tutorial)

Step-by-step instructions for developers or users.

**Required Sections:**
- Prerequisites
- Step-by-Step Instructions
- Examples
- Troubleshooting
- Next Steps

### RFC (Request for Comments)

Proposals for significant changes that need community input.

**Required Sections:**
- Summary
- Motivation
- Detailed Design
- Drawbacks
- Alternatives
- Open Questions

### GUIDELINES

Standards and best practices for the project.

**Required Sections:**
- Purpose
- Rules/Standards
- Examples
- Exceptions (if any)

## Writing Style

### General Principles

1. **Be Clear and Concise**: Write for your audience, avoid unnecessary jargon
2. **Be Specific**: Use concrete examples over abstract concepts
3. **Be Consistent**: Follow existing patterns in the documentation
4. **Be Current**: Update documents when code changes affect them

### Code Style

1. Follow Rust conventions and idioms
2. Use `rustfmt` for all code examples
3. Include error handling in examples
4. Prefer async/await syntax for async code

### Language

- Use present tense ("returns" not "will return")
- Use active voice ("The function processes" not "The data is processed by")
- Use second person for instructions ("You can configure" not "One can configure")

## Documentation Location

```
rak/
├── docs/                           # All documentation
│   ├── YYYYMMDD_HHmm_*.md         # Timestamped documents
│   └── assets/                     # Images, diagrams, etc.
├── README.md                       # Project overview
└── crates/
    └── */
        └── README.md               # Crate-specific documentation
```

## Review Process

1. **Create** documentation following these guidelines
2. **Self-Review** for clarity, accuracy, and completeness
3. **Submit** for peer review (if applicable)
4. **Update** based on feedback
5. **Maintain** as code evolves

## Updating Existing Documents

When updating a document:

1. Update the "Last Updated" timestamp in the header
2. Consider updating the Status field if applicable
3. Add a changelog section if making significant changes
4. Do not modify the filename timestamp (it represents creation time)

## Tools and Resources

- **Markdown Linting**: Use `markdownlint` for consistency
- **Diagrams**: Use Mermaid for inline diagrams, or store images in `docs/assets/`
- **Links**: Use relative links for internal documentation
- **API Docs**: Use `cargo doc` for API documentation, these guidelines are for conceptual docs

## Examples of Good Documentation

Refer to these documents as examples:
- This document (20251119_1430_DOCUMENTATION_GUIDELINES.md)
- 20251119_1400_IMPLEMENTATION_SUMMARY.md (status tracking)
- 20251119_1410_PROJECT_SCOPE.md (comprehensive scope)
- README.md (project overview)

## Anti-Patterns to Avoid

❌ **Don't:**
- Write documentation that duplicates API docs
- Use ambiguous language ("maybe", "probably", "should work")
- Include outdated code examples
- Create documents without clear purpose
- Use generic filenames without timestamps

✅ **Do:**
- Focus on concepts, architecture, and high-level usage
- Be precise and definitive
- Test code examples before including them
- Have a clear target audience
- Follow the naming convention

## Questions?

If you have questions about these guidelines, open an issue or discussion in the repository.

