use std::sync::Mutex;

// XXX: Using mutexes is inelegant. It is just to keep the compiler happy.
pub static SOURCE_NAME: Mutex<String> = Mutex::new(String::new());

pub static SOURCE_LINE: Mutex<u32> = Mutex::new(0);
pub static SOURCE_COL: Mutex<u32> = Mutex::new(0);

pub static COLUMN_NUM: Mutex<u32> = Mutex::new(0);

pub enum RickError {
    UnclosedString,
    NumberParseFailure,
    IllegalCharacter(u8),
    IllegalEscapeCode(char),
    IdentifierTooLong,
    NonPrintableInString(u8),
}

pub fn inc_col() {
    *COLUMN_NUM.lock().unwrap() += 1;
}

pub fn inc_line() {
    *SOURCE_LINE.lock().unwrap() += 1;
    *COLUMN_NUM.lock().unwrap() = 1;
}

pub fn set_loc(line: u32, col: u32) {
    *SOURCE_LINE.lock().unwrap() = line;
    *SOURCE_COL.lock().unwrap() = col;
}

pub fn set_col(col: u32) {
    *SOURCE_COL.lock().unwrap() = col;
}

pub fn save_loc() {
    *SOURCE_COL.lock().unwrap() = *COLUMN_NUM.lock().unwrap();
}

pub fn set_source(filename: String) {
    *SOURCE_NAME.lock().unwrap() = filename;
}

pub fn get_loc() -> (u32, u32) {
    (
        *SOURCE_LINE.lock().unwrap(),
        *SOURCE_COL.lock().unwrap(),
    )
}

pub fn report_err(reason: RickError) -> ! {
    match reason {
        RickError::UnclosedString => { eprintln!("rick: {}: {}:{} error: string not closed",
            SOURCE_NAME.lock().unwrap(),
            SOURCE_LINE.lock().unwrap(),
            SOURCE_COL.lock().unwrap());
        },

        RickError::NumberParseFailure => { eprintln!("rick: {}: {}:{} error: failed to parse number literal",
            SOURCE_NAME.lock().unwrap(),
            SOURCE_LINE.lock().unwrap(),
            SOURCE_COL.lock().unwrap());
        },

        RickError::IllegalCharacter(d) => { eprintln!("rick: {}: {}:{} error: illegal character (ASCII #{})",
            SOURCE_NAME.lock().unwrap(),
            SOURCE_LINE.lock().unwrap(),
            SOURCE_COL.lock().unwrap(),
            d);
        },

        RickError::IllegalEscapeCode(c) => { eprintln!("rick: {}: {}:{} error: illegal escape code '\\{}' in string",
            SOURCE_NAME.lock().unwrap(),
            SOURCE_LINE.lock().unwrap(),
            SOURCE_COL.lock().unwrap(),
            c);
        },

        RickError::IdentifierTooLong => { eprintln!("rick: {}: {}:{} error: identifier too long",
            SOURCE_NAME.lock().unwrap(),
            SOURCE_LINE.lock().unwrap(),
            SOURCE_COL.lock().unwrap());
        },

        RickError::NonPrintableInString(d) => { eprintln!("rick: {}: {}:{} error: non-printable character (ASCII #{}) in string",
            SOURCE_NAME.lock().unwrap(),
            SOURCE_LINE.lock().unwrap(),
            SOURCE_COL.lock().unwrap(),
            d);
        },

        _ => {
            eprintln!("Unreachable: Error message not yet implemented.");
        },
    }

    std::process::exit(1);
}