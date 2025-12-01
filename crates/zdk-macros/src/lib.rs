//! Procedural macros for ZDK
//!
//! This crate provides the `#[tool]` attribute macro for creating tools ergonomically.

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, Lit, parse_macro_input};

/// Derives the `Tool` trait for a struct
///
/// # Example
///
/// ```ignore
/// use zdk_macros::Tool;
/// use zdk_core::Tool;
///
/// #[derive(Tool)]
/// struct MyTool {
///     name: String,
/// }
/// ```
#[proc_macro_derive(Tool, attributes(tool))]
pub fn derive_tool(_input: TokenStream) -> TokenStream {
    // For future implementation if needed
    TokenStream::new()
}

/// Converts a function into a Tool implementation
///
/// # Example
///
/// ```ignore
/// use zdk_macros::tool;
/// use zdk_core::{ToolContext, Result, ToolResponse};
/// use std::sync::Arc;
///
/// #[tool(description = "Adds two numbers together")]
/// async fn add(ctx: Arc<dyn ToolContext>, x: f64, y: f64) -> Result<ToolResponse> {
///     Ok(ToolResponse {
///         result: serde_json::json!({"sum": x + y}),
///     })
/// }
/// ```
#[proc_macro_attribute]
pub fn tool(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    // Parse attributes
    let attrs = parse_tool_attributes(args);
    let description = attrs
        .get("description")
        .cloned()
        .unwrap_or_else(|| "No description provided".to_string());

    let fn_name = &input_fn.sig.ident;
    let tool_name = fn_name.to_string();
    let fn_visibility = &input_fn.vis;
    let fn_body = &input_fn.block;
    let fn_async = &input_fn.sig.asyncness;

    // Generate the tool creation function
    let creator_name = syn::Ident::new(&format!("create_{}_tool", fn_name), fn_name.span());

    let output = quote! {
        /// Original function (kept for direct usage if needed)
        #fn_visibility #fn_async fn #fn_name(
            ctx: ::std::sync::Arc<dyn ::zdk_core::ToolContext>,
            params: ::serde_json::Value,
        ) -> ::zdk_core::Result<::zdk_core::ToolResponse> {
            #fn_body
        }

        /// Tool creator function
        #fn_visibility fn #creator_name() -> ::zdk_core::Result<::zdk_tool::FunctionTool> {
            use ::zdk_tool::ToolSchema;

            let schema = ToolSchema::new()
                .property("params", "object", "Tool parameters")
                .build();

            ::zdk_tool::FunctionTool::builder()
                .name(#tool_name)
                .description(#description)
                .schema(schema)
                .execute(|ctx, params| async move {
                    #fn_name(ctx, params).await
                })
                .build()
        }
    };

    TokenStream::from(output)
}

fn parse_tool_attributes(args: TokenStream) -> std::collections::HashMap<String, String> {
    let mut attrs = std::collections::HashMap::new();

    if args.is_empty() {
        return attrs;
    }

    // Parse as attribute arguments
    let parsed = syn::parse::Parser::parse(
        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
        args,
    );

    if let Ok(metas) = parsed {
        for meta in metas {
            if let syn::Meta::NameValue(nv) = meta {
                if nv.path.is_ident("description") {
                    if let syn::Expr::Lit(expr_lit) = &nv.value {
                        if let Lit::Str(s) = &expr_lit.lit {
                            attrs.insert("description".to_string(), s.value());
                        }
                    }
                }
            }
        }
    }

    attrs
}
