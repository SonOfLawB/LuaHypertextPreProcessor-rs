use std::{fs::File, io::Read, rc::Rc};

use mlua::{HookTriggers, Lua, LuaOptions, StdLib, Table};
struct LuaBlockMarker
{
    start_position: usize,
    end_position: usize,
}
fn main() {

    let mut file = File::open("/Users/lawsonborn/Documents/lhpp/src/test.lhpp").unwrap();

    let mut file_as_string: String = String::new();

    file.read_to_string(&mut file_as_string).unwrap();

    let code_block_start = file_as_string.find("<lua>").unwrap();
    let code_block_end = file_as_string.find("</lua>").unwrap();

    let lua_interpeter = Lua::new_with(StdLib::NONE, LuaOptions::new()).unwrap();

    let mut output_buffer = Rc::new(String::new());

    let out_func = lua_interpeter.create_function(|lua, output: String| {
        let existing_output: Result<String, mlua::Error> = lua.globals().get("FINALOUT");
        let mut val = match existing_output {
            Ok(str) => str,
            Err(_) => String::new(),
        };

        val.push_str(&output);

        lua.globals().set("FINALOUT",val).unwrap();
        return Result::Ok(());
    }).unwrap();


    let mut chunk = lua_interpeter.load(&file_as_string[code_block_start + 5 .. code_block_end]);
    chunk = chunk.set_environment(lua_interpeter.globals());

    chunk.environment().unwrap().set("out", out_func).unwrap();
    let return_value: String = chunk.eval().unwrap();
    
    file_as_string.replace_range(code_block_start..code_block_end+6, &return_value);
    println!("{}", file_as_string);

}
