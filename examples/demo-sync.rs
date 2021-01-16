extern crate termkey;

fn main() {
    let mouse = 0; // TODO parse arg -m, default 1000
    let mouse_proto = 0; // TODO parse arg -p (no default)
    let format = termkey::c::Format::VIM;

    let mut tk = termkey::TermKey::new(0, termkey::c::Flag::SPACESYMBOL | termkey::c::Flag::CTRLC);
    if !(tk.get_flags() & termkey::c::Flag::UTF8).is_empty() {
        println!("Termkey in UTF-8 mode")
    }
    if !(tk.get_flags() & termkey::c::Flag::RAW).is_empty() {
        println!("Termkey in RAW mode")
    }
    if mouse != 0 {
        println!("\x1b[?{}hMouse mode active", mouse);
        if mouse_proto != 0 {
            println!("\x1b[?{}h", mouse_proto);
        }
    }
    loop {
        match tk.waitkey() {
            termkey::Result::Eof => break,
            termkey::Result::Key(key) => {
                let s = tk.strfkey(key, format);
                println!("Key {}", s);

                match key {
                    termkey::Event::Mouse {
                        mods: _,
                        ev: _,
                        button: _,
                        line,
                        col,
                    } => {
                        println!(
                            "Mouse (printing unimplemented, sorry) at line={}, col={}\n",
                            line, col
                        )
                    }
                    termkey::Event::Position { line, col } => {
                        println!("Cursor position report at line={}, col={}\n", line, col)
                    }
                    termkey::Event::ModeReport {
                        initial,
                        mode,
                        value,
                    } => {
                        let initial_str = if initial != 0 { "DEC" } else { "ANSI" };
                        println!("Mode report {} mode {} = {}\n", initial_str, mode, value)
                    }
                    termkey::Event::UnknownCsi => {
                        println!("Unrecognised CSI (printing unimplemented, sorry)\n")
                    }
                    _ => {}
                }
                if let termkey::Event::Unicode {
                    mods,
                    codepoint,
                    utf8: _,
                } = key
                {
                    if !(mods & termkey::c::KeyMod::CTRL).is_empty()
                        && (codepoint == 'C' || codepoint == 'c')
                    {
                        break;
                    }
                    if mods.is_empty() && codepoint == '?' {
                        // println!("\x1b[?6n"); // DECDSR 6 == request cursor position
                        println!("\x1b[?1$p"); // DECRQM == request mode, DEC origin mode
                    }
                }
            }
            termkey::Result::Error { err: _ } => {
                println!("Error of some sort")
            }
            _ => {
                panic!()
            }
        }
    }
    if mouse != 0 {
        println!("\x1b[?{}lMouse mode deactivated", mouse)
    }
}
