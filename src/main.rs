use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use dotenv::dotenv;
use reqwest::header;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

#[derive(Debug, Serialize, Deserialize)]
#[derive(Clone)]
pub struct Format {
    pub ext: String,
    pub url: String,
    pub hash: String,
    pub mime: String,
    pub name: String,
    pub path: String,
    pub size: f64,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[derive(Clone)]
pub struct Formats {
    pub small: Format,
    pub thumbnail: Format,
}

#[derive(Debug, Serialize, Deserialize)]
#[derive(Clone)]
pub struct Image {
    pub id: i32,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "alternativeText")]
    pub alternative_text: Option<String>,
    pub caption: Option<String>,
    pub width: i32,
    pub height: i32,
    pub formats: Formats,
    #[serde(rename = "hash")]
    pub hash: String,
    pub ext: String,
    pub mime: String,
    pub size: f64,
    #[serde(rename = "url")]
    pub url: String,
    #[serde(rename = "previewUrl")]
    pub preview_url: Option<String>,
    pub provider: String,
    pub provider_metadata: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub blurhash: String,
    pub placeholder: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let base_url = env::var("BASE_URL").expect("BASE_URL is not set");
    let auth_token = env::var("AUTH_TOKEN").expect("AUTH_TOKEN is not set");
    let api_key = env::var("CLOUDINARY_KEY").expect("API_KEY is not set");
    let api_secret = env::var("CLOUDINARY_SECRET").expect("API_SECRET is not set");
    let cloudinary_url = env::var("CLOUDINARY_URL").expect("CLOUDINARY_URL is not set");
    let folder_name = env::var("FOLDER_NAME").expect("FOLDER_NAME is not set");
    let chunk_size = env::var("CHUNK_SIZE").expect("CHUNK_SIZE is not set");

    let client = reqwest::Client::new();
    let url = format!("{}/{}", &base_url, "api/upload/files");
    let authorization_header = format!("Bearer {}", &auth_token);
    let response = client
        .get(&url)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, &authorization_header)
        .send()
        .await;

    match response {
        Ok(response) => {
            let response = response.json::<Vec<Image>>().await;
            match response {
                Ok(response) => {
                    let parsed_chunk_size = chunk_size.parse::<usize>();

                    if parsed_chunk_size.is_err() {
                        println!("Error: {:?}", parsed_chunk_size.err());
                        return;
                    }

                    let chunks = response.chunks(parsed_chunk_size.unwrap())
                        .map(|chunk| chunk.to_vec())
                        .collect::<Vec<_>>();

                    for chunk in chunks {
                        let tasks: Vec<_> = chunk
                            .into_iter()
                            .map(|image| {
                                let api_key = api_key.clone();
                                let api_secret = api_secret.clone();
                                let cloudinary_url = cloudinary_url.clone();
                                let folder_name = folder_name.clone();
                                let authorization_header = authorization_header.clone();
                                let client = client.clone();
                                let base_url = base_url.clone();

                                tokio::spawn(async move {
                                    let _upload_result = upload_to_cloudinary(
                                        &client,
                                        &image,
                                        &base_url,
                                        &api_key,
                                        &api_secret,
                                        &cloudinary_url,
                                        &folder_name,
                                        &authorization_header,
                                    )
                                    .await;
                                })
                            })
                            .collect();

                        // Wait for this chunk of tasks to finish before moving onto the next chunk
                        let results: Vec<_> = futures::future::join_all(tasks).await;

                        for result in results {
                            match result {
                                Ok(_) => println!("Upload successful"),
                                Err(e) => println!("Error: {:?}", e),
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}

async fn upload_to_cloudinary(
    client: &reqwest::Client,
    image: &Image,
    base_url: &String,
    api_key: &String,
    api_secret: &String,
    cloudinary_url: &String,
    folder_name: &String,
    authorization_header: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let image_url = format!("{}{}", base_url, &image.url);
    let image_name = &image.name;

    let image_data = client
        .get(&image_url)
        .header(header::AUTHORIZATION, authorization_header)
        .send()
        .await?
        .bytes()
        .await?;

    let image_base64 = base64::encode(&image_data);
    let image_base64_for_cloudinary = format!("data:image/jpg;base64,{}", image_base64);

    let signature = generate_signature(
        folder_name,
        &image_name,
        &image_name,
        current_timestamp().parse::<i64>().unwrap(),
        api_secret,
    );

    let form = reqwest::multipart::Form::new()
        .text("folder", folder_name.clone())
        .text("file", image_base64_for_cloudinary)
        .text("public_id", image_name.clone())
        .text("api_key", api_key.clone())
        .text("timestamp", current_timestamp())
        .text("display_name", image_name.clone())
        .text("signature", signature.clone())
        .text("api_secret", api_secret.clone());

    post_to_cloudinary(&client, &cloudinary_url, form).await?;

    return Ok(());
}

async fn post_to_cloudinary(
    client: &reqwest::Client,
    cloudinary_url: &String,
    form: reqwest::multipart::Form,
) -> Result<(), Box<dyn std::error::Error>> {
    let _response = client
        .post(cloudinary_url)
        .multipart(form)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    return Ok(());
}

fn generate_signature(
    asset_folder: &str,
    display_name: &str,
    public_id: &str,
    timestamp: i64,
    api_secret: &str,
) -> String {
    let binding = timestamp.to_string();
    let mut params = vec![
        ("folder", asset_folder),
        ("display_name", display_name),
        ("public_id", public_id),
        ("timestamp", &binding),
    ];

    params.sort();

    let string_to_hash = params
        .into_iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<String>>()
        .join("&")
        + api_secret;

    let mut s = Sha1::new();
    s.update(string_to_hash.as_bytes());
    let digest = s.finalize();
    let hex_string = digest
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    hex_string
}

// Function to get the current timestamp
fn current_timestamp() -> String {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs().to_string()
}
