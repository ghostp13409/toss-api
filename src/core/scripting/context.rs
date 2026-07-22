pub fn get_client_shim(env_json: &str, headers_json: &str) -> String {
    format!(r#"
        const __env_array = {};
        const __headers_array = {};

        const __env_vars = {{}};
        for (const item of __env_array) {{
            __env_vars[item.key] = item;
        }}

        const __headers = {{}};
        for (const item of __headers_array) {{
            __headers[item.key] = item;
        }}

        const client = {{
            environment: {{
                set: function(k, v) {{
                    __env_vars[k] = {{ key: k, value: String(v), enabled: true, description: null }};
                }},
                get: function(k) {{
                    return __env_vars[k] ? __env_vars[k].value : undefined;
                }},
                unset: function(k) {{
                    delete __env_vars[k];
                }}
            }},
            globals: {{
                set: function(k, v) {{ client.environment.set(k, v); }},
                get: function(k) {{ return client.environment.get(k); }},
                unset: function(k) {{ client.environment.unset(k); }}
            }},
            variables: {{
                set: function(k, v) {{ client.environment.set(k, v); }},
                get: function(k) {{ return client.environment.get(k); }}
            }},
            request: {{
                headers: {{
                    add: function(k, v) {{
                        __headers[k] = {{ key: k, value: String(v), enabled: true, description: null }};
                    }},
                    remove: function(k) {{
                        delete __headers[k];
                    }}
                }}
            }},
            test: function(name, fn) {{
                try {{
                    fn();
                    __test_results.push({{ name: name, passed: true, error: null }});
                }} catch(e) {{
                    __test_results.push({{ name: name, passed: false, error: String(e) }});
                }}
            }},
            expect: function(val) {{
                return {{
                    to: {{
                        equal: function(expected) {{
                            if (val !== expected) throw new Error("Expected " + expected + " but got " + val);
                        }}
                    }}
                }};
            }}
        }};
    "#, env_json, headers_json)
}
