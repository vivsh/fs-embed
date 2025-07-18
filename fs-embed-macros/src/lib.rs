use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Lit, LitStr, parse::Parse, parse_macro_input};


/// Embed a directory at compile time, returning a `Dir` enum. The path should be a literal string
/// and strictly relative to the crate root.
/// fs_embed!("dir")                 → Dir::from_embedded
#[proc_macro]
pub fn fs_embed(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as EmbedArgs);

    let rel_lit: LitStr = match args.path {
        Lit::Str(s) => s,
        other => return compile_error("first argument must be a string literal", other.span()),
    };

    let rel_path = rel_lit.value();
    let call_span = rel_lit.span(); // proc_macro2::Span

    // ── validate directory exists inside crate root ────────────────────────
    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => dir,
        Err(_) => return compile_error("fs_embed!: CARGO_MANIFEST_DIR not set", call_span),
    };

    let full_path = match std::path::Path::new(&manifest_dir)
        .join(&rel_path)
        .canonicalize()
        .map_err(|_| {
            syn::Error::new(
                call_span,
                format!("fs_embed!: failed to resolve path: {}", rel_path),
            )
        }) {
        Ok(p) => p,
        Err(msg) => return compile_error(msg.to_string(), call_span),
    };

    let full_path = match full_path.to_str() {
        Some(p) => p,
        None => return compile_error("fs_embed!: path must be valid UTF-8", call_span),
    };

    if !full_path.starts_with(&manifest_dir) {
        let msg = format!(
            "fs_embed!: directory not found:\n  {full_path}\n  expected to be inside crate root:\n  {manifest_dir}\n  relative path: {rel_path}",
        );
        return compile_error(&msg, call_span);
    };

    let full_literal: LitStr = LitStr::new(full_path, call_span);

    let embed_code = quote! {
        ::fs_embed::Dir::from_embedded(include_dir::include_dir!(#full_literal), #full_literal)
    };

    quote! { #embed_code }.into()
}



/// Emit `compile_error!($msg)` at the given span.
#[doc(hidden)]
fn compile_error<S: AsRef<str>>(msg: S, span: Span) -> TokenStream {
    let lit = LitStr::new(msg.as_ref(), span);
    quote!(compile_error!(#lit)).into()
}

struct EmbedArgs {
    path: Lit,
}

impl Parse for EmbedArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path: Lit = input.parse()?;
        Ok(EmbedArgs { path })
    }
}
