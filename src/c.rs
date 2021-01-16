#![allow(non_camel_case_types)]

pub use libc::c_char;
pub use libc::c_int;
pub use libc::c_long;
pub use libc::c_uchar;
pub use libc::c_ulong;
pub use libc::size_t;

pub static VERSION_MAJOR: c_int = 0;
pub static VERSION_MINOR: c_int = 17;
/// # Safety
#[allow(non_snake_case)]
pub unsafe fn CHECK_VERSION() {
    termkey_check_version(VERSION_MAJOR, VERSION_MINOR);
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum Sym {
    UNKNOWN = -1,
    NONE = 0,

    /* Special names in C0 */
    BACKSPACE,
    TAB,
    ENTER,
    ESCAPE,

    /* Special names in G0 */
    SPACE,
    DEL,

    /* Special keys */
    UP,
    DOWN,
    LEFT,
    RIGHT,
    BEGIN,
    FIND,
    INSERT,
    DELETE,
    SELECT,
    PAGEUP,
    PAGEDOWN,
    HOME,
    END,

    /* Special keys from terminfo */
    CANCEL,
    CLEAR,
    CLOSE,
    COMMAND,
    COPY,
    EXIT,
    HELP,
    MARK,
    MESSAGE,
    MOVE,
    OPEN,
    OPTIONS,
    PRINT,
    REDO,
    REFERENCE,
    REFRESH,
    REPLACE,
    RESTART,
    RESUME,
    SAVE,
    SUSPEND,
    UNDO,

    /* Numeric keypad special keys */
    KP0,
    KP1,
    KP2,
    KP3,
    KP4,
    KP5,
    KP6,
    KP7,
    KP8,
    KP9,
    KPENTER,
    KPPLUS,
    KPMINUS,
    KPMULT,
    KPDIV,
    KPCOMMA,
    KPPERIOD,
    KPEQUALS,

    /* et cetera ad nauseum */
    N_SYMS,
}

impl ::std::fmt::Display for Sym {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let symi: c_long = unsafe { ::std::mem::transmute(self) };
        let _ = write!(fmt, "{}", symi);
        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum Type {
    UNICODE,
    FUNCTION,
    KEYSYM,
    MOUSE,
    POSITION,
    MODEREPORT,
    /* add other recognised types here */
    UNKNOWN_CSI = -1,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum Result {
    NONE,
    KEY,
    EOF,
    AGAIN,
    ERROR,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum MouseEvent {
    UNKNOWN,
    PRESS,
    DRAG,
    RELEASE,
}

impl ::std::fmt::Display for MouseEvent {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let symi: c_long = unsafe { ::std::mem::transmute(self) };
        let _ = write!(fmt, "{}", symi);
        Ok(())
    }
}

bitflags! { pub struct KeyMod: ::libc::c_int
{
  const SHIFT = 1 << 0;
  const ALT   = 1 << 1;
  const CTRL  = 1 << 2;
}}

impl ::std::fmt::Display for KeyMod {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let symi: c_int = unsafe { ::std::mem::transmute(*self) };
        let _ = write!(fmt, "{}", symi);
        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Key {
    pub type_: Type,
    pub code: c_long,
    /*
    union {
      long       codepoint; /* UNICODE */
      isize      number;    /* FUNCTION */
      Sym sym;       /* KEYSYM */
      char       mouse[4];  /* MOUSE */
                            /* opaque. see termkey_interpret_mouse */
    } code;
    */
    pub modifiers: c_int,

    /* Any Unicode character can be UTF-8 encoded in no more than 6 bytes, plus
     * terminating NUL */
    pub utf8: [c_char; 7],
}
impl ::std::default::Default for Key {
    fn default() -> Key {
        Key {
            type_: Type::UNICODE,
            code: 0,
            modifiers: 0,
            utf8: [0; 7],
        }
    }
}
impl Key {
    /// # Safety
    pub unsafe fn codepoint(&self) -> c_long {
        self.code
    }
    /// # Safety
    pub unsafe fn num(&self) -> c_int {
        let s: &c_int = &*(&self.code as *const i64 as *const i32);
        *s
    }
    /// # Safety
    pub unsafe fn sym(&self) -> Sym {
        let s: &Sym = &*(&self.code as *const i64 as *const Sym);
        *s
    }
}
impl Key {
    pub fn from_codepoint(mods: KeyMod, codepoint: char, utf8: [c_char; 7]) -> Key {
        unsafe {
            let mods: c_int = ::std::mem::transmute(mods);
            let codepoint: c_long = codepoint as c_long;
            Key {
                type_: Type::UNICODE,
                code: codepoint,
                modifiers: mods,
                utf8,
            }
        }
    }
    pub fn from_num(mods: KeyMod, num: isize) -> Key {
        unsafe {
            let mods: c_int = ::std::mem::transmute(mods);
            let num: c_int = num as c_int;
            let mut key = Key {
                type_: Type::FUNCTION,
                code: 0,
                modifiers: mods,
                utf8: [0; 7],
            };
            let code: &mut c_int = &mut *(&mut key.code as *mut i64 as *mut i32);
            *code = num;
            key
        }
    }
    pub fn from_sym(mods: KeyMod, sym: Sym) -> Key {
        unsafe {
            let mods: c_int = ::std::mem::transmute(mods);
            let mut key = Key {
                type_: Type::KEYSYM,
                code: 0,
                modifiers: mods,
                utf8: [0; 7],
            };
            let code: &mut Sym = &mut *(&mut key.code as *mut i64 as *mut Sym);
            *code = sym;
            key
        }
    }
    pub fn from_mouse(mods: KeyMod, ev: MouseEvent, button: c_int, line: c_int, col: c_int) -> Key {
        unsafe {
            let mods: c_int = ::std::mem::transmute(mods);
            let mut key = Key {
                type_: Type::MOUSE,
                code: 0,
                modifiers: mods,
                utf8: [0; 7],
            };
            key.construct_mouse_code(ev, button, line, col);
            key
        }
    }
    pub fn from_position(line: c_int, col: c_int) -> Key {
        unsafe {
            let mut key = Key {
                type_: Type::POSITION,
                code: 0,
                modifiers: 0,
                utf8: [0; 7],
            };
            key.set_linecol(line, col);
            key
        }
    }
    pub fn from_mode_report(initial: c_int, mode: c_int, value: c_int) -> Key {
        unsafe {
            let mut key = Key {
                type_: Type::MODEREPORT,
                code: 0,
                modifiers: 0,
                utf8: [0; 7],
            };
            key.construct_mode_report_code(initial, mode, value);
            key
        }
    }
    unsafe fn construct_mouse_code(
        &mut self,
        ev: MouseEvent,
        button: c_int,
        line: c_int,
        col: c_int,
    ) {
        self.set_linecol(line, col);
        let fields: &mut [c_char; 4] = &mut *(&mut self.code as *mut i64 as *mut [i8; 4]);
        fields[0] = match button {
            1 | 2 | 3 => button - 1,
            4 | 5 => button - 4 + 64,
            _ => 66,
        } as c_char;
        if ev == MouseEvent::DRAG {
            fields[0] |= 0x20;
        }
        if ev == MouseEvent::RELEASE {
            fields[3] = (fields[3] as c_uchar | 0x80) as c_char;
        }
    }
    unsafe fn construct_mode_report_code(&mut self, initial: c_int, mode: c_int, value: c_int) {
        let fields: &mut [c_char; 4] = &mut *(&mut self.code as *mut i64 as *mut [i8; 4]);
        fields[0] = initial as c_char;
        fields[1] = (mode >> 8) as c_char;
        fields[2] = (mode & 0xff) as c_char;
        fields[3] = value as c_char;
    }
    unsafe fn set_linecol(&mut self, mut line: c_int, mut col: c_int) {
        if col > 0xfff {
            col = 0xfff;
        }
        if line > 0x7ff {
            line = 0x7ff;
        }

        let fields: &mut [c_char; 4] = &mut *(&mut self.code as *mut i64 as *mut [i8; 4]);
        fields[1] = (col & 0x0ff) as c_char;
        fields[2] = (line & 0x0ff) as c_char;
        fields[3] = ((col & 0xf00) >> 8 | (line & 0x300) >> 4) as c_char;
    }
}

#[derive(Clone, Copy)]
pub enum TermKey {}

bitflags! { pub struct Flag : ::libc::c_int
{
  const NOINTERPRET = 1 << 0; /* Do not interpret C0//DEL codes if possible */
  const CONVERTKP   = 1 << 1; /* Convert KP codes to regular keypresses */
  const RAW         = 1 << 2; /* Input is raw bytes, not UTF-8 */
  const UTF8        = 1 << 3; /* Input is definitely UTF-8 */
  const NOTERMIOS   = 1 << 4; /* Do not make initial termios calls on construction */
  const SPACESYMBOL = 1 << 5; /* Sets SPACESYMBOL */
  const CTRLC       = 1 << 6; /* Allow Ctrl-C to be read as normal, disabling SIGINT */
  const EINTR       = 1 << 7; /* Return ERROR on signal (EINTR) rather than retry */
}}

bitflags! { pub struct Canon : ::libc::c_int
{
  const SPACESYMBOL = 1 << 0; /* Space is symbolic rather than Unicode */
  const DELBS       = 1 << 1; /* Del is converted to Backspace */
}}

bitflags! { #[repr(C)] pub struct Format : ::libc::c_int
{
  const LONGMOD     = 1 << 0; /* Shift-... instead of S-... */
  const CARETCTRL   = 1 << 1; /* ^X instead of C-X */
  const ALTISMETA   = 1 << 2; /* Meta- or M- instead of Alt- or A- */
  const WRAPBRACKET = 1 << 3; /* Wrap special keys in brackets like <Escape> */
  const SPACEMOD    = 1 << 4; /* M Foo instead of M-Foo */
  const LOWERMOD    = 1 << 5; /* meta or m instead of Meta or M */
  const LOWERSPACE  = 1 << 6; /* page down instead of PageDown */

  const MOUSE_POS   = 1 << 8; /* Include mouse position if relevant; @ col,line */

/* Some useful combinations */
  const VIM         = (Format::ALTISMETA.bits|Format::WRAPBRACKET.bits);
  const URWID       = (Format::LONGMOD.bits|Format::ALTISMETA.bits|
          Format::LOWERMOD.bits|Format::SPACEMOD.bits|Format::LOWERSPACE.bits);
}}

// Better to handle in makefile
//#[link(name = "termkey")]
extern "C" {
    pub fn termkey_check_version(major: c_int, minor: c_int);
    pub fn termkey_new(fd: c_int, flags: c_int) -> *mut TermKey;
    pub fn termkey_new_abstract(term: *const c_char, flags: c_int) -> *mut TermKey;
    pub fn termkey_free(tk: *mut TermKey);
    pub fn termkey_destroy(tk: *mut TermKey);

    pub fn termkey_start(tk: *mut TermKey) -> c_int;
    pub fn termkey_stop(tk: *mut TermKey) -> c_int;
    pub fn termkey_is_started(tk: *mut TermKey) -> c_int;

    pub fn termkey_get_fd(tk: *mut TermKey) -> c_int;

    pub fn termkey_get_flags(tk: *mut TermKey) -> c_int;
    pub fn termkey_set_flags(tk: *mut TermKey, newflags: c_int);

    pub fn termkey_get_waittime(tk: *mut TermKey) -> c_int;
    pub fn termkey_set_waittime(tk: *mut TermKey, msec: c_int);

    pub fn termkey_get_canonflags(tk: *mut TermKey) -> c_int;
    pub fn termkey_set_canonflags(tk: *mut TermKey, cflags: c_int);

    pub fn termkey_get_buffer_size(tk: *mut TermKey) -> size_t;
    pub fn termkey_set_buffer_size(tk: *mut TermKey, size: size_t) -> c_int;

    pub fn termkey_get_buffer_remaining(tk: *mut TermKey) -> size_t;

    pub fn termkey_canonicalise(tk: *mut TermKey, key: *mut Key);

    pub fn termkey_getkey(tk: *mut TermKey, key: *mut Key) -> Result;
    pub fn termkey_getkey_force(tk: *mut TermKey, key: *mut Key) -> Result;
    pub fn termkey_waitkey(tk: *mut TermKey, key: *mut Key) -> Result;

    pub fn termkey_advisereadable(tk: *mut TermKey) -> Result;

    pub fn termkey_push_bytes(tk: *mut TermKey, bytes: *const c_char, len: size_t) -> size_t;

    pub fn termkey_register_keyname(tk: *mut TermKey, sym: Sym, name: *const c_char) -> Sym;
    pub fn termkey_get_keyname(tk: *mut TermKey, sym: Sym) -> *const c_char;
    pub fn termkey_lookup_keyname(
        tk: *mut TermKey,
        str: *const c_char,
        sym: *mut Sym,
    ) -> *const c_char;

    pub fn termkey_keyname2sym(tk: *mut TermKey, keyname: *const c_char) -> Sym;

    pub fn termkey_interpret_mouse(
        tk: *mut TermKey,
        key: *const Key,
        event: *mut MouseEvent,
        button: *mut c_int,
        line: *mut c_int,
        col: *mut c_int,
    ) -> Result;

    pub fn termkey_interpret_position(
        tk: *mut TermKey,
        key: *const Key,
        line: *mut c_int,
        col: *mut c_int,
    ) -> Result;

    pub fn termkey_interpret_modereport(
        tk: *mut TermKey,
        key: *const Key,
        initial: *mut c_int,
        mode: *mut c_int,
        value: *mut c_int,
    ) -> Result;

    pub fn termkey_interpret_csi(
        tk: *mut TermKey,
        key: *const Key,
        args: *mut c_long,
        nargs: *mut size_t,
        cmd: *mut c_ulong,
    ) -> Result;

    pub fn termkey_strfkey(
        tk: *mut TermKey,
        buffer: *mut c_char,
        len: size_t,
        key: *mut Key,
        format: Format,
    ) -> size_t;
    pub fn termkey_strpkey(
        tk: *mut TermKey,
        str: *const c_char,
        key: *mut Key,
        format: Format,
    ) -> *const c_char;

    pub fn termkey_keycmp(tk: *mut TermKey, key1: *const Key, key2: *const Key) -> c_int;
}
