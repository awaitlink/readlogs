use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, spanned::Spanned, AttributeArgs, FnArg, ItemFn, Lit, Meta, NestedMeta, Path,
};

#[proc_macro_attribute]
pub fn traceable_configurable_parser(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as AttributeArgs);
    let input = parse_macro_input!(input as ItemFn);

    let TraceableConfigurableParserArguments { debug } = parse_arguments(attr);

    let vis = input.vis;

    let sig = input.sig.clone();
    let name = input.sig.ident;

    let block = input.block;
    let name_expr_suffix = if debug {
        let mut suffix = quote! { + "(" };

        for arg in input.sig.inputs.iter() {
            match arg {
                FnArg::Receiver(_) => panic!(
                    "#[traceable_configurable_parser] is not supported for functions that use `self`"
                ),
                FnArg::Typed(arg) => {
                    let span = arg.ty.span();
                    let arg_name = arg.pat.clone();

                    suffix = quote_spanned! {span=>
                        #suffix
                        + &::std::format!("{:?}, ", #arg_name)
                    }
                },
            }
        }

        quote! { #suffix + ")" }
    } else {
        quote! { + "(...)" }
    };

    let result = quote! {
        #vis #sig {
            move |input| {
                #[cfg(feature = "trace")]
                let __traceable_configurable_parser_name = ::std::stringify!(#name).to_string() #name_expr_suffix;

                #[cfg(feature = "trace")]
                let (__traceable_configurable_parser_depth, input) = ::nom_tracable::forward_trace(
                    input,
                    &__traceable_configurable_parser_name
                );

                let output = #block;

                #[cfg(feature = "trace")]
                return ::nom_tracable::backward_trace(
                    output,
                    &__traceable_configurable_parser_name,
                    __traceable_configurable_parser_depth
                );

                #[cfg(not(feature = "trace"))]
                return output;
            }
        }
    };

    result.into()
}

struct TraceableConfigurableParserArguments {
    pub debug: bool,
}

fn parse_arguments(attr: AttributeArgs) -> TraceableConfigurableParserArguments {
    let mut result = TraceableConfigurableParserArguments { debug: true };

    for expr in attr {
        if let NestedMeta::Meta(Meta::NameValue(expr)) = expr {
            if path_equals(&expr.path, "debug") {
                if let Lit::Bool(expr) = &expr.lit {
                    result.debug = expr.value;
                } else {
                    panic!("`debug` value should be a `bool` literal")
                }
            } else {
                panic!(
                    "unexpected argument: `{}`",
                    expr.path
                        .segments
                        .iter()
                        .map(|segment| segment.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::")
                )
            }
        } else {
            panic!("unexpected meta inside `traceable_configurable_parser(...)`")
        }
    }

    result
}

fn path_equals(path: &Path, s: &str) -> bool {
    path.segments.len() == 1 && path.segments.last().unwrap().ident == s
}
