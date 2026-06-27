#![no_std]
#![cfg_attr(not(test), no_main)]

mod cache;
mod dir;
mod entry_table;
mod global_alloc;
mod output_config;
mod utils;
use dir::OpenDir;
mod errors;
mod io;
use io::{OutputLong, OutputShort};
use output_config::{Alignments, Display, MAX_INT_LEN, Mode, ReturnConfig, Sort, Width};

use crate::{
    entry_table::{LongTable, ShortTable},
    errors::FliResult,
    output_config::Mode::Stream,
};
extern crate alloc;

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { libc::abort() }
}

fn run(argc: i32, argv: *const *mut libc::c_char) -> FliResult<()> {
    // default is short and sorted by name output , ie Alloc(Sort::Name)
    let mut config = ReturnConfig::default();
    loop {
        let opt = unsafe { libc::getopt(argc, argv, c"lStU".as_ptr()) };
        if opt == -1 {
            break;
        }
        // simple ls like logic - last flag wins:
        // ls -l -U -t == long and sort by time
        // ls -l -t -U == long and unsrted
        match opt as u8 {
            b'S' => config.mode = Mode::Alloc(Sort::Size),
            b't' => config.mode = Mode::Alloc(Sort::Time),
            b'l' => {
                config.mode = Mode::Alloc(Sort::Name);
                config.display = Display::Long;
            }
            b'U' => config.mode = Stream,
            _ => {}
        }
    }

    match (config.mode, config.display) {
        (Mode::Stream, Display::Short) => {
            let dir = OpenDir::new(config.path)?;
            //https://www.man7.org/linux/man-pages/man3/readdir.3.html
            // here we need to be very careful , readdir() returns raw pointer to the next entry,
            // so after each iteration, we can consider it as invalid and should not use after
            let mut output = OutputShort::new();
            dir.into_iter().for_each(|entry| output.stream_short(entry));
        }
        (Mode::Stream, Display::Long) => {
            let alignments = Alignments {
                n_link_width: Width::new(MAX_INT_LEN)?,
                size_width: Width::new(MAX_INT_LEN)?,
            };
            let mut output = OutputLong::new(alignments);
            let dir = OpenDir::new(config.path)?;
            dir.into_iter()
                .try_for_each(|entry| output.stream_long(entry))?;
        }
        (Mode::Alloc(_), Display::Short) => {
            let mut output = OutputShort::new();
            let mut arena = ShortTable::new();
            OpenDir::new(config.path)?.try_for_each(|e| arena.push(e))?;
            arena.sort_by();
            output.push_arena_short(arena);
        }
        (Mode::Alloc(sort), Display::Long) => {
            let mut arena = LongTable::new();
            OpenDir::new(config.path)?.try_for_each(|e| arena.push(e))?;
            arena.sort_by(sort);
            let alignments = arena.get_alignments()?;
            let mut output = OutputLong::new(alignments);
            output.push_arena_long(arena)?;
        }
    }
    Ok(())
}

// so seems libc handles linker and there is no entry hassle

#[cfg_attr(not(test), unsafe(no_mangle))]
fn main(argc: i32, argv: *const *mut libc::c_char) -> i32 {
    match run(argc, argv) {
        Ok(()) => 0,
        Err(e) => e.to_exit_code(),
        //add error printing here
    }
}
