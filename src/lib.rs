#[macro_use]
extern crate bitflags;

extern crate libc;
pub mod c;

pub struct TermKey {
    tk: *mut c::TermKey,
}

impl TermKey {
    pub fn new(fd: c::c_int, flags: c::Flag) -> TermKey {
        unsafe {
            c::CHECK_VERSION();
            let tk = c::termkey_new(fd, std::mem::transmute(flags));
            if tk as usize == 0 {
                panic!()
            }
            TermKey { tk }
        }
    }
    pub fn new_abstract(term: &str, flags: c::Flag) -> TermKey {
        unsafe {
            c::CHECK_VERSION();
            ::std::ffi::CString::new(term.as_bytes())
                .map(|c_buffer| {
                    let tk = c::termkey_new_abstract(c_buffer.as_ptr(), std::mem::transmute(flags));
                    if tk as usize == 0 {
                        panic!()
                    }
                    TermKey { tk }
                })
                .unwrap()
        }
    }
}

impl Drop for TermKey {
    fn drop(&mut self) {
        unsafe { c::termkey_destroy(self.tk) }
    }
}

impl TermKey {
    pub fn start(&mut self) //-> Result<(), ()>
    {
        unsafe {
            if c::termkey_start(self.tk) == 0 {
                panic!()
            }
        }
    }
    pub fn stop(&mut self) //-> Result<(), ()>
    {
        unsafe {
            if c::termkey_stop(self.tk) == 0 {
                panic!()
            }
        }
    }
    pub fn is_started(&mut self) -> bool {
        unsafe { c::termkey_is_started(self.tk) != 0 }
    }
}

impl TermKey {
    pub fn get_fd(&mut self) -> isize {
        unsafe { c::termkey_get_fd(self.tk) as isize }
    }

    pub fn get_flags(&mut self) -> c::Flag {
        unsafe { std::mem::transmute(c::termkey_get_flags(self.tk)) }
    }
    pub fn set_flags(&mut self, newflags: c::Flag) {
        unsafe { c::termkey_set_flags(self.tk, std::mem::transmute(newflags)) }
    }

    pub fn get_waittime(&mut self) -> isize {
        unsafe { c::termkey_get_waittime(self.tk) as isize }
    }
    pub fn set_waittime(&mut self, msec: isize) {
        unsafe { c::termkey_set_waittime(self.tk, msec as c::c_int) }
    }

    pub fn get_canonflags(&mut self) -> c::Canon {
        unsafe { std::mem::transmute(c::termkey_get_canonflags(self.tk)) }
    }
    pub fn set_canonflags(&mut self, cflags: c::Canon) {
        unsafe { c::termkey_set_canonflags(self.tk, std::mem::transmute(cflags)) }
    }

    pub fn get_buffer_size(&mut self) -> usize {
        unsafe { c::termkey_get_buffer_size(self.tk) as usize }
    }
    pub fn set_buffer_size(&mut self, size: usize) -> isize {
        unsafe { c::termkey_set_buffer_size(self.tk, size as c::size_t) as isize }
    }

    pub fn get_buffer_remaining(&mut self) -> usize {
        unsafe { c::termkey_get_buffer_remaining(self.tk) as usize }
    }
}

#[derive(Clone, Copy)]
pub struct Utf8Char {
    pub bytes: [c::c_char; 7],
}
impl PartialEq for Utf8Char {
    fn eq(&self, other: &Utf8Char) -> bool {
        self.bytes == other.bytes
    }
}
impl PartialOrd for Utf8Char {
    fn partial_cmp(&self, other: &Utf8Char) -> Option<::std::cmp::Ordering> {
        self.bytes.partial_cmp(&other.bytes)
    }
}

impl Utf8Char {
    pub fn s(&self) -> &str {
        unsafe {
            let bytes: &[c::c_char] = &self.bytes;
            let bytes: &[u8] = &*(bytes as *const [i8] as *const [u8]);
            let string = ::std::str::from_utf8_unchecked(bytes);
            let (_, first_char) = string.char_indices().next().unwrap();
            string.get_unchecked(0..first_char.len_utf8())
        }
    }
}

