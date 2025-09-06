//! Procedural macros demo crate
//!
//! Exposes three macros:
//! - #[derive(HelloWorld)] -> adds `fn hello_world(&self) -> String` to your type.
//! - #[timeit]             -> wraps a (non-async) function body with timing prints.
//! - csv!(a, b, c)         -> compile-time string: concat!(stringify!(a), ",", stringify!(b), ...)

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, spanned::Spanned, AttributeArgs, DeriveInput, Expr, ItemFn, Lit, Meta,
    NestedMeta, punctuated::Punctuated, Token,
};

/* ───────────────────────────── Derive: HelloWorld ───────────────────────────── */

#[proc_macro_derive(HelloWorld)]
pub fn derive_hello_world(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    // Only allow structs (keep the demo simple).
    let data_span = input.ident.span();
    let name = input.ident;

    let is_struct = matches!(input.data, syn::Data::Struct(_));
    if !is_struct {
        let err = syn::Error::new(
            data_span,
            "#[derive(HelloWorld)] only supports structs in this demo",
        );
        return err.to_compile_error().into();
    }

    // Generate an inherent impl method: hello_world(&self) -> String
    let expanded = quote! {
        impl #name {
            pub fn hello_world(&self) -> ::std::string::String {
                ::std::format!("Hello from {}!", ::std::stringify!(#name))
            }
        }
    };
    expanded.into()
}

/* ───────────────────────────── Attribute: #[timeit] ────────────────────────────
Usage:
    #[timeit]           // label defaults to function name
    fn work() { ... }

    #[timeit("custom")] // explicit label
    fn work() { ... }

Notes:
- For brevity, this demo rejects `async fn` and `impl Trait` in the signature.
  (You could support async by wrapping with an `async move { ... }` block.)
*/

#[proc_macro_attribute]
pub fn timeit(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse attribute args (optional string literal)
    let args = parse_macro_input!(attr as AttributeArgs);
    let label_lit = match args.as_slice() {
        [] => None,
        [NestedMeta::Lit(Lit::Str(s))] => Some(s.value()),
        [bad] => {
            let err = syn::Error::new(bad.span(), "#[timeit] expects no args or a single string literal");
            return err.to_compile_error().into();
        }
    };

    let mut func: ItemFn = parse_macro_input!(item as ItemFn);

    // Disallow async for this demo
    if func.sig.asyncness.is_some() {
        let err = syn::Error::new(
            func.sig.fn_token.span(),
            "#[timeit] demo does not support async fn (wrap your body differently)",
        );
        return err.to_compile_error().into();
    }

    // Build label
    let fname = func.sig.ident.to_string();
    let label = label_lit.unwrap_or_else(|| format!("{fname}()"));

    // Take original pieces
    let vis = &func.vis;
    let sig = &func.sig;
    let block = &func.block;

    // Replace function body with timed wrapper (preserve return value)
    let wrapped = quote! {
        #vis #sig {
            let __timeit_start = ::std::time::Instant::now();
            let __timeit_ret = (|| #block)();
            let __timeit_elapsed = __timeit_start.elapsed();
            ::std::println!("[timeit] {} took {:?}", #label, __timeit_elapsed);
            __timeit_ret
        }
    };

    // Return the wrapped function tokens
    wrapped.into()
}

/* ───────────────────────── Function-like: csv!(...) ────────────────────────────
Builds a compile-time string by concatenating the token text of each argument:
    csv!(a, 1 + 2, some::path)  =>  "a,1 + 2,some::path"

This shows:
- parsing punctuated lists with `syn`,
- constructing `concat!(...)` at compile time via `quote!`,
- `stringify!(#expr)` to turn tokens into string parts.
*/

#[proc_macro]
pub fn csv(input: TokenStream) -> TokenStream {
    let exprs: Punctuated<Expr, Token![,]> = parse_macro_input!(input with Punctuated::parse_terminated);
    if exprs.is_empty() {
        // Empty -> empty string literal
        return quote! { "" }.into();
    }

    // Build: concat!( stringify!(expr1), ",", stringify!(expr2), ",", ... )
    let mut pieces = Vec::new();
    for (i, e) in exprs.iter().enumerate() {
        let e_tokens = e.to_token_stream();
        pieces.push(quote! { ::std::stringify!(#e_tokens) });
        if i + 1 != exprs.len() {
            pieces.push(quote! { "," });
        }
    }
    let out = quote! { ::std::concat!( #( #pieces ),* ) };
    out.into()
}

/* ──────────────────────────────── Docs notes ────────────────────────────────
INTERNALS / MENTAL MODEL
- `proc_macro` functions receive a `TokenStream` (syntax tokens) at *compile time* and return
  another `TokenStream` that the compiler parses as Rust code.
- Typical stack:
    TokenStream (proc_macro) → proc-macro2::TokenStream → parse with `syn` → build code with `quote!`.
- Errors: build `syn::Error` at a relevant `Span` and return `error.to_compile_error()` so
  the compiler shows a nice message at the right source location.

HYGIENE / PATHS
- Procedural macros expand in the caller’s context. Prefer absolute paths in generated code
  (e.g., `::std::time::Instant`) so you don’t depend on imports at the call site.
- Use spans from the input where appropriate so errors point to the user code.

SPANS
- `syn` uses `proc_macro2::Span`, which maps to `proc_macro::Span` under the hood.
  Attaching spans to tokens helps better diagnostics and IDE experiences.

LIMITATIONS / DESIGN
- Attribute macros must output items; derive macros are invoked on *items* and usually implement traits,
  but they may also generate inherent impls (as shown).
- Function-like macros can generate any expression/items; keep expansions small and clear.
- For async functions, if you want to support them in attribute macros, you’ll typically transform
  `async fn` into a state machine-compatible wrapper or generate an async block.

PERF & BINARY SIZE
- The macro runs at *compile time*; runtime cost is just whatever code you generate.
- Generated code can increase binary size (e.g., inlined formatting); prefer minimal expansions.

*/
