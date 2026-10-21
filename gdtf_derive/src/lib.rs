use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Attribute};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum RootMode {
    Start,
    #[default]
    None,
    Update,
}

fn get_root_mode(attrs: &[Attribute]) -> RootMode {
    for attr in attrs {
        if !attr.path().is_ident("verify") {
            continue;
        }

        let mut found_mode = None;

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("push_to_roots") || meta.path.is_ident("push_to_root")
                || meta.path.is_ident("root") || meta.path.is_ident("roots")
            {
                if meta.input.peek(syn::Token![=]) {
                    let value = meta.value()?;
                    let value_str = if let Ok(ident) = value.parse::<syn::Ident>() {
                        ident.to_string()
                    } else if let Ok(lit) = value.parse::<syn::LitStr>() {
                        lit.value()
                    } else {
                        return Ok(());
                    };

                    found_mode = Some(match value_str.as_str() {
                        "start" | "initial" | "at_start" | "once"  => RootMode::Start,
                        "none" | "never" | "off" | "false" => RootMode::None,
                        "update" | "always" | "refresh" | "each" => RootMode::Update,
                        other => {
                            panic!("unknown push_to_roots mode `{}`; expected `start`, `none`, or `update`", other);
                        }
                    });
                } else {
                    found_mode = Some(RootMode::Start);
                }
            }
            Ok(())
        });

        if let Some(mode) = found_mode {
            return mode;
        }
    }

    RootMode::None
}

fn get_verify_self_position(attrs: &[Attribute]) -> Option<VerifySelfPosition> {
    for attr in attrs {
        if !attr.path().is_ident("verify") {
            continue;
        }

        let mut position = None;

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("verify_self") {
                if meta.input.peek(syn::Token![=]) {
                    let value = meta.value()?;
                    let ident = value.parse::<syn::Ident>()?;

                    position = Some(match ident.to_string().as_str() {
                        "before_all" => VerifySelfPosition::BeforeAll,
                        "after_uniqueness" => VerifySelfPosition::AfterUniqueness,
                        "after_all" => VerifySelfPosition::AfterAll,
                        other => {
                            panic!("unknown verify_self position `{}`; expected `before_all`, `after_uniqueness` or `after_all`", other);
                        }
                    });
                } else {
                    position = Some(VerifySelfPosition::BeforeAll);
                }
            }
            Ok(())
        });

        if let Some(pos) = position {
            return Some(pos);
        }
    }
    None
}

enum VerifySelfPosition {
    BeforeAll,
    AfterUniqueness,
    AfterAll,
}

fn get_uniqueness_field(attrs: &[Attribute]) -> Option<syn::Ident> {
    for attr in attrs {
        if !attr.path().is_ident("verify") {
            continue;
        }

        let mut field = None;

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("uniqueness") {
                let value = meta.value()?;
                field = Some(value.parse::<syn::Ident>()?);
            }

            Ok(())
        });

        if let Some(f) = field {
            return Some(f);
        }
    }

    None
}

fn get_vec_inner_type(input: &DeriveInput, field_ident: &syn::Ident) -> Option<syn::Ident> {
    let Data::Struct(data) = &input.data else {
        return None;
    };

    let Fields::Named(fields) = &data.fields else {
        return None;
    };

    let field = fields.named.iter().find(|f|
        f.ident.as_ref() == Some(field_ident)
    )?;

    let syn::Type::Path(type_path) = &field.ty else {
        return None;
    };

    let segment = type_path.path.segments.last()?;

    if segment.ident != "Vec" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    let syn::GenericArgument::Type(syn::Type::Path(inner_type)) = args.args.first()? else {
        return None;
    };

    inner_type.path.segments.last().map(|segment| segment.ident.clone())
}