// called Key in C
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub enum Event {
    UnknownCsi,

    Unicode {
        codepoint: char,
        mods: c::KeyMod,
        utf8: Utf8Char,
    },
    Function {
        num: isize,
        mods: c::KeyMod,
    },
    KeySym {
        sym: c::Sym,
        mods: c::KeyMod,
    },
    Mouse {
        ev: c::MouseEvent,
        mods: c::KeyMod,
        button: isize,
        line: isize,
        col: isize,
    },
    Position {
        line: isize,
        col: isize,
    },
    ModeReport {
        initial: isize,
        mode: isize,
        value: isize,
    },
}

impl Event {
    /// # Safety
    pub unsafe fn from_c(tk: *mut c::TermKey, key: c::Key) -> Event {
        match key.type_ {
            c::Type::UNICODE => Event::Unicode {
                mods: std::mem::transmute(key.modifiers),
                codepoint: std::char::from_u32(key.codepoint() as u32).unwrap(),
                utf8: Utf8Char { bytes: key.utf8 },
            },
            c::Type::FUNCTION => Event::Function {
                mods: std::mem::transmute(key.modifiers),
                num: key.num() as isize,
            },
            c::Type::KEYSYM => Event::KeySym {
                mods: std::mem::transmute(key.modifiers),
                sym: key.sym(),
            },
            c::Type::MOUSE => {
                let mut ev: c::MouseEvent = c::MouseEvent::UNKNOWN;
                let mut button: c::c_int = 0;
                let mut line: c::c_int = 0;
                let mut col: c::c_int = 0;
                if c::termkey_interpret_mouse(tk, &key, &mut ev, &mut button, &mut line, &mut col)
                    != c::Result::KEY
                {
                    panic!()
                }
                Event::Mouse {
                    mods: std::mem::transmute(key.modifiers),
                    ev,
                    button: button as isize,
                    line: line as isize,
                    col: col as isize,
                }
            }
            c::Type::POSITION => {
                let mut line: c::c_int = 0;
                let mut col: c::c_int = 0;
                if c::termkey_interpret_position(tk, &key, &mut line, &mut col) != c::Result::KEY {
                    panic!()
                }
                Event::Position {
                    line: line as isize,
                    col: col as isize,
                }
            }
            c::Type::MODEREPORT => {
                let mut initial: c::c_int = 0;
                let mut mode: c::c_int = 0;
                let mut value: c::c_int = 0;
                if c::termkey_interpret_modereport(tk, &key, &mut initial, &mut mode, &mut value)
                    != c::Result::KEY
                {
                    panic!()
                }
                Event::ModeReport {
                    initial: initial as isize,
                    mode: mode as isize,
                    value: value as isize,
                }
            }
            c::Type::UNKNOWN_CSI => {
                // termkey 0.17 hard-codes size as 16. Oops!
                // once termkey is fixed we should change this to a loop

                // Removed, I have decided not to expose this API
                Event::UnknownCsi
            }
        }
    }
}

pub enum Result {
    None_,
    Key(Event),
    Eof,
    Again,
    Error { err: ::std::io::Error },
}
impl Result {
    /// # Safety
    pub unsafe fn from_c(tk: *mut c::TermKey, key: c::Key, res: c::Result) -> Result {
        match res {
            c::Result::NONE => Result::None_,
            c::Result::KEY => Result::Key(Event::from_c(tk, key)),
            c::Result::EOF => Result::Eof,
            c::Result::AGAIN => Result::Again,
            c::Result::ERROR => Result::Error {
                err: ::std::io::Error::last_os_error(),
            },
        }
    }
}

impl TermKey {
    pub fn getkey(&mut self) -> Result {
        let mut key: c::Key = std::default::Default::default();
        let res = unsafe { c::termkey_getkey(self.tk, &mut key) };
        unsafe { Result::from_c(self.tk, key, res) }
    }
    pub fn getkey_force(&mut self) -> Result {
        let mut key: c::Key = std::default::Default::default();
        let res = unsafe { c::termkey_getkey_force(self.tk, &mut key) };
        unsafe { Result::from_c(self.tk, key, res) }
    }
    pub fn waitkey(&mut self) -> Result {
        let mut key: c::Key = std::default::Default::default();
        let res = unsafe { c::termkey_waitkey(self.tk, &mut key) };
        unsafe { Result::from_c(self.tk, key, res) }
    }
    // will never return Key
    pub fn advisereadable(&mut self) -> Result {
        let res = unsafe { c::termkey_advisereadable(self.tk) };
        unsafe { Result::from_c(self.tk, std::default::Default::default(), res) }
    }
    pub fn push_bytes(&mut self, bytes: &[u8]) -> usize {
        unsafe {
            c::termkey_push_bytes(
                self.tk,
                std::mem::transmute(&bytes[0]),
                bytes.len() as c::size_t,
            ) as usize
        }
    }
}

