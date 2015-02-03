#![feature(collections)]
#![feature(hash)]
#![feature(int_uint)]
#![feature(libc)]
#![feature(os)]
#![feature(std_misc)]

#[macro_use]
extern crate bitflags;

extern crate libc;
pub mod c;

pub struct TermKey
{
    tk: *mut c::TermKey,
}

impl TermKey
{
    pub fn new(fd: c::c_int, flags: c::X_TermKey_Flag) -> TermKey
    {
        unsafe
        {
            c::TERMKEY_CHECK_VERSION();
            let tk = c::termkey_new(fd, std::mem::transmute(flags));
            if tk as usize == 0
            {
                panic!()
            }
            TermKey{tk: tk}
        }
    }
    pub fn new_abstract(term: &str, flags: c::X_TermKey_Flag) -> TermKey
    {
        unsafe
        {
            c::TERMKEY_CHECK_VERSION();
            let c_buffer = ::std::ffi::CString::from_slice(term.as_bytes());
            let tk = c::termkey_new_abstract(c_buffer.as_ptr(), std::mem::transmute(flags));
            if tk as usize == 0
            {
                panic!()
            }
            TermKey{tk: tk}
        }
    }
}

impl Drop for TermKey
{
    fn drop(&mut self)
    {
        unsafe
        {
            c::termkey_destroy(self.tk)
        }
    }
}

impl TermKey
{
    pub fn start(&mut self) //-> Result<(), ()>
    {
        unsafe
        {
            if c::termkey_start(self.tk) == 0
            {
                panic!()
            }
        }
    }
    pub fn stop(&mut self) //-> Result<(), ()>
    {
        unsafe
        {
            if c::termkey_stop(self.tk) == 0
            {
                panic!()
            }
        }
    }
    pub fn is_started(&mut self) -> bool
    {
        unsafe
        {
            c::termkey_is_started(self.tk) != 0
        }
    }
}

impl TermKey
{
    pub fn get_fd(&mut self) -> isize
    {
        unsafe
        {
            c::termkey_get_fd(self.tk) as isize
        }
    }

    pub fn get_flags(&mut self) -> c::X_TermKey_Flag
    {
        unsafe
        {
            std::mem::transmute(c::termkey_get_flags(self.tk))
        }
    }
    pub fn set_flags(&mut self, newflags: c::X_TermKey_Flag)
    {
        unsafe
        {
            c::termkey_set_flags(self.tk, std::mem::transmute(newflags))
        }
    }

    pub fn get_waittime(&mut self) -> isize
    {
        unsafe
        {
            c::termkey_get_waittime(self.tk) as isize
        }
    }
    pub fn set_waittime(&mut self, msec: isize)
    {
        unsafe
        {
            c::termkey_set_waittime(self.tk, msec as c::c_int)
        }
    }

    pub fn get_canonflags(&mut self) -> c::X_TermKey_Canon
    {
        unsafe
        {
            std::mem::transmute(c::termkey_get_canonflags(self.tk))
        }
    }
    pub fn set_canonflags(&mut self, cflags: c::X_TermKey_Canon)
    {
        unsafe
        {
            c::termkey_set_canonflags(self.tk, std::mem::transmute(cflags))
        }
    }

    pub fn get_buffer_size(&mut self) -> usize
    {
        unsafe
        {
            c::termkey_get_buffer_size(self.tk) as usize
        }
    }
    pub fn set_buffer_size(&mut self, size: usize) -> isize
    {
        unsafe
        {
            c::termkey_set_buffer_size(self.tk, size as c::size_t) as isize
        }
    }

    pub fn get_buffer_remaining(&mut self) -> usize
    {
        unsafe
        {
            c::termkey_get_buffer_remaining(self.tk) as usize
        }
    }
}

#[derive(Copy)]
pub struct Utf8Char
{
    pub bytes: [c::c_char; 7],
}
impl PartialEq for Utf8Char
{
    fn eq(&self, other: &Utf8Char) -> bool
    {
        self.bytes == other.bytes
    }
}
impl PartialOrd for Utf8Char
{
    fn partial_cmp(&self, other: &Utf8Char) -> Option<::std::cmp::Ordering>
    {
        self.bytes.partial_cmp(&other.bytes)
    }
}

impl Utf8Char
{
    pub fn s<'a>(&'a self) -> &'a str
    {
        unsafe
        {
            let bytes: &[c::c_char] = &self.bytes;
            let bytes: &[u8] = ::std::mem::transmute(bytes);
            ::std::str::from_utf8_unchecked(bytes).slice_chars(0, 1)
        }
    }
}

// called TermKeyKey in C
#[derive(Copy, PartialEq, PartialOrd)]
pub enum TermKeyEvent
{
    UnknownCsiEvent,