#[proc_macro_derive(GdtfVerify, attributes(verify))]
pub fn gdtf_verify_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident.clone();
    let root_mode = get_root_mode(&input.attrs);

    let wrap_update = |stmt: proc_macro2::TokenStream| -> proc_macro2::TokenStream {
        if root_mode == RootMode::Update {
            quote! {
                roots.push(Box::new(self.clone()));
                #stmt
                roots.pop::<#name>();
            }
        } else {
            stmt
        }
    };

    let mut verify_self_before = quote!{};
    let mut verify_self_after_uniqueness = quote!{};
    let mut verify_self_after_all = quote!{};

    let self_verify_quote = quote! {
        warnings.extend(self.verify_self(roots)?);
    };

    let self_verify_quote = wrap_update(self_verify_quote);

    match get_verify_self_position(&input.attrs) {
        Some(VerifySelfPosition::BeforeAll) => {
            verify_self_before = self_verify_quote;
        }
        Some(VerifySelfPosition::AfterUniqueness) => {
            verify_self_after_uniqueness = self_verify_quote;
        }
        Some(VerifySelfPosition::AfterAll) => {
            verify_self_after_all = self_verify_quote;
        }
        None => {}
    }

    let uniquenes_quote = if let Some(field) = get_uniqueness_field(&input.attrs) {
        let item_type = get_vec_inner_type(&input, &field)
            .unwrap_or_else(|| {
                panic!("field `{}` used for uniqueness must have type Vec<T>", field)
            });

        let item_type_name = item_type.to_string();
        let struct_name = name.to_string();

        quote! {
            let mut seen = std::collections::HashSet::new();

            for item in &self.#field {
                let name = item.name();
                if !seen.insert(name.clone()) {
                    return Err(format!(
                         "{} name '{}' is not unique within {}", #item_type_name, name, #struct_name
                    ));
                }
            }
        }
    } else {
        quote! {}
    };

    let push_start = if root_mode == RootMode::Start {
        quote! { roots.push(Box::new(self.clone())); }
    } else {
        quote! {}
    };

    let pop_end = if root_mode == RootMode::Start {
        quote! { roots.pop::<#name>(); }
    } else {
        quote! {}
    };

    let expanded = match input.data {
        Data::Struct(data) => {
            match data.fields {
                Fields::Named(fields) => {
                    let verify_statements: Vec<_> = fields.named.iter().map(|f| {
                        let ident = f.ident.as_ref().unwrap();
                        let inner_stmt = quote! {
                            warnings.extend(self.#ident.verify(roots, archive)?);
                        };
                        wrap_update(inner_stmt)
                    }).collect();

                    quote! {
                        impl Verify for #name {
                            fn verify(
                                &mut self, roots: &mut crate::parsing_helpers::verification::VerificationRoots,
                                archive: &mut ZipArchive<Cursor<bytes::Bytes>>
                            ) -> Result<Vec<crate::parsing_helpers::verification::VerifyWarning>, String> {
                                let mut warnings = vec![];
                                #push_start

                                #verify_self_before

                                #uniquenes_quote

                                #verify_self_after_uniqueness

                                #(#verify_statements)*

                                #verify_self_after_all

                                #pop_end
                                Ok(warnings)
                            }
                            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {self}
                        }
                    }
                },
                Fields::Unnamed(fields) => {
                    let verify_statements: Vec<_> = (0..fields.unnamed.len())
                        .map(|i| {
                            let index = syn::Index::from(i);
                            let inner_stmt = quote! {
                                warnings.extend(self.#index.verify(roots, archive)?);
                            };
                            wrap_update(inner_stmt)
                        })
                        .collect();
                    quote! {
                        impl Verify for #name {
                            fn verify(
                                &mut self, roots: &mut crate::parsing_helpers::verification::VerificationRoots,
                                archive: &mut ZipArchive<Cursor<bytes::Bytes>>
                            ) -> Result<Vec<crate::parsing_helpers::verification::VerifyWarning>, String> {
                                let mut warnings = vec![];
                                #push_start

                                #verify_self_before

                                #uniquenes_quote

                                #verify_self_after_uniqueness

                                #(#verify_statements)*

                                #verify_self_after_all

                                #pop_end
                                Ok(warnings)
                            }
                            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {self}
                        }
                    }
                }

                Fields::Unit => {
                    quote! {
                        impl Verify for #name {
                            fn verify(
                                &mut self, roots: &mut crate::parsing_helpers::verification::VerificationRoots,
                                archive: &mut ZipArchive<Cursor<bytes::Bytes>>
                            ) -> Result<Vec<crate::parsing_helpers::verification::VerifyWarning>, String> {
                                Ok(vec![])
                            }

                            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {self}
                        }
                    }
                }
            }
        },

        Data::Enum(data) => {
            let mut match_arms = Vec::new();
            for variant in data.variants {
                let variant_ident = variant.ident;
                match variant.fields {
                    Fields::Named(fields) => {
                        let field_names: Vec<_> = fields.named
                            .iter().map(|f| f.ident.clone().unwrap())
                            .collect();

                        if root_mode == RootMode::Update {
                            let arm_statements: Vec<_> = field_names.iter().map(|name| {
                                quote! {
                                    roots.push(Box::new(self.clone()));
                                    warnings.extend(#name.verify(roots, archive)?);
                                    roots.pop::<#name>();
                                }
                            }).collect();

                            match_arms.push(quote! {
                                Self::#variant_ident { #(#field_names),* } => {
                                    #(#arm_statements)*
                                }
                            });
                        } else {
                            match_arms.push(quote! {
                                Self::#variant_ident { #(#field_names),* } => {
                                    #(warnings.extend(#field_names.verify(roots, archive)?);)*
                                }
                            });
                        }
                    }

                    Fields::Unnamed(fields) => {
                        let vars: Vec<_> = (0..fields.unnamed.len())
                            .map(|i| syn::Ident::new(&format!("v{}", i), proc_macro2::Span::call_site()))
                            .collect();

                        if root_mode == RootMode::Update {
                            let arm_statements: Vec<_> = vars.iter().map(|var| {
                                quote! {
                                    roots.push(Box::new(self.clone()));
                                    warnings.extend(#var.verify(roots, archive)?);
                                    roots.pop:<#name>();
                                }
                            }).collect();

                            match_arms.push(quote! {
                                Self::#variant_ident(#(#vars),*) => {
                                    #(#arm_statements)*
                                }
                            });
                        } else {
                            match_arms.push(quote! {
                                Self::#variant_ident(#(#vars),*) => {
                                    warnings.extend(#(#vars.verify(roots, archive)?)*)
                                }
                            });
                        }
                    }

                    Fields::Unit => {
                        match_arms.push(quote! {
                            Self::#variant_ident => {}
                        })
                    }
                }
            }

            quote! {
                impl Verify for #name {
                    fn verify(
                        &mut self, roots: &mut crate::parsing_helpers::verification::VerificationRoots,
                        archive: &mut ZipArchive<Cursor<bytes::Bytes>>
                    ) -> Result<Vec<crate::parsing_helpers::verification::VerifyWarning>, String> {
                        let mut warnings = vec![];
                        #push_start

                        #verify_self_before

                        #uniquenes_quote

                        #verify_self_after_uniqueness

                        match self {
                            #(#match_arms)*
                        }

                        #verify_self_after_all

                        #pop_end
                        Ok(warnings)
                    }
                    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {self}
                }
            }
        }

        _ => panic!("GdtfVerify can only be derived for structs and enums."),
    };

    TokenStream::from(expanded)
}