impl TermKey {
    // Unsupported because it requires static strings (C literals)
    // Also would require rethinking the enum nature.
    // pub fn register_keyname(&mut self, sym: c::Sym, name: &str) -> c::Sym { }

    //pub fn get_keyname(&mut self, sym: c::Sym) -> &'static str
    //{
    //unsafe
    //{
    //std::str::from_c_str(c::termkey_get_keyname(self.tk, sym))
    //}
    //}

    pub fn lookup_keyname<'a>(&mut self, s: &'a str, sym: &mut c::Sym) -> Option<&'a str> {
        unsafe {
            ::std::ffi::CString::new(s.as_bytes())
                .ok()
                .and_then(|cbuf| {
                    let rbuf = c::termkey_lookup_keyname(self.tk, cbuf.as_ptr(), sym);
                    let ci = cbuf.as_ptr() as usize;
                    let ri = rbuf as usize;
                    if ri != 0 {
                        let off = ri - ci;
                        let sbytelen = s.as_bytes().len();
                        Some(s.get_unchecked(off..sbytelen))
                    } else {
                        None
                    }
                })
        }
    }

    pub fn keyname2sym(&mut self, keyname: &str) -> c::Sym {
        unsafe {
            ::std::ffi::CString::new(keyname.as_bytes())
                .map(|name| c::termkey_keyname2sym(self.tk, name.as_ptr()))
                .unwrap()
        }
    }
}

impl TermKey {
    pub fn strfkey(&mut self, key: Event, format: c::Format) -> String {
        let mut buf: [c::c_char; 52] = [0; 52];
        let mut key_ = match key {
            Event::Unicode {
                mods,
                codepoint,
                utf8,
            } => c::Key::from_codepoint(mods, codepoint, utf8.bytes),
            Event::Function { mods, num } => c::Key::from_num(mods, num),
            Event::KeySym { mods, sym } => c::Key::from_sym(mods, sym),
            Event::Mouse {
                ev,
                mods,
                button,
                line,
                col,
            } => c::Key::from_mouse(
                mods,
                ev,
                button as c::c_int,
                line as c::c_int,
                col as c::c_int,
            ),
            Event::Position { line, col } => {
                c::Key::from_position(line as c::c_int, col as c::c_int)
            }
            Event::ModeReport {
                initial,
                mode,
                value,
            } => c::Key::from_mode_report(initial as c::c_int, mode as c::c_int, value as c::c_int),
            Event::UnknownCsi => {
                // TODO implement
                return "unknown csi (stringification not implemented)".to_string();
            }
        };
        unsafe {
            let sz = c::termkey_strfkey(self.tk, &mut buf[0], 52, &mut key_, format) as usize;
            assert!(sz < 52, "key name should not be that long!");
            std::str::from_utf8_unchecked(&*(&buf[0..sz] as *const [i8] as *const [u8])).to_string()
        }
    }

    pub fn strpkey<'a>(&mut self, s: &'a str, format: c::Format) -> Option<(Event, &'a str)> {
        unsafe {
            ::std::ffi::CString::new(s.as_bytes())
                .ok()
                .and_then(|cbuf| {
                    let mut ckey: c::Key = std::default::Default::default();
                    let rbuf = c::termkey_strpkey(self.tk, cbuf.as_ptr(), &mut ckey, format);
                    let ci = cbuf.as_ptr() as usize;
                    let ri = rbuf as usize;
                    if ri != 0 {
                        let key = Event::from_c(self.tk, ckey);
                        let off = ri - ci;
                        let sbytelen = s.as_bytes().len();
                        Some((key, s.get_unchecked(off..sbytelen)))
                    } else {
                        None
                    }
                })
        }
    }
}
