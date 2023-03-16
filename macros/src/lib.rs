use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, Ident, Pat};

fn get_name(func_argument: &FnArg) -> Ident {
    match func_argument {
        FnArg::Typed(fn_type) => match &*fn_type.pat {
            Pat::Ident(ident) => return ident.ident.clone(),
            _ => {
                panic!("Not supported function argument: {:?}", func_argument)
            }
        },
        _ => {
            panic!("Not supported function argument: {:?}", func_argument)
        }
    }
}

#[proc_macro_attribute]
pub fn callback(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut func = syn::parse::<syn::ItemFn>(item).expect("expected a function");
    let name = &func.sig.ident;
    let all_inputs = func.sig.inputs.iter().collect::<Vec<_>>();
    assert_eq!(all_inputs.len(), 4);
    let context_var_name = get_name(all_inputs.get(0).unwrap());
    let function_var_name = get_name(all_inputs.get(1).unwrap());
    let this_var_name = get_name(all_inputs.get(2).unwrap());
    let args_var_name = get_name(all_inputs.get(3).unwrap());
    // println!("first_name {:?}", first_name);

    let block = &func.block;
    let attrs = func.attrs;
    let result = quote! {
        unsafe extern "C" fn #name(
            __base_ctx: rusty_jsc::private::JSContextRef,
            __function: rusty_jsc::private::JSObjectRef,
            __this_object: rusty_jsc::private::JSObjectRef,
            __argument_count: rusty_jsc::private::size_t,
            __arguments: *const rusty_jsc::private::JSValueRef,
            mut __exception: *mut rusty_jsc::private::JSValueRef,
        ) -> rusty_jsc::private::JSValueRef {
            let #context_var_name = rusty_jsc::JSContext::from(__base_ctx);
            let #function_var_name: rusty_jsc::JSObject= __function.into();
            let #this_var_name: rusty_jsc::JSObject = __this_object.into();
            let #args_var_name = if __argument_count == 0 {
                vec![]
            }
            else {
                let __args_refs_slice = unsafe { std::slice::from_raw_parts(__arguments, __argument_count as _) };
                // drop(arguments, argument_count);
                // // println!("args_refs_slice {}", args_refs_slice.len());
                __args_refs_slice.iter().map(|r| (*r).into()).collect::<Vec<_>>()
            };
            let #args_var_name: &[JSValue] = &#args_var_name;
            // println!("ARG 0: {}", #args_var_name[0].to_string(&#context_var_name));

            let res: Result<JSValue, JSValue> = #block;
            match res {
                Ok(res) => res.into(),
                Err(err) => {
                    *__exception = err.into();
                    let ctx2 = rusty_jsc::JSContext::from(__base_ctx);
                    rusty_jsc::JSValue::undefined(&ctx2).into()
                }
            }
        }
    };
    let new_func = result.into();
    let mut new_func = syn::parse::<syn::ItemFn>(new_func).expect("expected a function");
    new_func.attrs = attrs.clone();
    new_func.vis = func.vis;
    new_func.sig.generics = func.sig.generics;
    new_func.sig.constness = func.sig.constness;
    new_func.sig.variadic = func.sig.variadic;
    new_func.sig.asyncness = func.sig.asyncness;

    let result2 = quote! {
        #new_func
    };
    result2.into()
}
