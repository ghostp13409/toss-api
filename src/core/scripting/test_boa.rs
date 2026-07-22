use boa_engine::{Context, JsResult, JsValue, JsArgs, NativeFunction, property::Attribute, object::ObjectInitializer, js_string};
fn test_func(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    Ok(JsValue::undefined())
}
pub fn test_compile(context: &mut Context) {
    let obj = ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(test_func), js_string!("test"), 0)
        .build();
    let _ = context.register_global_property(js_string!("Test"), obj, Attribute::all());
}
