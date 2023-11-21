use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, *};

#[proc_macro_attribute]
pub fn precompile(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item: TokenStream = item.into();
    let attr: TokenStream = attr.into();

    let Ok(item) = parse2::<ItemFn>(item) else {
        return quote! {
            ::core::compile_error!("expected function");
        }
        .into();
    };

    if !attr.is_empty() {
        return quote! { ::core::compile_error!("#[precompile::precompile] does not take any arguments
        use #[precompile_with(A, B, ...)] to add types."); }
        .into();
    };

    let ItemFn {
        mut attrs,
        vis,
        mut sig,
        block: _,
    } = item.clone();

    let mut new_attrs = Vec::new();
    let mut types: Vec<Vec<_>> = Vec::new();

    for attr in attrs {
        match attr.meta.clone() {
            Meta::List(list) => {
                if list
                    .path
                    .get_ident()
                    .map(|ident| &*ident.to_string() == "precompile_with")
                    == Some(true)
                {
                    let tokens = list.tokens.clone();
                    if let Ok(ty) = parse2::<Type>(tokens) {
                        types.push(vec![ty]);
                    } else {
                        let tokens = list.tokens;
                        let Ok(ty_tuple) = parse2::<TypeTuple>(quote! { (#tokens) }) else {
                            return quote! { ::core::compile_error!("expected comma-separated list of types"); }.into();
                        };
                        types.push(ty_tuple.elems.into_iter().collect());
                    };
                } else {
                    new_attrs.push(attr);
                }
            }
            _ => new_attrs.push(attr),
        }
    }
    attrs = new_attrs;

    let mut inner = item;
    inner.attrs = attrs.clone();
    inner.sig.ident = Ident::new("__precompile_inner_impl", Span::call_site());

    let Generics {
        lt_token: _,
        params,
        gt_token: _,
        where_clause,
    } = inner.sig.generics.clone();

    let mut ty_param = Vec::new();
    let mut ty_param_names = Vec::new();

    for param in params.clone().into_iter() {
        match param {
            GenericParam::Type(ty) => {
                ty_param_names.push(ty.ident.clone());
                ty_param.push(ty);
            }
            GenericParam::Lifetime(_) => {}
            GenericParam::Const(_) => {
                return quote! { ::core::compile_error!("precompiling const generics is currently unsupported"); }.into();
            }
        }
    }

    let mut inputs = Punctuated::new();
    let mut input_name = Vec::new();
    let mut input_types = Vec::new();
    let output_type = sig.output.clone();

    for (idx, input) in sig.inputs.clone().into_iter().enumerate() {
        match input {
            FnArg::Receiver(_) => {
                return quote! { ::core::compile_error!("only free functions can be precompiled"); }
                    .into()
            }
            FnArg::Typed(mut ty) => {
                let ident = Ident::new(&format!("__{idx}"), Span::call_site());
                ty.pat = Box::new(Pat::Ident(PatIdent {
                    attrs: Vec::new(),
                    by_ref: None,
                    mutability: None,
                    ident: ident.clone(),
                    subpat: None,
                }));
                let tyty = (*ty.ty).clone();
                input_types.push(quote! { #tyty });
                inputs.push_value(FnArg::Typed(ty));
                inputs.push_punct(Comma {
                    spans: [Span::call_site()],
                });
                input_name.push(ident);
            }
        }
    }
    sig.inputs = inputs.clone();

    let spec_code = types
        .into_iter()
        .map(|types| {
            let inner_no_generics = inner.clone();
            let ItemFn {
                attrs,
                vis: _,
                mut sig,
                block: _,
            } = inner_no_generics;

            let Generics {
                lt_token, gt_token, ..
            } = sig.generics;

            sig.inputs = inputs.clone();

            sig.ident = Ident::new("__precompile_inner_impl_spec", Span::call_site());
            sig.generics = Generics {
                lt_token,
                params: Punctuated::new(),
                gt_token,
                where_clause: None,
            };

            quote! {
                #(type #ty_param_names = #types;)*
                impl ::precompile::Impl for __PrecompileImplSpec<(#(#types,)*)> {
                    const FN_PTR: *const () = {
                        #(#attrs)*
                        pub #sig {
                            #[allow(unused_unsafe)]
                            unsafe { __precompile_inner_impl::<#(#types,)*>(#(#input_name,)*) }
                        }
                        __precompile_inner_impl_spec as *const ()
                    };
                }
            }
        })
        .collect::<Vec<_>>();

    let abi = sig.abi.clone();

    let code = quote! {
        #(#attrs)*
        #vis #sig {
            #inner

            struct __PrecompileImplGeneric<Inner>(::core::marker::PhantomData<Inner>);
            struct __PrecompileImplSpec<Inner>(::core::marker::PhantomData<Inner>);

            impl<#(#ty_param,)*> ::precompile::Impl for __PrecompileImplGeneric<(#(#ty_param_names,)*)> #where_clause {
                const FN_PTR: *const () = __precompile_inner_impl::<#(#ty_param_names,)*> as *const ();
            }

            #({ #spec_code })*

            unsafe {
                use ::precompile::Impl;
                let mut __fn_ptr: unsafe #abi fn(#(#input_types,)*) #output_type = ::core::mem::transmute(::precompile::pick(
                    __PrecompileImplGeneric(::core::marker::PhantomData::<(#(#ty_param_names,)*)>),
                    __PrecompileImplSpec   (::core::marker::PhantomData::<(#(#ty_param_names,)*)>),
                ));

                __fn_ptr(#(#input_name,)*)
            }
        }
    };
    code.into()
}
