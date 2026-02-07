use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, Data, DataStruct, DeriveInput, Error, Expr,
    ExprLit, Fields, GenericArgument, Ident, Lit, Meta, MetaNameValue, PathArguments, Result, Type,
};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match generate(&input) {
        Ok(t) => t.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

fn generate(input: &DeriveInput) -> Result<TokenStream> {
    let struct_name = &input.ident;
    let builder_name = format_ident!("{}Builder", struct_name);

    let Data::Struct(DataStruct { fields, .. }) = &input.data else {
        panic!("");
    };

    let builder_struct = add_builder_struct(&builder_name, fields)?;
    let builder_function = add_builder_function(&builder_name, fields)?;
    let build_function = add_build_function(&input.data, &struct_name)?;
    let field_builders = add_field_builders(&input.data)?;

    let expanded = quote! {
        impl #struct_name {
            #builder_function
        }

        #builder_struct

        impl #builder_name {
            #build_function

            #field_builders
        }
    };

    Ok(expanded)
}

fn add_builder_struct(builder_name: &Ident, fields: &Fields) -> Result<TokenStream> {
    match fields {
        Fields::Named(fields) => {
            let struct_fields = fields
                .named
                .iter()
                .map(|f| {
                    let f_name = &f.ident;
                    let f_type = &f.ty;
                    let has_attr_each = try_extract_builder_attr_each(&f.attrs)?;

                    if let Some(_) = has_attr_each {
                        Ok(quote_spanned! { f.span() =>
                            #f_name: #f_type
                        })
                    } else if check_is_option(f_type) {
                        Ok(quote_spanned! { f.span() =>
                            #f_name: #f_type
                        })
                    } else {
                        Ok(quote_spanned! { f.span() =>
                            #f_name: std::option::Option<#f_type>
                        })
                    }
                })
                .collect::<Result<Vec<_>>>()?;

            Ok(quote! {
                pub struct #builder_name {
                    # (#struct_fields,)*
                }
            })
        }
        _ => unimplemented!(),
    }
}

fn add_builder_function(builder_name: &Ident, fields: &Fields) -> Result<TokenStream> {
    match fields {
        Fields::Named(fields) => {
            let fields_assignments = fields
                .named
                .iter()
                .map(|f| {
                    let f_name = &f.ident;
                    let has_attr_each = try_extract_builder_attr_each(&f.attrs)?;

                    if let Some(_) = has_attr_each {
                        Ok(quote_spanned! { f.span() =>
                            #f_name: vec![]
                        })
                    } else {
                        Ok(quote_spanned! { f.span() =>
                            #f_name:  std::option::Option::None
                        })
                    }
                })
                .collect::<Result<Vec<_>>>()?;

            Ok(quote! {
                pub fn builder() -> #builder_name {
                    #builder_name {
                        # (#fields_assignments,)*
                    }
                }
            })
        }
        _ => unimplemented!(),
    }
}

fn add_build_function(data: &Data, return_type: &Ident) -> Result<TokenStream> {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let not_none_checks = fields
                    .named
                    .iter()
                    .map(|f| {
                        let f_name = &f.ident;
                        let f_type = &f.ty;
                        let err_msg = format!("field {:?} was none!", f_name);
                        let has_attr_each = try_extract_builder_attr_each(&f.attrs)?;

                        if let Some(_) = has_attr_each {
                            Ok(quote_spanned! { f.span() =>
                                let #f_name = std::mem::take(&mut self.#f_name);
                            })
                        } else if check_is_option(f_type) {
                            Ok(quote_spanned! { f.span() =>
                                let #f_name = self.#f_name.take();
                            })
                        } else {
                            Ok(quote_spanned! { f.span() =>
                                let #f_name = self.#f_name.take().expect(#err_msg);
                            })
                        }
                    })
                    .collect::<Result<Vec<_>>>()?;

                let fields_assignments = fields.named.iter().map(|f| {
                    let f_name = &f.ident;

                    quote_spanned! { f.span() =>
                        #f_name
                    }
                });

                Ok(quote! {
                    pub fn build(&mut self) -> ::std::result::Result<#return_type, ::std::boxed::Box<dyn ::std::error::Error>> {
                        # (#not_none_checks)*

                        std::result::Result::Ok(#return_type {
                            # (#fields_assignments,)*
                        })
                    }
                })
            }
            _ => unimplemented!("Only named fields are supported!"),
        },
        _ => unimplemented!("Only structs are supported!"),
    }
}

