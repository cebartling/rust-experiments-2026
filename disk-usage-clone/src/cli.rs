use clap::Parser;

use crate::entry::SortOrder;

#[derive(Parser, Debug)]
#[command(name = "dusk", about = "Disk usage analysis tool", version)]
pub struct CliArgs {
    /// Paths to analyze (defaults to current directory)
    #[arg(default_value = ".")]
    pub paths: Vec<String>,

    /// Print sizes in human-readable format (e.g., 1.5K, 2.3M, 4.1G)
    #[arg(short = 'H', long = "human-readable")]
    pub human_readable: bool,

    /// Display only a total for each argument
    #[arg(short, long)]
    pub summarize: bool,

    /// Max depth of directory traversal
    #[arg(short = 'd', long = "max-depth")]
    pub max_depth: Option<usize>,

    /// Show all files, not just directories
    #[arg(short, long)]
    pub all: bool,

    /// Sort order: size, size-asc, name, none
    #[arg(long, default_value = "none")]
    pub sort: String,

    /// Number of threads for parallel traversal
    #[arg(short = 'j', long)]
    pub threads: Option<usize>,

    /// Disable colorized output
    #[arg(long)]
    pub no_color: bool,
}

impl CliArgs {
    pub fn sort_order(&self) -> SortOrder {
        SortOrder::parse(&self.sort).unwrap_or(SortOrder::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_args() {
        let args = CliArgs::parse_from(["dusk"]);
        assert_eq!(args.paths, vec!["."]);
        assert!(!args.human_readable);
        assert!(!args.summarize);
        assert_eq!(args.max_depth, None);
        assert!(!args.all);
        assert_eq!(args.sort, "none");
        assert_eq!(args.threads, None);
        assert!(!args.no_color);
    }

    #[test]
    fn test_human_readable_flag() {
        let args = CliArgs::parse_from(["dusk", "-H"]);
        assert!(args.human_readable);
    }

    #[test]
    fn test_summarize_flag() {
        let args = CliArgs::parse_from(["dusk", "-s"]);
        assert!(args.summarize);
    }

    #[test]
    fn test_max_depth() {
        let args = CliArgs::parse_from(["dusk", "-d", "3"]);
        assert_eq!(args.max_depth, Some(3));
    }

    #[test]
    fn test_all_flag() {
        let args = CliArgs::parse_from(["dusk", "-a"]);
        assert!(args.all);
    }

    #[test]
    fn test_sort_option() {
        let args = CliArgs::parse_from(["dusk", "--sort", "size"]);
        assert_eq!(args.sort, "size");
        assert_eq!(args.sort_order(), SortOrder::SizeDescending);
    }

    #[test]
    fn test_threads_option() {
        let args = CliArgs::parse_from(["dusk", "-j", "4"]);
        assert_eq!(args.threads, Some(4));
    }

    #[test]
    fn test_no_color_flag() {
        let args = CliArgs::parse_from(["dusk", "--no-color"]);
        assert!(args.no_color);
    }

    #[test]
    fn test_multiple_paths() {
        let args = CliArgs::parse_from(["dusk", "/tmp", "/var"]);
        assert_eq!(args.paths, vec!["/tmp", "/var"]);
    }

    #[test]
    fn test_combined_flags() {
        let args = CliArgs::parse_from(["dusk", "-H", "-s", "-a", "-d", "2", "--sort", "name"]);
        assert!(args.human_readable);
        assert!(args.summarize);
        assert!(args.all);
        assert_eq!(args.max_depth, Some(2));
        assert_eq!(args.sort_order(), SortOrder::Name);
    }

    #[test]
    fn test_sort_order_invalid_defaults_to_none() {
        let args = CliArgs::parse_from(["dusk", "--sort", "bogus"]);
        assert_eq!(args.sort_order(), SortOrder::None);
    }
}
