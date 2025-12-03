use std::{error::Error, fmt::Display, fs::File, io::Read};

use httparse::{EMPTY_HEADER, Request};
use mlua::{Lua, LuaOptions, StdLib};

const LUA_STARTBLOCK_SIZE: usize = "<lua>".len();
const LUA_ENDBLOCK_SIZE: usize = "</lua>".len();

struct LuaBlockMarker<'a>
{
    start_position: usize,
    end_position: usize,
    data: &'a str
}

#[derive(Debug)]
enum CodeBlockParseError
{
    MissingBlockStart,
    MissingBlockEnd
}

impl Error for CodeBlockParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }

}

impl Display for CodeBlockParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeBlockParseError::MissingBlockStart => f.write_str("Missing code block start"),
            CodeBlockParseError::MissingBlockEnd => f.write_str("Missing code block end"),
        }
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut args = std::env::args();

    let _exec_path = args.next().unwrap();

    let target_file_path = args.next().expect("Missing target file!");
    println!("{:?}", target_file_path);

    let mut file_handler = File::open(&target_file_path)?;

    let request_string = args.next();

    let mut file_as_string: String = String::new();

    file_handler.read_to_string(&mut file_as_string)?;


    let code_blocks = get_codeblock_positions(&file_as_string)?;
    

    let lua_interpeter = Lua::new_with(StdLib::NONE, LuaOptions::new())?;

    if request_string.is_some()
    {
        let mut header_slice = [EMPTY_HEADER; 1000];
        let mut request = Request::new(&mut header_slice);
        let requst_str = request_string.unwrap();

        request.parse(requst_str.as_bytes())?;

        let global_table = lua_interpeter.globals();
    
        global_table.set("PATH", request.path.expect("Expected Valid Path"))?;
        global_table.set("METHOD", request.method.expect("Expected Valid Method"))?;
        global_table.set("VERSION", request.version.expect("Expected valid version"))?;

        let header_table = lua_interpeter.create_table().unwrap();

        for header in request.headers
        {
            header_table.set(header.name, String::from_utf8_lossy(header.value))?;
        }
        global_table.set("HEADERS", header_table).unwrap();
    }


    let mut final_string_buffer = String::new();
    let mut last_block_end_pos = 0;

    for code_block in code_blocks
    {
        let preceding = &file_as_string[last_block_end_pos .. code_block.start_position];
        last_block_end_pos = code_block.end_position + LUA_ENDBLOCK_SIZE;

        final_string_buffer.push_str(preceding);

        let lua_chunk = lua_interpeter.load(code_block.data);
        let output_string: String = lua_chunk.eval()?;
        final_string_buffer.push_str(&output_string);
    }

    final_string_buffer.push_str(&file_as_string[last_block_end_pos .. file_as_string.len()]);

    print!("{}", final_string_buffer);
    
    Ok(())
}

fn get_codeblock_positions<'a> (input: &'a str) -> Result<Vec<LuaBlockMarker<'a>>,  CodeBlockParseError>
{
    let mut positions: Vec<LuaBlockMarker> = Vec::new();

    let code_block_starts: Vec<(usize, &str)> = input.match_indices("<lua>").collect();
    let mut code_block_ends: Vec<(usize, &str)> = input.match_indices("</lua>").collect();
    code_block_ends.reverse();

    println!("{:?}", code_block_starts);
    println!("{:?}", code_block_ends);

    if code_block_starts.len() > code_block_ends.len()
    {
        return Err(CodeBlockParseError::MissingBlockEnd);
    }
    else if code_block_starts.len() < code_block_ends.len()
    {
        return Err(CodeBlockParseError::MissingBlockStart);
    }


    for (pos, _) in code_block_starts.iter() {
        let endpos = code_block_ends.pop().expect("Vectors should have equal size");

        positions.push(LuaBlockMarker {
            start_position: *pos,
            end_position: endpos.0,
            data: &input[*pos + LUA_STARTBLOCK_SIZE .. endpos.0],
        });
    }

    Ok(positions)
}
