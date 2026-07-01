use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use rsomics_common::{run, CommonFlags, RsomicsError, ToolMeta};

use rsomics_modularity::modularity_from_edge_list;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

/// Newman-Girvan modularity Q of a graph partition
/// (`networkx.community.modularity`, undirected + unweighted).
///
/// Reads an undirected edge list (`u v` per line; `#` comments and blank lines
/// skipped; string node names; parallel edges deduplicated and self-loops
/// dropped as in a simple `nx.Graph`). Every node must be assigned to exactly
/// one community in `--communities`, or the tool fails loud (networkx's
/// `NotAPartition`). Community iteration order is the first-appearance order of
/// the labels in that file. Output is the single float Q.
#[derive(Parser, Debug)]
#[command(name = "rsomics-modularity", version, about, long_about = None)]
pub struct Cli {
    /// Edge list; `-` or omitted reads stdin.
    #[arg(value_name = "EDGES")]
    pub edges: Option<PathBuf>,

    /// Node-to-community assignment: `node community_label` per line.
    #[arg(long, value_name = "FILE")]
    pub communities: PathBuf,

    /// Resolution parameter γ.
    #[arg(long, default_value_t = 1.0)]
    pub resolution: f64,

    #[command(flatten)]
    pub common: CommonFlags,
}

fn parse_assignments(text: &str) -> Result<Vec<(String, String)>, RsomicsError> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let (Some(node), Some(label)) = (parts.next(), parts.next()) else {
            return Err(RsomicsError::InvalidInput(format!(
                "community line needs `node label`: {line:?}"
            )));
        };
        out.push((node.to_owned(), label.to_owned()));
    }
    Ok(out)
}

impl Cli {
    pub fn run(self) -> ExitCode {
        let common = self.common.clone();
        run(&common, META, || {
            let mut input = String::new();
            match &self.edges {
                Some(p) if p.as_os_str() != "-" => {
                    fs::File::open(p)
                        .map_err(RsomicsError::Io)?
                        .read_to_string(&mut input)
                        .map_err(RsomicsError::Io)?;
                }
                _ => {
                    io::stdin()
                        .lock()
                        .read_to_string(&mut input)
                        .map_err(RsomicsError::Io)?;
                }
            }
            let comm_text = fs::read_to_string(&self.communities).map_err(RsomicsError::Io)?;
            let assignments = parse_assignments(&comm_text)?;

            let result = modularity_from_edge_list(&input, &assignments, self.resolution)?;
            if !common.json {
                println!("{:.17}", result.modularity);
            }
            Ok(result)
        })
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    #[test]
    fn cli_debug_assert() {
        super::Cli::command().debug_assert();
    }
}
