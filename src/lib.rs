use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Type, Data, DeriveInput, Field, Fields, LitInt, Result, Ident};

struct FieldOrder {
    pub order: i32,
    pub field_name: Ident,
    pub dtype: Type
}

///A procedural macro for deserializing ordered arrays into keyed structs using Serde.
#[proc_macro_derive(DeserializeOrdered, attributes(order))]
pub fn derive_order(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(named_field) => &named_field.named,
            _ => return syn::Error::new_spanned(
                &input,
                "DeserializeOrdered only supports structs with named fields",
            ).to_compile_error().into(),
        },
        _ => return syn::Error::new_spanned(
            &input,
            "DeserializeOrdered can only be derived for structs",
        ).to_compile_error().into(),
    };

    let mut field_orders = vec![];

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();

        // Extract #[serde(order = x)] attribute
        let order = match get_order_from_field(field) {
            Ok(order) => order,
            Err(err) => return err.to_compile_error().into(),
        };

        field_orders.push(FieldOrder {
            order,
            field_name: field_name.clone(),
            dtype: field.ty.clone(),
        });
    }

    // Check if every field has an order and that all orders are unique
    let total_fields = fields.len();
    if field_orders.len() != total_fields {
        return syn::Error::new_spanned(
            &input,
            "DeserializeOrdered requires all fields do have #[serde(order = x)]",
        ).to_compile_error().into();
    }

    //Check for duplicate orders
    let orders_set = field_orders.iter().map(|fo| fo.order).collect::<HashSet<_>>();
    if orders_set.len() != total_fields {
        return syn::Error::new_spanned(
            &input,
            "DeserializeOrdered requires all fields to have unique orders",
        ).to_compile_error().into();
    }

    // Sort fields by order index
    field_orders.sort_by_key(|order| order.order);
    let field_names: Vec<_> = field_orders.iter().map(|fo| fo.field_name.to_owned()).collect();
    let field_types: Vec<_> = field_orders.iter().map(|fo| fo.dtype.to_owned()).collect();
    let orders: Vec<_> = field_orders.iter().map(|fo| fo.order.to_owned()).collect();
    
    // Generate deserialization logic (same as before)
    let deserialization = quote! {
        impl<'de> serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                use serde::de::{SeqAccess, Visitor};
                use std::fmt;

                struct OrderedVisitor;

                impl<'de> Visitor<'de> for OrderedVisitor {
                    type Value = #name;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("a sequence with ordered fields")
                    }

                    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                    where
                        A: SeqAccess<'de>,
                    {
                        let mut index = 0;

                        #(
                            let mut #field_names: Option<#field_types> = None;
                        )*

                        while let Ok(element) = seq.next_element::<serde_value::Value>() {
                            if element.is_none() {break;}
                
                            let element = element.unwrap();
                            match index {
                                #(
                                    // #orders => #field_names = element.deserialize_into::<#field_types>().unwrap(),
                                    #orders => {
                                        let result = match element.deserialize_into::<#field_types>() {
                                            Ok(result) => result,
                                            Err(err) => 
                                                return Err(serde::de::Error::custom(format!("Failed to deserialize key because {:?}", err))),
                                        };
                    
                                        #field_names = Some(result);
                                    },
                                )*
                                _ => {}
                            }
                
                            index+=1;
                        }

                        #(
                            let #field_names: #field_types = match #field_names {
                                Some(result) => result,
                                None => return Err(serde::de::Error::custom("Order was outside the bounds of the message")),
                            };
                        )*

                        Ok(#name {
                            #(#field_names),*
                        })
                    }
                }

                deserializer.deserialize_seq(OrderedVisitor)
            }
        }
    };

    TokenStream::from(deserialization)
}

//Grabs the order from #[order = ...]
fn get_order_from_field(field: &Field) -> Result<i32> {
    for attribute in &field.attrs {
        if attribute.path().is_ident("order") {
            let order: LitInt = attribute.parse_args()?;
            return Ok(order.base10_parse::<i32>()?);
        }
    }

    Err(syn::Error::new_spanned(
        field,
        "No `order` attribute found, which is required for DeserializeOrdered",
    ))
}