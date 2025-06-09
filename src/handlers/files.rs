use actix_web::{web, HttpResponse, Error};
use diesel::prelude::*;
use uuid::Uuid;
use std::{io::Write, path::Path};
use futures_util::StreamExt;

use crate::{models::{File, NewFile}, database::DbPool};

const UPLOAD_DIR: &str = "./uploads/";

pub async fn upload_file(
    pool: web::Data<DbPool>,
    mut payload: web::Payload,
) -> Result<HttpResponse, Error> {
    use crate::schema::files::dsl::*;

    std::fs::create_dir_all(UPLOAD_DIR)?;

    let file_id = Uuid::new_v4();
    let file_name = format!("{}.data", file_id);
    let file_path = Path::new(UPLOAD_DIR).join(&file_name);

    let mut file = std::fs::File::create(&file_path)?;
    let mut bytes: i64 = 0;

    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        bytes += chunk.len() as i64;
        file.write_all(&chunk)?;
    }

    let new_file = NewFile {
        name: file_id.to_string(),
        storage_path: file_path.to_string_lossy().to_string(),
        size: bytes,
        mime_type: None,
    };

    let mut conn = pool.get().unwrap();
    diesel::insert_into(files)
        .values(&new_file)
        .execute(&mut conn).unwrap();

    Ok(HttpResponse::Ok().json("File uploaded successfully"))
}

