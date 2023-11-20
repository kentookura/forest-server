use anyhow::{bail, Result};
use glob::glob;
use std::path::PathBuf;

enum BuildError {
    DirError,
    DefaultNotPresent,
    NoGlobsMatched,
}

pub struct Build {}
pub struct Tree {}

pub struct Forest {
    trees: Vec<Tree>,
}

fn validate_dirs(pattern: &str) -> Result<Vec<PathBuf>, BuildError> {
    if glob(pattern) {
        //.unwrap().any(|p| p.unwrap().exists()) {
        Ok(glob(pattern).unwrap().map(|r| r.unwrap()).collect())
    } else if pattern == "trees" {
        Err(BuildError::DefaultNotPresent)
    } else {
        Err(BuildError::NoGlobsMatched)
    }
}

async fn build(trees: &String, root: Option<String>) -> Result<Output, Error> {
    let cmd_args = format!(
        "build --dev {} {}",
        (root.inspect(|s| "--root ".push_str(root))),
        trees
    );
    Command::new("forester")
        .args(&["build", "--dev", "--root", "index", trees])
        .output()
        .await
}

impl Build {
    pub fn run(&mut self) -> Result<Forest, BuildError> {
        //let dirs = validate_dirs(self.trees.as_str());
        match dirs {
            Err(err) => {
                match err {
                    BuildError::DefaultNotPresent => {
                        "Default directory './trees' not found.".into()
                    }
                    BuildError::NoGlobsMatched => format!("No directory matched by {}", self.trees),
                };

                //bail!(
                //    "{} {}",
                //    msg,
                //    "Specify a path using --trees=<DIR>. You can use patterns such as 'forests/*'"
                //);
                //cmd.error(
                //    clap::error::ErrorKind::DisplayHelp,
                //    format!(
                //    "{}\n{}",
                //    msg,
                //    "Specify a path using --trees=<DIR>. You can use patterns such as 'forests/*'"
                //),
                //)
                //.exit();
            }
            Ok(_) => {
                //runtime
                //    .filterer(Arc::new(
                //        filt(&[], &["__latexindent*"], &["tree", "js", "css", "xsl"]).await,
                //    ))
                //    .pathset(paths);
            }
        };
        Ok(())
    }
}
