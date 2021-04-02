use std::fs;
use std::io::Write;

use actix_multipart::Multipart;
use actix_web::{middleware, web, App, Error, HttpResponse, HttpServer};
use futures::{StreamExt, TryStreamExt};

use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize)]
pub struct PathName {
    pub nombre: String,
}

async fn save_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let filename = content_type.get_filename().unwrap();
        let filepath = format!("./tmp/{}", sanitize_filename::sanitize(&filename));

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
    let paths: Vec<PathName> = fs::read_dir("./")
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

// pub async fn deleteFile(id: u32) -> u32 {}
//
// pub async fn getFile(id: u32) -> File {}
