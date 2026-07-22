use boa_engine::{Context, JsResult, JsValue, JsArgs, NativeFunction, property::Attribute, object::ObjectInitializer, js_string};
use sha2::{Sha256, Digest};
use md5::Md5;
use hmac::{Hmac, Mac, KeyInit};
use base64::{Engine as _, engine::general_purpose::STANDARD};

type HmacSha256 = Hmac<Sha256>;

fn js_sha256(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let str = args.get_or_undefined(0).to_string(context)?.to_std_string().map_err(|e| boa_engine::JsError::from_opaque(JsValue::new(js_string!(e.to_string()))))?;
    let mut hasher = Sha256::new();
    hasher.update(str);
    let result = hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect::<String>();
    Ok(JsValue::new(js_string!(result)))
}

fn js_md5(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let str = args.get_or_undefined(0).to_string(context)?.to_std_string().map_err(|e| boa_engine::JsError::from_opaque(JsValue::new(js_string!(e.to_string()))))?;
    let mut hasher = Md5::new();
    hasher.update(str);
    let result = hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect::<String>();
    Ok(JsValue::new(js_string!(result)))
}

fn js_hmac_sha256(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let msg = args.get_or_undefined(0).to_string(context)?.to_std_string().map_err(|e| boa_engine::JsError::from_opaque(JsValue::new(js_string!(e.to_string()))))?;
    let key = args.get_or_undefined(1).to_string(context)?.to_std_string().map_err(|e| boa_engine::JsError::from_opaque(JsValue::new(js_string!(e.to_string()))))?;
    
    let mut mac = HmacSha256::new_from_slice(key.as_bytes()).map_err(|_| boa_engine::JsError::from_opaque(JsValue::new(js_string!("Invalid key length"))))?;
    mac.update(msg.as_bytes());
    let result = mac.finalize().into_bytes().iter().map(|b| format!("{:02x}", b)).collect::<String>();
    Ok(JsValue::new(js_string!(result)))
}

fn js_base64_encode(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let str = args.get_or_undefined(0).to_string(context)?.to_std_string().map_err(|e| boa_engine::JsError::from_opaque(JsValue::new(js_string!(e.to_string()))))?;
    let result = STANDARD.encode(str);
    Ok(JsValue::new(js_string!(result)))
}

fn js_base64_decode(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let str = args.get_or_undefined(0).to_string(context)?.to_std_string().map_err(|e| boa_engine::JsError::from_opaque(JsValue::new(js_string!(e.to_string()))))?;
    let bytes = STANDARD.decode(str).map_err(|_| boa_engine::JsError::from_opaque(JsValue::new(js_string!("Invalid base64"))))?;
    let result = String::from_utf8(bytes).map_err(|_| boa_engine::JsError::from_opaque(JsValue::new(js_string!("Invalid utf8"))))?;
    Ok(JsValue::new(js_string!(result)))
}

pub fn register_crypto(context: &mut Context) -> JsResult<()> {
    let crypto = ObjectInitializer::new(context)
        .function(NativeFunction::from_fn_ptr(js_sha256), js_string!("sha256"), 1)
        .function(NativeFunction::from_fn_ptr(js_md5), js_string!("md5"), 1)
        .function(NativeFunction::from_fn_ptr(js_hmac_sha256), js_string!("hmacSha256"), 2)
        .function(NativeFunction::from_fn_ptr(js_base64_encode), js_string!("base64Encode"), 1)
        .function(NativeFunction::from_fn_ptr(js_base64_decode), js_string!("base64Decode"), 1)
        .build();
    
    context.register_global_property(js_string!("Crypto"), crypto, Attribute::all())?;
    Ok(())
}
