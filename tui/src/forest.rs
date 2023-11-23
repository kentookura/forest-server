use actix_web::HttpResponse;
use anyhow::Result;
use askama::Template;
use askama_actix::TemplateToResponse;
use glob::glob;
use std::{path::PathBuf, process::Output};
use tokio::process::Command;

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
    if glob(pattern).unwrap().any(|p| p.unwrap().exists()) {
        Ok(glob(pattern).unwrap().map(|r| r.unwrap()).collect())
    } else if pattern == "trees" {
        Err(BuildError::DefaultNotPresent)
    } else {
        Err(BuildError::NoGlobsMatched)
    }
}

pub async fn build(trees_dir: &String, root: String) -> Result<Output, std::io::Error> {
    let cmd_args = format!("build --dev --root {} {}", root, trees_dir);
    Command::new("forester").args(&[cmd_args]).output().await
}

#[derive(Template)]
#[template(path = "trees.html")]
pub struct TreeTemplate<'a> {
    pub trees: &'a Vec<Tree>,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct HomeTemplate {}

//pub struct HomeTemplate<'a> {
//    pub Trees: &'a Vec<Tree>,
//}

pub async fn index() -> HttpResponse {
    let home = HomeTemplate {};
    home.to_response()
    //match Tree::get_trees().await {
    //    Ok(trees) => {
    //        let home = HomeTemplate;// { trees: &trees };
    //        home.to_response()
    //    }
    //    Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    //}
}
