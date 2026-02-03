use mlua::Lua;

pub struct ScriptEngine {
    lua: Lua,
}

impl ScriptEngine {
    pub fn new() -> Self {
        Self { lua: Lua::new() }
    }

    pub fn init(&self) -> mlua::Result<()> {
        let globals = self.lua.globals();

        // Example: Expose a simple log function
        globals.set(
            "log",
            self.lua.create_function(|_, msg: String| {
                // In a real editor, this would log to a message buffer
                println!("Lua: {}", msg);
                Ok(())
            })?,
        )?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn run_script(&self, script: &str) -> mlua::Result<()> {
        self.lua.load(script).exec()
    }
}
