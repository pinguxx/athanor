//! # Athanor Macros
//!
//! Procedural macros for the Athanor agentic framework.
//!
//! Provides the `#[reagent]` attribute macro which derives JSON Schema input specifications,
//! validates parameters, and generates `Reagent` trait implementations from standard
//! asynchronous Rust functions.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::meta::parser;
use syn::{parse_macro_input, FnArg, ItemFn, LitStr, Meta, MetaNameValue, Pat, Path};

#[derive(Default)]
struct ReagentArgs {
    name: Option<String>,
    description: Option<String>,
    core_crate: Option<Path>,
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    if result.is_empty() {
        result.push('X');
    }

    result
}

/// The `#[reagent]` attribute macro transforms an asynchronous Rust function into an
/// executable `Reagent` capability with compile-time generated JSON Schema.
///
/// # Attributes
///
/// - `name = "..."`: Overrides the default tool name (defaults to the function name).
/// - `description = "..."`: Overrides the tool description (defaults to the function's doc comments).
/// - `crate = "..."`: Overrides the path to `athanor_core` (defaults to `::athanor_core`).
///
/// # Generated Artifacts
///
/// For a target function `get_stock_price`:
/// 1. Emits `struct GetStockPriceInput` deriving `serde::Deserialize`, `serde::Serialize`, and `schemars::JsonSchema`.
/// 2. Emits `struct GetStockPriceReagent`.
/// 3. Implements `athanor_core::reagent::Reagent` for `GetStockPriceReagent`.
/// 4. Preserves the original `async fn get_stock_price` for direct calling.
#[proc_macro_attribute]
pub fn reagent(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut args = ReagentArgs::default();

    let attr_parser = parser(|meta| {
        if meta.path.is_ident("name") {
            let val: LitStr = meta.value()?.parse()?;
            args.name = Some(val.value());
            Ok(())
        } else if meta.path.is_ident("description") {
            let val: LitStr = meta.value()?.parse()?;
            args.description = Some(val.value());
            Ok(())
        } else if meta.path.is_ident("crate") {
            let val: LitStr = meta.value()?.parse()?;
            let path: Path = syn::parse_str(&val.value())?;
            args.core_crate = Some(path);
            Ok(())
        } else {
            Err(meta
                .error("unsupported reagent attribute, expected 'name', 'description', or 'crate'"))
        }
    });

    parse_macro_input!(attr with attr_parser);

    let input_fn = parse_macro_input!(item as ItemFn);

    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            &input_fn.sig,
            "The #[reagent] macro can only be applied to async functions",
        )
        .to_compile_error()
        .into();
    }

    let mut doc_comments = Vec::new();
    for attr in &input_fn.attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(MetaNameValue {
                value:
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }),
                ..
            }) = &attr.meta
            {
                doc_comments.push(lit_str.value().trim().to_string());
            }
        }
    }

    let extracted_description = doc_comments.join(" ");
    let final_description = args.description.unwrap_or(extracted_description);

    let mut param_names = Vec::new();
    let mut param_types = Vec::new();
    let mut param_docs = Vec::new();
    let mut call_args = Vec::new();

    for input in &input_fn.sig.inputs {
        match input {
            FnArg::Receiver(_) => {
                return syn::Error::new_spanned(
                    input,
                    "#[reagent] cannot be applied to methods with 'self' receiver",
                )
                .to_compile_error()
                .into();
            }
            FnArg::Typed(pat_type) => {
                let ident = match &*pat_type.pat {
                    Pat::Ident(pat_ident) => pat_ident.ident.clone(),
                    _ => {
                        return syn::Error::new_spanned(
                            &pat_type.pat,
                            "#[reagent] only supports standard named parameter patterns",
                        )
                        .to_compile_error()
                        .into();
                    }
                };

                let ty = (*pat_type.ty).clone();

                let mut param_doc_attrs = Vec::new();
                for attr in &pat_type.attrs {
                    if attr.path().is_ident("doc") {
                        param_doc_attrs.push(attr.clone());
                    }
                }

                param_names.push(ident.clone());
                param_types.push(ty);
                param_docs.push(param_doc_attrs);
                call_args.push(quote! { parsed.#ident });
            }
        }
    }

    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    let tool_name = args.name.unwrap_or_else(|| fn_name_str.clone());

    let pascal_name = to_pascal_case(&fn_name_str);
    let input_struct_ident = format_ident!("{}Input", pascal_name);
    let reagent_struct_ident = format_ident!("{}Reagent", pascal_name);

    let core_path = args
        .core_crate
        .unwrap_or_else(|| syn::parse_quote!(::athanor_core));

    let vis = &input_fn.vis;

    let input_struct = if param_names.is_empty() {
        quote! {
            #[derive(::serde::Deserialize, ::serde::Serialize, ::schemars::JsonSchema, Debug, Clone, Default)]
            #vis struct #input_struct_ident {}
        }
    } else {
        quote! {
            #[derive(::serde::Deserialize, ::serde::Serialize, ::schemars::JsonSchema, Debug, Clone)]
            #vis struct #input_struct_ident {
                #(
                    #(#param_docs)*
                    pub #param_names: #param_types,
                )*
            }
        }
    };

    let reagent_struct = quote! {
        #[derive(Debug, Clone, Copy, Default)]
        #vis struct #reagent_struct_ident;
    };

    let parse_input = if param_names.is_empty() {
        quote! {
            let parsed: #input_struct_ident = if input.is_null() {
                #input_struct_ident {}
            } else {
                ::serde_json::from_value(input)
                    .map_err(|err| #core_path::reagent::ReagentError::InvalidInput(err.to_string()))?
            };
        }
    } else {
        quote! {
            let parsed: #input_struct_ident = ::serde_json::from_value(input)
                .map_err(|err| #core_path::reagent::ReagentError::InvalidInput(err.to_string()))?;
        }
    };

    let reagent_impl = quote! {
        #[#core_path::async_trait]
        impl #core_path::reagent::Reagent for #reagent_struct_ident {
            fn name(&self) -> &'static str {
                #tool_name
            }

            fn description(&self) -> &'static str {
                #final_description
            }

            fn input_schema(&self) -> ::serde_json::Value {
                let schema = ::schemars::schema_for!(#input_struct_ident);
                ::serde_json::to_value(&schema).unwrap_or_else(|_| ::serde_json::json!({
                    "type": "object",
                    "properties": {}
                }))
            }

            fn output_schema(&self) -> ::serde_json::Value {
                ::serde_json::json!({
                    "type": "object"
                })
            }

            async fn transmute(
                &self,
                input: ::serde_json::Value,
            ) -> ::std::result::Result<::serde_json::Value, #core_path::reagent::ReagentError> {
                #parse_input

                let result = #fn_name(#(#call_args),*).await
                    .map_err(|err| -> #core_path::reagent::ReagentError {
                        err.into()
                    })?;

                ::serde_json::to_value(result)
                    .map_err(|err| #core_path::reagent::ReagentError::Fatal(err.to_string()))
            }
        }
    };

    let mut clean_fn = input_fn.clone();
    for input in &mut clean_fn.sig.inputs {
        if let FnArg::Typed(pat_type) = input {
            pat_type.attrs.retain(|attr| !attr.path().is_ident("doc"));
        }
    }

    let expanded = quote! {
        #clean_fn

        #input_struct

        #reagent_struct

        #reagent_impl
    };

    TokenStream::from(expanded)
}

