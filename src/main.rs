#![feature(slicing_syntax)]
#![feature(if_let)]

extern crate fmt;
extern crate mysql;
extern crate "rust-hl-lua" as lua;

use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};
use std::sync::TaskPool;
use fmt::{Fmt, FmtParser};
use lua::Lua;
use lua_cache::LuaCache;

mod lua_cache;
mod db;

struct Conn<'a> {
    stream: &'a mut TcpStream,
    luacache: LuaCache,
}

impl<'a> Conn<'a> {
    fn new(stream: &'a mut TcpStream) -> Conn {
        Conn {
            stream: stream,
            luacache: LuaCache::new(),
        }
    }
}

fn main() {
    let pool = TaskPool::new(4);
    let listener = TcpListener::bind("0.0.0.0:8001");
    let mut acceptor = listener.listen();
    db::have_a_try_to_mysql();
    for stream in acceptor.incoming() {
        match stream {
            Ok(mut stream) => {
                pool.execute(proc() {
                    on_conn(&mut stream);
                });
            }
            Err(e) => { println!("Error: {}", e); break; }
        }
    }
    drop(acceptor);
}

fn on_conn(stream: &mut TcpStream) {
    println!("on conn");
    let mut parser = FmtParser::new();
    let mut conn = Conn::new(stream);
    let mut buf = [0u8, ..128];
    loop {
        match conn.stream.read(buf.as_mut_slice()) {
            Ok(len) => {
                println!("on data: {} bytes", len);
                parser.push(buf[0..len], |cmd,fmt| {
                        on_cmd_fmt(&mut conn, cmd, fmt);
                    });
            },
            Err(e) => {
                println!("read err: {}\nclose conn...", e);
                match conn.stream.close_write() { _ => {} };
                break;
            }
        }
    }
}

fn on_cmd_fmt(conn: &mut Conn, cmd: i16, fmt: &Fmt) {
    println!("on cmd: {}, {}", cmd, fmt.get_type());
    let packet = fmt.packet(9);
    match conn.stream.write(packet.as_slice()) { _ => {} };
    let luavm = &mut Lua::new();
    luavm.set("liigo_sum", lua_sum);
    luavm.set("liigo_add", |x: int, y: int| x + y);
    luavm.set("name", "Liigo Zhuang");
    let path = Path::new("./test.lua");
    match conn.luacache.execute(&path, luavm) {
        Ok(r) => println!("lua returns: {}", r),
        Err(e) => println!("lua execute fails: {}", e),
    }
}

fn lua_sum(a: int, b: int) -> int {
    a + b
}