fn add_field_builders(data: &Data) -> Result<TokenStream> {
    match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => {
                let setters = fields.named.iter().map(|f| {
                    let f_name = &f.ident;
                    let f_type = &f.ty;
                    let has_attr_each = try_extract_builder_attr_each(&f.attrs)?;

                    if let Some(f_name_override) = has_attr_each {
                        let inner_vec_type = get_inner_vec_type(f_type).unwrap();

                        Ok(quote_spanned! { f.span() =>
                            fn #f_name_override(&mut self, #f_name_override: #inner_vec_type) -> &mut Self {
                                self.#f_name.push(#f_name_override);
                                self
                            }
                        })
                    }
                    else if check_is_option(f_type) {
                        let inner_opt_type = get_inner_option_type(f_type).unwrap();

                        Ok(quote_spanned! { f.span() =>
                            fn #f_name(&mut self, #f_name: #inner_opt_type) -> &mut Self {
                                self.#f_name = ::std::option::Option::Some(#f_name);
                                self
                            }
                        })
                    } else {
                        Ok(quote_spanned! { f.span() =>
                            fn #f_name(&mut self, #f_name: #f_type) -> &mut Self {
                                self.#f_name = ::std::option::Option::Some(#f_name);
                                self
                            }
                        })
                    }
                }).collect::<Result<Vec<_>>>()?;

                Ok(quote! {
                    # ( #setters)*
                })
            }
            _ => unimplemented!("Only named fields are supported!"),
        },
        _ => unimplemented!("Only structs are supported!"),
    }
}

fn check_is_option(ty: &Type) -> bool {
    matches!(ty, Type::Path(tp) if tp.path.segments.last().is_some_and(|seg| seg.ident == "Option"))
}

fn check_is_vec(ty: &Type) -> bool {
    matches!(ty, Type::Path(tp) if tp.path.segments.last().is_some_and(|seg| seg.ident == "Vec"))
}

fn get_inner_option_type(ty: &Type) -> Option<&Type> {
    if check_is_option(ty) {
        let Type::Path(type_path) = ty else {
            return None;
        };

        let segment = &type_path.path.segments[0];

        if let PathArguments::AngleBracketed(args) = &segment.arguments {
            if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                return Some(inner_type);
            }
        }
    }

    None
}

fn get_inner_vec_type(ty: &Type) -> Option<&Type> {
    if check_is_vec(ty) {
        if let Type::Path(type_path) = ty {
            if let Some(segment) = type_path.path.segments.last() {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                        return Some(inner_type);
                    }
                }
            }
        }
    }

    None
}

fn try_extract_builder_attr_each(attrs: &Vec<Attribute>) -> Result<Option<Ident>> {
    for attr in attrs {
        if attr.path().is_ident("builder") {
            if let Meta::List(meta) = &attr.meta {
                let name_value: MetaNameValue =
                    meta.parse_args().map_err(|e| Error::new_spanned(meta, e))?;

                if name_value.path.is_ident("each") {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }) = name_value.value
                    {
                        return Ok(Some(Ident::new(&s.value(), Span::call_site())));
                    }
                } else {
                    return Err(Error::new_spanned(
                        meta,
                        "expected `builder(each = \"...\")`",
                    ));
                }
            }
        }
    }

    Ok(None)
}
