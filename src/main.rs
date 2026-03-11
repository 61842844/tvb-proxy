use axum::{extract::{Path, State, Query}, response::{IntoResponse, Response}, routing::get, Router};
use std::sync::Arc;
use aes::Aes128;
use ctr::cipher::{KeyIvInit, StreamCipher};
use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Cursor, Read};

type Aes128Ctr = ctr::Ctr128BE<Aes128>;

struct AppState {
    client: reqwest::Client,
    key: [u8; 16],
}

#[tokio::main]
async fn main() {
    // 从环境变量获取 KEY，若无则使用默认值
    let key_str = std::env::var("KEY").unwrap_or_else(|_| "2d2fd7b1661b1e28de38268872b48480".to_string());
    let clean_key = key_str.split(':').last().unwrap_or(&key_str).trim();
    let key_vec = hex::decode(clean_key).expect("Invalid Hex Key");
    let mut key = [0u8; 16];
    key.copy_from_slice(&key_vec);

    let state = Arc::new(AppState {
        client: reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build().unwrap(),
        key,
    });

    let app = Router::new()
        .route("/proxy/*path", get(proxy_handler))
        .with_state(state);

    let addr = "0.0.0.0:8080";
    println!("服务已启动，监听: {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn proxy_handler(
    Path(path): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let base_url = "http://edgeware-live.edgeware.tvb.com";
    let mut url = format!("{}/{}", base_url, path);
    
    if !params.is_empty() {
        let query_str: String = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        url = format!("{}?{}", url, query_str);
    }

    let Ok(resp) = state.client.get(&url).send().await else {
        return Response::builder().status(500).body(axum::body::Body::from("Request Failed")).unwrap();
    };
    
    let mut data = resp.bytes().await.unwrap().to_vec();

    if path.ends_with(".m4s") {
        if let Some(pos) = data.windows(4).position(|w| w == b"senc") {
            let mut reader = Cursor::new(&data[pos + 12..]);
            let mut iv = [0u8; 16];
            let _ = reader.read_exact(&mut iv[..8]); 
            
            if let Some(mdat_pos) = data.windows(4).position(|w| w == b"mdat") {
                let mut cipher = Aes128Ctr::new(&state.key.into(), &iv.into());
                cipher.apply_keystream(&mut data[mdat_pos + 8..]);
            }
        }
    }

    Response::builder()
        .header("Access-Control-Allow-Origin", "*")
        .header("Content-Type", "video/mp4")
        .body(axum::body::Body::from(data))
        .unwrap()
}
