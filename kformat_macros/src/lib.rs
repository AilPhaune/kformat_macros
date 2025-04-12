use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, punctuated::Punctuated, Expr, Ident, LitStr, Token};

struct WriteSingleArgs {
    value: Expr,
    _comma1: Token![,],
    value_trait: Ident,
    _comma2: Token![,],
    value_write_fn: Ident,
    _comma3: Token![,],
    writer: Expr,
    _comma4: Token![,],
    writer_trait: Ident,
    _comma5: Token![,],
    hint_pretty: Expr,
    _comma6: Token![,],
    hint_radix: Expr,
    _comma7: Token![,],
    hint_width: Expr,
    _comma8: Token![,],
    hint_precision: Expr,
    _comma9: Token![,],
    hint_case: Expr,
}

impl Parse for WriteSingleArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            value: input.parse()?,
            _comma1: input.parse()?,
            value_trait: input.parse()?,
            _comma2: input.parse()?,
            value_write_fn: input.parse()?,
            _comma3: input.parse()?,
            writer: input.parse()?,
            _comma4: input.parse()?,
            writer_trait: input.parse()?,
            _comma5: input.parse()?,
            hint_pretty: input.parse()?,
            _comma6: input.parse()?,
            hint_radix: input.parse()?,
            _comma7: input.parse()?,
            hint_width: input.parse()?,
            _comma8: input.parse()?,
            hint_precision: input.parse()?,
            _comma9: input.parse()?,
            hint_case: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn write_single(input: TokenStream) -> TokenStream {
    let WriteSingleArgs {
        value,
        value_trait,
        value_write_fn,
        writer,
        writer_trait,
        hint_pretty,
        hint_radix,
        hint_width,
        hint_precision,
        hint_case,
        ..
    } = parse_macro_input!(input as WriteSingleArgs);

    let expanded = quote! {
        {
            let __value_as_trait: &dyn #value_trait = &#value;
            let mut __writer = &mut #writer as &mut dyn #writer_trait;

            __value_as_trait.#value_write_fn(__writer, #hint_pretty, #hint_radix, #hint_width, #hint_precision, #hint_case)
        }
    };

    TokenStream::from(expanded)
}

struct WriteInput {
    writer: Ident,
    _comma: Token![,],
    trait_format: Ident,
    _comma2: Token![,],
    format_fn: Ident,
    _comma3: Token![,],
    trait_debug: Ident,
    _comma4: Token![,],
    debug_fn: Ident,
    _comma5: Token![,],
    trait_writeable: Ident,
    _comma6: Token![,],
    error_type: Ident,
    _comma7: Token![,],
    format: LitStr,
    _comma8: Token![,],
    args: Punctuated<Expr, Token![,]>,
}

