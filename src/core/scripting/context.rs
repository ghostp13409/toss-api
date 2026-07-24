pub fn get_client_shim(env_json: &str, headers_json: &str, url_json: &str) -> String {
    format!(r#"
        const __env_array = {};
        const __headers_array = {};
        var __url = {};

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
                    var desc = null;
                    if (__env_vars[k] && __env_vars[k].description) {{
                        desc = __env_vars[k].description;
                    }}
                    __env_vars[k] = {{ key: k, value: String(v), enabled: true, description: desc }};
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
                url: __url,
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
                var to = {{
                    equal: function(expected) {{
                        if (val !== expected) throw new Error("Expected " + expected + " but got " + val);
                    }},
                    eql: function(expected) {{
                        if (val !== expected && JSON.stringify(val) !== JSON.stringify(expected)) {{
                            throw new Error("Expected " + JSON.stringify(expected) + " but got " + JSON.stringify(val));
                        }}
                    }},
                    exist: function() {{
                        if (val === undefined || val === null) throw new Error("Expected value to exist but got " + val);
                    }},
                    ok: function() {{
                        if (!val) throw new Error("Expected " + val + " to be truthy");
                    }},
                    contain: function(item) {{
                        if (typeof val === 'string' || Array.isArray(val)) {{
                            if (val.indexOf(item) === -1) throw new Error("Expected " + JSON.stringify(val) + " to contain " + JSON.stringify(item));
                        }} else if (val && typeof val === 'object') {{
                            if (!(item in val)) throw new Error("Expected object to contain key " + item);
                        }}
                    }},
                    include: function(item) {{
                        if (typeof val === 'string' || Array.isArray(val)) {{
                            if (val.indexOf(item) === -1) throw new Error("Expected " + JSON.stringify(val) + " to include " + JSON.stringify(item));
                        }} else if (val && typeof val === 'object') {{
                            if (!(item in val)) throw new Error("Expected object to include key " + item);
                        }}
                    }},
                    property: function(prop) {{
                        if (!val || typeof val !== 'object' || !(prop in val)) {{
                            throw new Error("Expected object to have property '" + prop + "'");
                        }}
                    }},
                    a: function(type) {{
                        if (typeof val !== type) throw new Error("Expected type " + type + " but got " + typeof val);
                    }},
                    an: function(type) {{
                        if (typeof val !== type) throw new Error("Expected type " + type + " but got " + typeof val);
                    }}
                }};
                to.have = to;
                var beFn = function(expected) {{
                    if (arguments.length === 0) return to;
                    if (val !== expected) throw new Error("Expected " + expected + " but got " + val);
                }};
                beFn.equal = to.equal;
                beFn.eql = to.eql;
                beFn.ok = function() {{ if (!val) throw new Error("Expected " + val + " to be truthy"); }};
                beFn.true = function() {{ if (val !== true) throw new Error("Expected " + val + " to be true"); }};
                beFn.false = function() {{ if (val !== false) throw new Error("Expected " + val + " to be false"); }};
                beFn.a = to.a;
                beFn.an = to.an;
                to.be = beFn;

                return {{ to: to }};
            }}
        }};
        
        const pm = client;
        
        const timestamp = function() {{ return Date.now(); }};
        const uuid = function() {{
            return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {{
                var r = Math.random() * 16 | 0, v = c == 'x' ? r : (r & 0x3 | 0x8);
                return v.toString(16);
            }});
        }};
    "#, env_json, headers_json, url_json)
}
