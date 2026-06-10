#![no_std]
#![no_main]

// TODO:
// - add tests

mod dir;
mod entry_table;
mod global_alloc;
mod output_config;
mod utils;
use dir::OpenDir;
use entry_table::EntryTable;
mod errors;
mod io;
use io::Output;
use output_config::{Alignments, Display, Mode, ReturnConfig, Sort};
use utils::MAX_INT_LEN;

use crate::errors::FliResult;
extern crate alloc;

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { libc::abort() }
}

fn run(argc: i32, argv: *const *mut libc::c_char) -> FliResult<()> {
    let mut config = ReturnConfig::default();
    let mut sort: Option<Sort> = None;

    loop {
        let opt = unsafe { libc::getopt(argc, argv, c"slS".as_ptr()) };
        if opt == -1 {
            break;
        }
        match opt as u8 {
            b's' => sort = Some(Sort::Name),
            b'S' => sort = Some(Sort::Size),
            b'l' => config.display = Display::Long,
            _ => {}
        }
    }

    if let Some(s) = sort {
        config.mode = Mode::Alloc(s)
    }
    let mut output = Output::new(None);

    match (config.mode, config.display) {
        (Mode::Stream, Display::Short) => {
            let dir = OpenDir::new(config.path)?;
            //https://www.man7.org/linux/man-pages/man3/readdir.3.html
            // here we need to be very careful , readdir() returns raw pointer to the next entry,
            // so after each iteration, we can consider it as invalid and should not use after
            for entry in dir {
                output.stream_short(entry);
            }
        }
        (Mode::Stream, Display::Long) => {
            let alignments = Alignments {
                n_link_width: MAX_INT_LEN,
                size_width: MAX_INT_LEN,
            };
            let dir = OpenDir::new(config.path)?;
            output.alignments = Some(alignments);
            for entry in dir {
                output.stream_long(entry)?;
            }
        }
        (Mode::Alloc(sort), Display::Short) => {
            let mut arena = EntryTable::new();
            let dir = OpenDir::new(config.path)?;
            for entry in dir {
                arena.push_short(entry);
            }
            match sort {
                Sort::Name => arena.sort_by_name(),
                Sort::Size => (),
            }
            output.push_arena_short(arena);
        }
        (Mode::Alloc(sort), Display::Long) => {
            let mut arena = EntryTable::new();
            let dir = OpenDir::new(config.path)?;
            for entry in dir {
                arena.push_long(entry)?;
            }
            match sort {
                Sort::Name => arena.sort_by_name(),
                Sort::Size => (),
            }
            output.alignments = arena.get_alignments();
            if output.alignments.is_some() {
                output.push_arena_long(arena);
            }
        }
    }
    output.flush();
    Ok(())
}

// so seems libc handles linker and there is no entry hassle
#[unsafe(no_mangle)]
fn main(argc: i32, argv: *const *mut libc::c_char) -> i32 {
    match run(argc, argv) {
        Ok(()) => 0,
        Err(e) => e.to_exit_code(),
        //add error printing here
    }
}
