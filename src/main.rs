#[macro_use]
extern crate clap;
extern crate assert_cli;
extern crate ignore;

use self::display::draw_it;
use clap::{App, AppSettings, Arg};
use utils::{find_big_ones, get_dir_tree};

mod display;
mod utils;
mod lib;

static DEFAULT_NUMBER_OF_LINES: &'static str = "15";

fn main() {

    let options = App::new("Dust")
        .setting(AppSettings::TrailingVarArg)
        .arg(
            Arg::with_name("number_of_lines")
                .short("n")
                .help("Number of lines of output to show")
                .takes_value(true)
                .default_value(DEFAULT_NUMBER_OF_LINES),
        )
        .arg(
            Arg::with_name("use_apparent_size")
                .short("s")
                .help("If set will use file length. Otherwise we use blocks"),
        )
        .arg(Arg::with_name("inputs").multiple(true))
        .get_matches();

    let filenames = {
        match options.values_of("inputs") {
            None => vec!["."],
            Some(r) => r.collect(),
        }
    };
    let number_of_lines = value_t!(options.value_of("number_of_lines"), usize).unwrap();
    let use_apparent_size = options.is_present("use_apparent_size");

    let (permissions, nodes) = get_dir_tree(&filenames, use_apparent_size);
    let slice_it = find_big_ones(nodes, number_of_lines);
    draw_it(permissions, &slice_it);
}

#[cfg(test)]
mod tests;
