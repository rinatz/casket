use std::os::raw::{c_char, c_void};

pub enum KCDB {}

pub const KCESUCCESS: i32 = 0;
pub const KCENOIMPL: i32 = 1;
pub const KCEINVALID: i32 = 2;
pub const KCENOREPOS: i32 = 3;
pub const KCENOPERM: i32 = 4;
pub const KCEBROKEN: i32 = 5;
pub const KCEDUPREC: i32 = 6;
pub const KCENOREC: i32 = 7;
pub const KCELOGIC: i32 = 8;
pub const KCESYSTEM: i32 = 9;
pub const KCEMISC: i32 = 15;

pub const KCOREADER: u32 = 1 << 0;
pub const KCOWRITER: u32 = 1 << 1;
pub const KCOCREATE: u32 = 1 << 2;
pub const KCOTRUNCATE: u32 = 1 << 3;
pub const KCOAUTOTRAN: u32 = 1 << 4;
pub const KCOAUTOSYNC: u32 = 1 << 5;
pub const KCONOLOCK: u32 = 1 << 6;
pub const KCOTRYLOCK: u32 = 1 << 7;
pub const KCONOREPAIR: u32 = 1 << 8;

#[link(name = "kyotocabinet")]
extern "C" {
    pub fn kcfree(ptr: *mut c_void);

    pub fn kcdbnew() -> *mut KCDB;
    pub fn kcdbdel(db: *mut KCDB);

    pub fn kcdbopen(db: *mut KCDB, path: *const c_char, mode: u32) -> i32;
    pub fn kcdbclose(db: *mut KCDB) -> i32;

    pub fn kcdbemsg(db: *mut KCDB) -> *const c_char;
    pub fn kcdbecode(db: *mut KCDB) -> i32;

    pub fn kcdbset(
        db: *mut KCDB,
        kbuf: *const c_char,
        ksiz: usize,
        vbuf: *const c_char,
        vsiz: usize,
    ) -> i32;
    pub fn kcdbget(db: *mut KCDB, kbuf: *const c_char, ksiz: usize, sp: *mut usize) -> *mut c_char;
}