#[derive(Default)]
struct ArtificerTestArgs {
    threshold: Option<f64>,
    cases: Option<String>,
    evals_crate: Option<Path>,
}

/// The `#[artificer_test]` attribute macro transforms an asynchronous benchmark evaluation
/// into a `#[tokio::test]` that enforces required pass rate thresholds.
///
/// # Attributes
///
/// - `threshold = 0.85`: Minimum required pass rate in range `[0.0, 1.0]` (defaults to `1.0`).
/// - `cases = "path/to/cases.json"`: Optional path to a JSON file containing benchmark test cases.
/// - `crate = "..."`: Overrides the path to `athanor_evals` (defaults to `::athanor_evals`).
#[proc_macro_attribute]
pub fn artificer_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut test_args = ArtificerTestArgs::default();

    if !attr.is_empty() {
        let attr_parser = parser(|meta| {
            if meta.path.is_ident("threshold") {
                if let Ok(lit) = meta.value()?.parse::<syn::LitFloat>() {
                    if let Ok(v) = lit.base10_parse::<f64>() {
                        test_args.threshold = Some(v);
                        return Ok(());
                    }
                } else if let Ok(lit) = meta.value()?.parse::<syn::LitInt>() {
                    if let Ok(v) = lit.base10_parse::<f64>() {
                        test_args.threshold = Some(v);
                        return Ok(());
                    }
                }
                Err(meta.error("expected float or integer for threshold"))
            } else if meta.path.is_ident("cases") {
                let lit: LitStr = meta.value()?.parse()?;
                test_args.cases = Some(lit.value());
                Ok(())
            } else if meta.path.is_ident("crate") {
                let path: Path = meta.value()?.parse()?;
                test_args.evals_crate = Some(path);
                Ok(())
            } else {
                Err(meta.error("unrecognized attribute for artificer_test"))
            }
        });

        parse_macro_input!(attr with attr_parser);
    }

    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_attrs = &input_fn.attrs;
    let fn_output = &input_fn.sig.output;
    let fn_block = &input_fn.block;

    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            &input_fn.sig,
            "#[artificer_test] functions must be declared as async fn",
        )
        .to_compile_error()
        .into();
    }

    let threshold = test_args.threshold.unwrap_or(1.0);
    let evals_path = test_args
        .evals_crate
        .unwrap_or_else(|| syn::parse_quote!(::athanor_evals));

    let expanded = quote! {
        #(#fn_attrs)*
        #[::tokio::test]
        #fn_vis async fn #fn_name() {
            async fn __inner_artificer_body() #fn_output {
                #fn_block
            }

            let __report_result = __inner_artificer_body().await;
            #evals_path::suite::ArtificerTestResult::verify_threshold(__report_result, #threshold);
        }
    };

    TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case_scenarios() {
        assert_eq!(to_pascal_case("get_stock_price"), "GetStockPrice");
        assert_eq!(to_pascal_case("calculate_tax_v2"), "CalculateTaxV2");
        assert_eq!(to_pascal_case("kebab-case-name"), "KebabCaseName");
        assert_eq!(to_pascal_case("space separated name"), "SpaceSeparatedName");
        assert_eq!(to_pascal_case("alreadyPascal"), "AlreadyPascal");
        assert_eq!(to_pascal_case(""), "X");
        assert_eq!(to_pascal_case("___"), "X");
    }
}