    UnicodeEvent{codepoint: char, mods: c::X_TermKey_KeyMod, utf8: Utf8Char},
    FunctionEvent{num: isize, mods: c::X_TermKey_KeyMod},
    KeySymEvent{sym: c::TermKeySym, mods: c::X_TermKey_KeyMod},
    MouseEvent{ev: c::TermKeyMouseEvent, mods: c::X_TermKey_KeyMod, button: isize, line: isize, col: isize},
    PositionEvent{line: isize, col: isize},
    ModeReportEvent{initial: isize, mode: isize, value: isize},
}

impl TermKeyEvent
{
    pub fn from_c(tk: *mut c::TermKey, key: c::TermKeyKey) -> TermKeyEvent
    {
        match key.type_
        {
            c::TermKeyType::TERMKEY_TYPE_UNICODE =>
            {
                unsafe
                {
                    TermKeyEvent::UnicodeEvent{mods: std::mem::transmute(key.modifiers),
                            codepoint: std::char::from_u32(key.codepoint() as u32).unwrap(),
                            utf8: Utf8Char{bytes: key.utf8}}
                }
            }
            c::TermKeyType::TERMKEY_TYPE_FUNCTION =>
            {
                unsafe
                {
                    TermKeyEvent::FunctionEvent{mods: std::mem::transmute(key.modifiers),
                            num: key.num() as isize}
                }
            }
            c::TermKeyType::TERMKEY_TYPE_KEYSYM =>
            {
                unsafe
                {
                    TermKeyEvent::KeySymEvent{mods: std::mem::transmute(key.modifiers),
                            sym: key.sym()}
                }
            }
            c::TermKeyType::TERMKEY_TYPE_MOUSE =>
            {
                let mut ev: c::TermKeyMouseEvent = c::TermKeyMouseEvent::TERMKEY_MOUSE_UNKNOWN;
                let mut button: c::c_int = 0;
                let mut line: c::c_int = 0;
                let mut col: c::c_int = 0;
                unsafe
                {
                    if c::termkey_interpret_mouse(tk, &key,
                            &mut ev, &mut button, &mut line, &mut col) != c::TermKeyResult::TERMKEY_RES_KEY
                    {
                        panic!()
                    }
                    TermKeyEvent::MouseEvent{mods: std::mem::transmute(key.modifiers), ev: ev, button: button as isize,
                            line: line as isize, col: col as isize}
                }
            }
            c::TermKeyType::TERMKEY_TYPE_POSITION =>
            {
                let mut line: c::c_int = 0;
                let mut col: c::c_int = 0;
                unsafe
                {
                    if c::termkey_interpret_position(tk, &key,
                            &mut line, &mut col) != c::TermKeyResult::TERMKEY_RES_KEY
                    {
                        panic!()
                    }
                    TermKeyEvent::PositionEvent{line: line as isize, col: col as isize}
                }
            }
            c::TermKeyType::TERMKEY_TYPE_MODEREPORT =>
            {
                let mut initial: c::c_int = 0;
                let mut mode: c::c_int = 0;
                let mut value: c::c_int = 0;
                unsafe
                {
                    if c::termkey_interpret_modereport(tk, &key,
                            &mut initial, &mut mode, &mut value) != c::TermKeyResult::TERMKEY_RES_KEY
                    {
                        panic!()
                    }
                    TermKeyEvent::ModeReportEvent{initial: initial as isize, mode: mode as isize, value: value as isize}
                }
            }
            c::TermKeyType::TERMKEY_TYPE_UNKNOWN_CSI =>
            {
                // termkey 0.17 hard-codes size as 16. Oops!
                // once termkey is fixed we should change this to a loop

                // Removed, I have decided not to expose this API
                TermKeyEvent::UnknownCsiEvent
            }
        }
    }
}

#[derive(Copy)]
pub enum TermKeyResult
{
    None_,
    Key(TermKeyEvent),
    Eof,
    Again,
    Error{errno: c::c_int},
}
impl TermKeyResult
{
    pub fn from_c(tk: *mut c::TermKey, key: c::TermKeyKey, res: c::TermKeyResult) -> TermKeyResult
    {
        match res
        {
            c::TermKeyResult::TERMKEY_RES_NONE => TermKeyResult::None_,
            c::TermKeyResult::TERMKEY_RES_KEY => TermKeyResult::Key(TermKeyEvent::from_c(tk, key)),
            c::TermKeyResult::TERMKEY_RES_EOF => TermKeyResult::Eof,
            c::TermKeyResult::TERMKEY_RES_AGAIN => TermKeyResult::Again,
            c::TermKeyResult::TERMKEY_RES_ERROR => TermKeyResult::Error{errno: std::os::errno() as c::c_int},
        }
    }
}

