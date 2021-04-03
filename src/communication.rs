#![allow(dead_code)]

use std::fs;
use std::io::Write;

use actix_multipart::Multipart;
use actix_web::{web, Error, HttpResponse};
use futures::{StreamExt, TryStreamExt};

use reqwest::blocking::Response;
use serde::{Deserialize, Serialize};
use url::Url;

const DIRNAME: &str = "tmp";

pub fn get_dir() -> String {
    format!("./{dir}", dir = DIRNAME)
}

#[derive(Deserialize, Serialize)]
pub struct PathName {
    pub nombre: String,
}

fn get_file_path(filename: &str) -> String {
    format!("{}/{}", get_dir(), sanitize_filename::sanitize(filename))
}

async fn save_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();
        let filepath = get_file_path(filename);

        // File::create is blocking operation, use threadpool
        let mut f = web::block(|| std::fs::File::create(filepath))
            .await
            .unwrap();

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&data).map(|_| f)).await?;
        }
    }
    Ok(HttpResponse::Ok().into())
}

pub fn get_files() -> Vec<PathName> {
    let paths: Vec<PathName> = fs::read_dir(get_dir())
        .unwrap()
        .map(|r| -> PathName {
            PathName {
                nombre: if let Ok(a) = r {
                    format!("{}", a.path().display())
                } else {
                    "".to_string()
                },
            }
        })
        .collect();

    paths
}

pub fn parse_json_file_list(json: String) -> Result<Vec<PathName>, Error> {
    let array: Vec<PathName> = serde_json::from_str(&json)?;
    Ok(array)
}

pub async fn delete_file(file_name: &str) -> Result<(), Error> {
    let filepath = get_file_path(file_name);
    Ok(fs::remove_file(filepath)?)
}
//
// pub async fn getFile(id: u32) -> File {}

pub async fn get(url: &str) -> Result<Response, ()> {
    println!("{}", url);
    let url = Url::parse(url);
    let response;
    match reqwest::blocking::get(url.unwrap().as_str()) {
        Ok(a) => {
            response = a;
        }
        Err(err) => {
            println!("error: {}", err);
            return Err(());
        }
    };

    Ok(response)
}

// pub async fn post(url: &str) -> Result<Response, ()> {
//     // let request_url = format!(
//     //     "https://api.github.com/repos/{owner}/{repo}/stargazers",
//     //     owner = "rust-lang-nursery",
//     //     repo = "rust-cookbook"
//     // );
//     println!("{}", url);
//     let response;
//     if let Ok(a) = reqwest::post(url).await {
//         response = a;
//     } else {
//         return Err(());
//     };
//
//     Ok(response)
// }
//
// pub async fn post() -> Result<String> {
//     let body = reqwest::get("https://www.rust-lang.org")
//         .await?
//         .text()
//         .await?;
//
//     println!("body = {:?}", body);
//     Ok(body)
// }
