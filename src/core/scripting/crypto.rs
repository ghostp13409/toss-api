pub fn get_crypto_shim() -> &'static str {
    r#"
        const Crypto = {
            sha256: function(str) { return "mock_sha256_" + str; },
            hmacSha256: function(str, key) { return "mock_hmac_" + str; },
            md5: function(str) { return "mock_md5_" + str; },
            base64Encode: function(str) { return btoa(str); },
            base64Decode: function(str) { return atob(str); }
        };
    "#
}