impl TermKey
{
    pub fn getkey(&mut self) -> TermKeyResult
    {
        let mut key: c::TermKeyKey = std::default::Default::default();
        let res = unsafe
        {
            c::termkey_getkey(self.tk, &mut key)
        };
        TermKeyResult::from_c(self.tk, key, res)
    }
    pub fn getkey_force(&mut self) -> TermKeyResult
    {
        let mut key: c::TermKeyKey = std::default::Default::default();
        let res = unsafe
        {
            c::termkey_getkey_force(self.tk, &mut key)
        };
        TermKeyResult::from_c(self.tk, key, res)
    }
    pub fn waitkey(&mut self) -> TermKeyResult
    {
        let mut key: c::TermKeyKey = std::default::Default::default();
        let res = unsafe
        {
            c::termkey_waitkey(self.tk, &mut key)
        };
        TermKeyResult::from_c(self.tk, key, res)
    }
    // will never return Key
    pub fn advisereadable(&mut self) -> TermKeyResult
    {
        let res = unsafe
        {
            c::termkey_advisereadable(self.tk)
        };
        TermKeyResult::from_c(self.tk, std::default::Default::default(), res)
    }
    pub fn push_bytes(&mut self, bytes: &[u8]) -> usize
    {
        unsafe
        {
            c::termkey_push_bytes(self.tk, std::mem::transmute(&bytes[0]), bytes.len() as c::size_t) as usize
        }
    }
}

impl TermKey
{
    // Unsupported because it requires static strings (C literals)
    // Also would require rethinking the enum nature.
    // pub fn register_keyname(&mut self, sym: c::TermKeySym, name: &str) -> c::TermKeySym { }

    //pub fn get_keyname(&mut self, sym: c::TermKeySym) -> &'static str
    //{
        //unsafe
        //{
            //std::str::from_c_str(c::termkey_get_keyname(self.tk, sym))
        //}
    //}

    pub fn lookup_keyname<'a>(&mut self, s: &'a str, sym: &mut c::TermKeySym) -> Option<&'a str>
    {
        unsafe
        {
            let cbuf = ::std::ffi::CString::from_slice(s.as_bytes());
            let rbuf = c::termkey_lookup_keyname(self.tk, cbuf.as_ptr(), sym);
            let ci = cbuf.as_ptr() as usize;
            let ri = rbuf as usize;
            if ri != 0
            {
                let off = ri - ci;
                let sbytelen = s.as_bytes().len();
                Some(s.slice_unchecked(off, sbytelen))
            }
            else
            {
                None
            }
        }
    }

    pub fn keyname2sym(&mut self, keyname: &str) -> c::TermKeySym
    {
        unsafe
        {
            let name = ::std::ffi::CString::from_slice(keyname.as_bytes());
            c::termkey_keyname2sym(self.tk, name.as_ptr())
        }
    }
}

impl TermKey
{
    pub fn strfkey(&mut self, key: TermKeyEvent, format: c::TermKeyFormat) -> String
    {
        let mut buf: [c::c_char; 52] = [0; 52];
        let mut key_ = match key
        {
            TermKeyEvent::UnicodeEvent{mods, codepoint, utf8} =>
            {
                c::TermKeyKey::from_codepoint(mods, codepoint, utf8.bytes)
            }
            TermKeyEvent::FunctionEvent{mods, num} =>
            {
                c::TermKeyKey::from_num(mods, num)
            }
            TermKeyEvent::KeySymEvent{mods, sym} =>
            {
                c::TermKeyKey::from_sym(mods, sym)
            }
            TermKeyEvent::MouseEvent{ev, mods, button, line, col} =>
            {
                c::TermKeyKey::from_mouse(self.tk, mods, ev, button as c::c_int, line as c::c_int, col as c::c_int)
            }
            TermKeyEvent::PositionEvent{line, col} =>
            {
                c::TermKeyKey::from_position(self.tk, line as c::c_int, col as c::c_int)
            }
            TermKeyEvent::ModeReportEvent{initial, mode, value} =>
            {
                c::TermKeyKey::from_mode_report(self.tk, initial as c::c_int, mode as c::c_int, value as c::c_int)
            }
            TermKeyEvent::UnknownCsiEvent =>
            {
                // TODO implement
                return "unknown csi (stringification not implemented)".to_string();
            }
        };
        unsafe
        {
            let sz = c::termkey_strfkey(self.tk, &mut buf[0], 52, &mut key_, format) as usize;
            assert!(sz < 52, "key name should not be that long!");
            std::str::from_utf8_unchecked(std::mem::transmute(&buf[0 .. sz])).to_string()
        }
    }

    pub fn strpkey<'a>(&mut self, s: &'a str, format: c::TermKeyFormat) -> Option<(TermKeyEvent, &'a str)>
    {
        unsafe
        {
            let cbuf = ::std::ffi::CString::from_slice(s.as_bytes());
            let mut ckey : c::TermKeyKey = std::default::Default::default();
            let rbuf = c::termkey_strpkey(self.tk, cbuf.as_ptr(), &mut ckey, format);
            let ci = cbuf.as_ptr() as usize;
            let ri = rbuf as usize;
            if ri != 0
            {
                let key = TermKeyEvent::from_c(self.tk, ckey);
                let off = ri - ci;
                let sbytelen = s.as_bytes().len();
                Some((key, s.slice_unchecked(off, sbytelen)))
            }
            else
            {
                None
            }
        }
    }
}
