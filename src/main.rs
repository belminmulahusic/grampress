#[macro_use]
mod macros;

mod cli;
mod compress;
mod decompress;
mod search;
mod utils;

#[derive(Clone, Copy)]
pub struct Flags {
    pub force: bool,
    pub quiet: bool,
    pub verbose: bool,
    pub sequitur: bool,
    pub bisection: bool,
    pub no_huffman: bool,
}

fn main() {
    let args = cli::parse_args();
    let flags = Flags {
        force: args.force,
        quiet: args.quiet,
        verbose: args.verbose,
        sequitur: args.sequitur,
        bisection: args.bisection,
        no_huffman: args.no_huffman,
    };

    if args.decompress {
        decompress::run(&args.input, flags);
    } else if args.list {
        decompress::list(&args.input);
    } else if let Some(pattern) = args.search {
        search::run(&args.input, &pattern, flags);
    } else {
        compress::run(&args.input, flags);
    }
}
