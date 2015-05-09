extern crate libc;
extern crate termkey;

use libc::c_int;

pub mod poll_
{
    use libc::c_short;
    use libc::c_int;
    use libc::c_ulong;

    pub static POLLIN: c_short = 0x001;
    pub static POLLPRI: c_short = 0x002;
    pub static POLLOUT: c_short = 0x004;

    pub static POLLRDNORM: c_short = 0x040;
    pub static POLLRDBAND: c_short = 0x080;
    pub static POLLWRNORM: c_short = 0x100;
    pub static POLLWRBAND: c_short = 0x200;

    pub static POLLMSG: c_short = 0x400;
    pub static POLLREMOVE: c_short = 0x1000;
    pub static POLLRDHUP: c_short = 0x2000;

    pub static POLLERR: c_short = 0x008;
    pub static POLLHUP: c_short = 0x010;
    pub static POLLNVAL: c_short = 0x020;

    #[repr(C)]
    #[allow(non_camel_case_types)]
    #[derive(Clone, Copy)]
    pub struct pollfd
    {
        pub fd: c_int,
        pub events: c_short,
        pub revents: c_short,
    }
    extern
    {
        pub fn poll(fds: *mut pollfd, nfds: c_ulong, timeout: c_int) -> c_int;
    }
}

pub fn poll_rd1(fd: isize, waittime: isize) -> isize
{
    let mut pfd = poll_::pollfd{fd: fd as c_int, events: poll_::POLLIN, revents: 0};
    unsafe
    {
        poll_::poll(&mut pfd, 1, waittime as c_int) as isize
    }
}


fn on_key(tk: &mut termkey::TermKey, key: termkey::TermKeyEvent)
{
    let s = tk.strfkey(key, termkey::c::TERMKEY_FORMAT_VIM);
    println!("{}", s);
}

fn main()
{
    let mut tk = termkey::TermKey::new(0, termkey::c::TERMKEY_FLAG_CTRLC);
    let mut running: bool = true;
    let mut nextwait = -1;

    while running
    {
        let p = poll_rd1(0, nextwait);
        if p == 0
        {
            match tk.getkey_force()
            {
                termkey::TermKeyResult::Key(key) => { on_key(&mut tk, key) }
                _ => {}
            }
        }
        if p > 0
        {
            tk.advisereadable();
        }
        loop
        {
            match tk.getkey()
            {
                termkey::TermKeyResult::Key(key) =>
                {
                    on_key(&mut tk, key);
                    match key
                    {
                        termkey::TermKeyEvent::UnicodeEvent{mods, codepoint, utf8: _} =>
                        {
                            if !(mods & termkey::c::TERMKEY_KEYMOD_CTRL).is_empty() && (codepoint == 'C' || codepoint == 'c')
                            {
                                running = false;
                            }
                        }
                        _ => {}
                    }
                }
                termkey::TermKeyResult::Again => { nextwait = tk.get_waittime(); break; }
                _ => { nextwait = -1; break; }
            }
        }
    }
}
