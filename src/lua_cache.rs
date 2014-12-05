use std::io::IoResult;
use std::io::fs::File;
use std::io::fs::PathExtensions;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use lua::{Lua, LuaError};

struct LuaInfo {
    lua_code: String,
    last_modify: u64,
}

pub struct LuaCache {
    infos: HashMap<Path, LuaInfo>,
}

impl LuaCache {
    pub fn new() -> LuaCache {
        LuaCache {
            infos: HashMap::new(),
        }
    }

    pub fn execute(&mut self, path: &Path, lua_vm: &mut Lua)
        -> Result<String, LuaError>
    {
        match self.code(path) {
            Ok(code) => {
                let r: Result<String,LuaError> = lua_vm.execute(code.as_slice());
                r
            }
            Err(e) => Err(LuaError::ReadError(e)),
        }
    }

    pub fn code(&mut self, path: &Path) -> IoResult<String> {
        match self.infos.entry(path.clone()) {
            Occupied(mut entry) => {
                let mut need_reload = true;
                let info = entry.get_mut();
                match path.stat() {
                    Ok(stat) => {
                        if stat.modified == info.last_modify {
                            need_reload = false;
                        }
                    }
                    Err(e) => { return Err(e); }
                }
                if need_reload {
                    *info = try!(load_lua_info(path));
                }
                return Ok(info.lua_code.clone());
            }
            Vacant(entry) => {
                let info = try!(load_lua_info(path));
                let info = entry.set(info);
                return Ok(info.lua_code.clone());
            }
        }
    }
}

fn load_lua_info(path: &Path) -> IoResult<LuaInfo> {
    println!("load lua file: {}", path.display());
    // we read the last_modify first, then read the code.
    let last_modify = match path.stat() {
        Ok(stat) => stat.modified,
        Err(_) => 0,
    };
    let mut file = try!(File::open(path));
    let lua_code = try!(file.read_to_string());
    Ok(LuaInfo{ lua_code: lua_code, last_modify: last_modify })
}