impl Parse for WriteInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            writer: input.parse()?,
            _comma: input.parse()?,
            trait_format: input.parse()?,
            _comma2: input.parse()?,
            format_fn: input.parse()?,
            _comma3: input.parse()?,
            trait_debug: input.parse()?,
            _comma4: input.parse()?,
            debug_fn: input.parse()?,
            _comma5: input.parse()?,
            trait_writeable: input.parse()?,
            _comma6: input.parse()?,
            error_type: input.parse()?,
            _comma7: input.parse()?,
            format: input.parse()?,
            _comma8: input.parse()?,
            args: Punctuated::parse_terminated(input)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormatMethod {
    Display,
    Debug,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormatHint {
    Pretty,
    Radix(usize),
    Width(usize),
    Precision(usize),
    Lowercase,
    Uppercase,
}

#[derive(Debug, Clone)]
struct FormatType {
    method: FormatMethod,
    hints: Vec<FormatHint>,
}

enum FormatPart {
    Text(String),
    Placeholder(FormatType),
}

fn parse_fmt_placeholder(fmt: &str) -> Result<FormatType, TokenStream> {
    if fmt.is_empty() {
        Ok(FormatType {
            method: FormatMethod::Display,
            hints: Vec::new(),
        })
    } else {
        let mut method = FormatMethod::Display;
        let parts = fmt.split(':');
        let mut hints = Vec::new();
        for part in parts {
            match part {
                "#" => {
                    if method != FormatMethod::Display {
                        return Err(syn::Error::new_spanned(
                            fmt,
                            "Duplicate '#' (pretty print) in format string",
                        )
                        .to_compile_error()
                        .into());
                    }
                    hints.push(FormatHint::Pretty);
                }
                "?" => {
                    method = FormatMethod::Debug;
                }
                "x" => {
                    if hints
                        .iter()
                        .any(|hint| matches!(hint, FormatHint::Radix(_)))
                    {
                        return Err(syn::Error::new_spanned(
                            fmt,
                            "Using 'x' (lowercase hexadecimal) hint when radix hint is already given",
                        )
                        .to_compile_error()
                        .into());
                    }
                    if hints.iter().any(|hint| hint == &FormatHint::Uppercase) {
                        return Err(syn::Error::new_spanned(
                            fmt,
                            "Using 'x' (lowercase hexadecimal) hint when uppercase hint is already given",
                        )
                        .to_compile_error()
                        .into());
                    }
                    hints.push(FormatHint::Radix(16));
                    if !hints.iter().any(|hint| hint == &FormatHint::Lowercase) {
                        hints.push(FormatHint::Lowercase);
                    }
                }
                "X" => {
                    if hints
                        .iter()
                        .any(|hint| matches!(hint, FormatHint::Radix(_)))
                    {
                        return Err(syn::Error::new_spanned(
                            fmt,
                            "Using 'X' (uppercase hexadecimal) hint when radix hint is already given",
                        )
                        .to_compile_error()
                        .into());
                    }
                    if hints.iter().any(|hint| hint == &FormatHint::Lowercase) {
                        return Err(syn::Error::new_spanned(
                            fmt,
                            "Using 'X' (uppercase hexadecimal) hint when lowercase hint is already given",
                        )
                        .to_compile_error()
                        .into());
                    }
                    hints.push(FormatHint::Radix(16));
                    if !hints.iter().any(|hint| hint == &FormatHint::Uppercase) {
                        hints.push(FormatHint::Uppercase);
                    }
                }
                "o" | "O" => {
                    if hints
                        .iter()
                        .any(|hint| matches!(hint, FormatHint::Radix(_)))
                    {
                        return Err(syn::Error::new_spanned(
                            fmt,
                            "Using 'o' or 'O' (octal) hint when radix hint is already given",
                        )
                        .to_compile_error()
                        .into());
                    }
                    hints.push(FormatHint::Radix(8));
                }
                "b" | "B" => {
                    if hints
                        .iter()
                        .any(|hint| matches!(hint, FormatHint::Radix(_)))
                    {
                        return Err(syn::Error::new_spanned(
                            fmt,
                            "Using 'b' or 'B' (binary) hint when radix hint is already given",
                        )
                        .to_compile_error()
                        .into());
                    }
                    hints.push(FormatHint::Radix(2));
                }
                _ => {
                    let chars = part.chars().collect::<Vec<char>>();
                    match chars[0] {
                        'r' | 'R' => {
                            if hints
                                .iter()
                                .any(|hint| matches!(hint, FormatHint::Radix(_)))
                            {
                                return Err(syn::Error::new_spanned(
                                    fmt,
                                    "Using 'r' or 'R' (radix) hint when radix hint is already given",
                                )
                                .to_compile_error()
                                .into());
                            }
                            hints.push(FormatHint::Radix(
                                chars[1..].iter().collect::<String>().parse().unwrap(),
                            ));
                        }
                        'w' | 'W' => {
                            if hints
                                .iter()
                                .any(|hint| matches!(hint, FormatHint::Width(_)))
                            {
                                return Err(syn::Error::new_spanned(
                                    fmt,
                                    "Using 'w' or 'W' (width) hint when width hint is already given",
                                )
                                .to_compile_error()
                                .into());
                            }
                            hints.push(FormatHint::Width(
                                chars[1..].iter().collect::<String>().parse().unwrap(),
                            ));
                        }
                        'p' | 'P' => {
                            if hints
                                .iter()
                                .any(|hint| matches!(hint, FormatHint::Precision(_)))
                            {
                                return Err(syn::Error::new_spanned(
                                    fmt,
                                    "Using 'p' or 'P' (precision) hint when precision hint is already given",
                                )
                                .to_compile_error()
                                .into());
                            }
                            hints.push(FormatHint::Precision(
                                chars[1..].iter().collect::<String>().parse().unwrap(),
                            ));
                        }
                        'u' | 'U' => {
                            if hints.iter().any(|hint| hint == &FormatHint::Uppercase) {
                                return Err(syn::Error::new_spanned(
                                    fmt,
                                    "Using 'u' or 'U' (uppercase) hint when uppercase hint is already given",
                                )
                                .to_compile_error()
                                .into());
                            }
                            hints.push(FormatHint::Uppercase);
                        }
                        'l' | 'L' => {
                            if hints.iter().any(|hint| hint == &FormatHint::Lowercase) {
                                return Err(syn::Error::new_spanned(
                                    fmt,
                                    "Using 'l' or 'L' (lowercase) hint when lowercase hint is already given",
                                )
                                .to_compile_error()
                                .into());
                            }
                            hints.push(FormatHint::Lowercase);
                        }
                        _ => {
                            return Err(syn::Error::new_spanned(
                                fmt,
                                format!("Unknown format hint: {}", part),
                            )
                            .to_compile_error()
                            .into());
                        }
                    }
                }
            }
        }

        Ok(FormatType { method, hints })
    }
}

fn parse_fmt_string(fmt: &str) -> Result<Vec<FormatPart>, TokenStream> {
    let mut parts = Vec::new();

    let mut part = String::new();
    let mut in_placeholder = false;
    let mut escape = false;
    for c in fmt.chars() {
        if escape {
            escape = false;
            part.push(c);
            continue;
        } else if c == '{' && !in_placeholder {
            in_placeholder = true;
            if !part.is_empty() {
                parts.push(FormatPart::Text(part));
                part = String::new();
            }
            continue;
        } else if c == '}' && in_placeholder {
            in_placeholder = false;
            parts.push(FormatPart::Placeholder(parse_fmt_placeholder(&part)?));
            part = String::new();
            continue;
        } else if c == '%' {
            escape = true;
            continue;
        } else {
            part.push(c);
        }
    }
    if !part.is_empty() {
        parts.push(FormatPart::Text(part));
    }

    Ok(parts)
}

#[proc_macro]
pub fn kwrite_to_raw(input: TokenStream) -> TokenStream {
    let WriteInput {
        writer,
        format,
        args,
        trait_format,
        trait_debug,
        trait_writeable,
        format_fn,
        debug_fn,
        error_type,
        ..
    } = parse_macro_input!(input as WriteInput);

    let args = args.iter().cloned().collect::<Vec<_>>();

    // Extract the format string as a plain string
    let format_str = format.value();

    let parsed_fmt = match parse_fmt_string(&format_str) {
        Ok(fmt) => fmt,
        Err(e) => {
            return e;
        }
    };

    let placeholders = parsed_fmt
        .iter()
        .filter_map(|part| match part {
            FormatPart::Placeholder(p) => Some(p.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();

    if placeholders.len() != args.len() {
        return syn::Error::new_spanned(
            format,
            format!(
                "Expected {} arguments for format string, but found {}",
                placeholders.len(),
                args.len()
            ),
        )
        .to_compile_error()
        .into();
    }

    let mut combined = proc_macro2::TokenStream::new();
    let mut count_placeholders = 0;

    for part in parsed_fmt.iter() {
        let expanded = match part {
            FormatPart::Text(t) => {
                let value = t.clone();
                quote! {
                    __result += (write_single!(#value, #trait_format, #format_fn, *__writer, #trait_writeable, core::option::Option::None, core::option::Option::None, core::option::Option::None, core::option::Option::None, core::option::Option::None))?;
                }
            }
            FormatPart::Placeholder(p) => {
                let mut hint_pretty = quote! { core::option::Option::None };
                let mut hint_radix = quote! { core::option::Option::None };
                let mut hint_width = quote! { core::option::Option::None };
                let mut hint_precision = quote! { core::option::Option::None };
                let mut hint_case = quote! { core::option::Option::None };
                for hint in p.hints.iter() {
                    match hint {
                        FormatHint::Lowercase => {
                            hint_case = quote! { core::option::Option::Some(false) };
                        }
                        FormatHint::Uppercase => {
                            hint_case = quote! { core::option::Option::Some(true) };
                        }
                        FormatHint::Pretty => {
                            hint_pretty = quote! { core::option::Option::Some(true) };
                        }
                        FormatHint::Radix(r) => {
                            let radix = *r;
                            hint_radix = quote! { core::option::Option::Some(#radix) };
                        }
                        FormatHint::Width(w) => {
                            let width = *w;
                            hint_width = quote! { core::option::Option::Some(#width) };
                        }
                        FormatHint::Precision(p) => {
                            let prec = *p;
                            hint_precision = quote! { core::option::Option::Some(#prec) };
                        }
                    }
                }
                match p.method {
                    FormatMethod::Display => {
                        let value = args.get(count_placeholders).unwrap();
                        count_placeholders += 1;
                        quote! {
                            __result += (write_single!((#value), #trait_format, #format_fn, *__writer, #trait_writeable, #hint_pretty, #hint_radix, #hint_width, #hint_precision, #hint_case))?;
                        }
                    }
                    FormatMethod::Debug => {
                        let value = args.get(count_placeholders).unwrap();
                        count_placeholders += 1;
                        quote! {
                            __result += (write_single!((#value), #trait_debug, #debug_fn, *__writer, #trait_writeable, #hint_pretty, #hint_radix, #hint_width, #hint_precision, #hint_case))?;
                        }
                    }
                }
            }
        };
        combined.extend(expanded);
    }

    let result = quote! {
        {
            let __closure: &dyn core::ops::Fn(&mut dyn #trait_writeable) -> core::result::Result<usize, #error_type> = &(|__writer| {
                use kformat_macros::write_single;
                let mut __result: usize = 0;
                #combined
                Ok(__result)
            });
            __closure(&mut #writer)
        }
    };

    TokenStream::from(result)
}
