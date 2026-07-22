pub fn get_console_shim() -> &'static str {
    r#"
        const __logs = [];
        const console = {
            log: function(...args) { __logs.push({ level: "log", message: args.join(" ") }); },
            warn: function(...args) { __logs.push({ level: "warn", message: args.join(" ") }); },
            error: function(...args) { __logs.push({ level: "error", message: args.join(" ") }); }
        };
    "#
}
