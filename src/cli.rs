use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "gpress", about = "A simple compression tool.")]

pub struct Cli {
    /// Decompress a .gps file
    #[arg(short = 'd', long, conflicts_with_all = &["search", "list"])]
    pub decompress: bool,

    /// Overwrite existing files
    #[arg(short = 'f', long)]
    pub force: bool,

    /// List file details
    #[arg(short = 'l', long, conflicts_with_all = &["search", "verbose", "force"])]
    pub list: bool,

    /// Suppress error messages
    #[arg(short = 'q', long, conflicts_with_all = &["verbose"])]
    pub quiet: bool,

    /// Verbose output
    #[arg(short = 'v', long, conflicts_with_all = &["quiet"])]
    pub verbose: bool,

    /// Search for a String in a compressed file
    #[arg(short = 's', long, value_name = "STRING", conflicts_with_all = &["decompress", "list"], allow_hyphen_values = true)]
    pub search: Option<String>,

    /// Use Sequitur as compression algorithm
    #[arg(long = "sequitur", conflicts_with_all = &["decompress", "list"])]
    pub sequitur: bool,
    #[arg(long = "bisection", conflicts_with_all = &["decompress", "list"])]
    pub bisection: bool,

    /// Disable Huffman coding
    #[arg(long = "no-huffman")]
    pub no_huffman: bool,

    #[arg(value_name = "INPUT FILE")]
    pub input: PathBuf,
}

pub fn parse_args() -> Cli {
    Cli::parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_compress() {
        let args = Cli::try_parse_from(&["gpress", "input.txt"]).unwrap();
        assert!(!args.decompress);
        assert!(!args.list);
        assert!(args.search.is_none());
        assert!(!args.force);
        assert!(!args.quiet);
        assert!(!args.verbose);
        assert_eq!(args.input.to_str().unwrap(), "input.txt");
    }

    #[test]
    fn test_parse_decompress() {
        let args = Cli::try_parse_from(&["gpress", "-d", "input.gps"]).unwrap();
        assert!(args.decompress);
        assert_eq!(args.input.to_str().unwrap(), "input.gps");
        assert!(!args.list);
        assert!(args.search.is_none());
    }

    #[test]
    fn test_parse_search() {
        let args = Cli::try_parse_from(&["gpress", "-s", "pattern", "input.gps"]).unwrap();
        assert!(args.search.is_some());
        assert_eq!(args.search.unwrap(), "pattern");
        assert_eq!(args.input.to_str().unwrap(), "input.gps");
        assert!(!args.decompress);
        assert!(!args.list);
    }

    #[test]
    fn test_parse_list() {
        let args = Cli::try_parse_from(&["gpress", "-l", "input.gps"]).unwrap();
        assert!(args.list);
        assert_eq!(args.input.to_str().unwrap(), "input.gps");
    }

    #[test]
    fn test_conflicting_args_error() {
        let args = Cli::try_parse_from(&["gpress", "-d", "-s", "pattern", "input.gps"]);
        assert!(args.is_err());

        let args = Cli::try_parse_from(&["gpress", "-l", "-s", "pattern", "input.gps"]);
        assert!(args.is_err());

        let args = Cli::try_parse_from(&["gpress", "-d", "-l", "input.gps"]);
        assert!(args.is_err());
    }
}
