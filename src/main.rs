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
    dir::is_dir,
    entry_table::{LongTable, ShortTable},
    errors::FliResult,
    output_config::{Mode::Stream, View},
    utils::base_dir_names,
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
    // getopt updates external var optind
    // no optind in rust libc
    unsafe extern "C" {
        static mut optind: i32;
    }
    loop {
        //https://sourceware.org/glibc/manual/2.43/html_mono/libc.html
        // POSIX demands the following behavior: the first non-option stops option processing.
        // This mode is selected by either setting the environment variable POSIXLY_CORRECT or beginning the options argument string with a plus sign (‘+’).
        let opt = unsafe { libc::getopt(argc, argv, c"+lrStU02".as_ptr()) };
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
            b'2' => config.view = View::Text,
            b'0' => config.view = View::Color,
            b'r' => config.is_reverse = true,
            _ => {}
        }
    }
    unsafe {
        if optind < argc {
            config.path = *argv.add(optind as usize)
        }
    };

    if !is_dir(config.path)? {
        config.is_single_file = true;
    }

    //rewrite to a single option..
    match (config.mode, config.display, config.is_single_file) {
        (_, _, true) => {
            let (dir_name, base_name) = base_dir_names(config.path.cast_mut());
            config.path = dir_name;
            let name_cstr = unsafe { core::ffi::CStr::from_ptr(base_name) };
            let mut dir = OpenDir::new(config.path)?;
            let entry = dir
                .find(|e| e.name() == name_cstr)
                .ok_or(errors::FliError::FindEntry)?;
            if config.display == Display::Long {
                let mut arena = LongTable::new();
                arena.push(entry)?;
                let alignments = arena.get_alignments()?;
                let mut output = OutputLong::new(alignments);
                output.push_arena_long(arena, config.view)?;
            } else {
                let mut output = OutputShort::new();
                output.stream_short(entry, config.view);
            }
        }
        (Mode::Stream, Display::Short, false) => {
            let dir = OpenDir::new(config.path)?;
            //https://www.man7.org/linux/man-pages/man3/readdir.3.html
            // here we need to be very careful , readdir() returns raw pointer to the next entry,
            // so after each iteration, we can consider it as invalid and should not use after
            let mut output = OutputShort::new();
            dir.into_iter()
                .for_each(|entry| output.stream_short(entry, config.view));
        }
        (Mode::Stream, Display::Long, false) => {
            let alignments = Alignments {
                n_link_width: Width::new(MAX_INT_LEN)?,
                size_width: Width::new(MAX_INT_LEN)?,
            };
            let mut output = OutputLong::new(alignments);
            let dir = OpenDir::new(config.path)?;
            dir.into_iter()
                .try_for_each(|entry| output.stream_long(entry, config.view))?;
        }
        (Mode::Alloc(_), Display::Short, false) => {
            let mut output = OutputShort::new();
            let mut arena = ShortTable::new();
            OpenDir::new(config.path)?.try_for_each(|e| arena.push(e))?;
            arena.sort_by(config.is_reverse);
            output.push_arena_short(arena, config.view);
        }
        (Mode::Alloc(sort), Display::Long, false) => {
            let mut arena = LongTable::new();
            OpenDir::new(config.path)?.try_for_each(|e| arena.push(e))?;
            arena.sort_by(sort, config.is_reverse);
            let alignments = arena.get_alignments()?;
            let mut output = OutputLong::new(alignments);
            output.push_arena_long(arena, config.view)?;
        }
    }
    Ok(())
}

// so seems libc handles linker and there is no entry hassle

#[cfg_attr(not(test), unsafe(no_mangle))]
fn main(argc: i32, argv: *const *mut libc::c_char) -> i32 {
    match run(argc, argv) {
        Ok(()) => 0,
        Err(e) => e as i32,
        //add error printing here
    }
}
