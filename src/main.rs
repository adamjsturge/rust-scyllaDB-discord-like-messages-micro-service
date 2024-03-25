use std::sync::Arc;

use axum::{body::Body, http::StatusCode, response::{Html, IntoResponse, Response}, routing::{delete, get, post, put}, Json, Router};
use scylla::{frame::Compression, load_balancing::DefaultPolicy, transport::errors::{NewSessionError, QueryError}, ExecutionProfile, IntoTypedRows, Session, SessionBuilder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
struct MessageRequest {
    channel_id: Option<i64>,
    message_id: Option<i64>,
    author_id: Option<i64>,
    content: Option<String>,
    message_id_offset: Option<i64>,    
}

#[derive(Serialize, Debug)]
struct MessageResponse {
    channel_id: i64,
    message_id: i64,
    author_id: i64,
    content: String,
}

#[tokio::main]
async fn main() {
    init_db()
        .await
        .expect("Failed to initialize database");

    test_everything().await;

    let app = Router::new()
    .route("/", get(handler))
    .route("/health", get(health_handler))
    .route("/messages", get(get_messages_handler))
    .route("/messages", post(add_message_handler))
    .route("/messages", delete(delete_message_handler))
    .route("/messages", put(edit_message_handler))
    ;

    let listener = tokio::net::TcpListener::bind("172.20.0.10:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn health_handler() -> Html<&'static str> {
    Html("<h1>OK</h1>")
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn test_everything() {
    let mut messages = get_intial_test_data().await;
    if messages.is_empty() {
        println!("No messages found, generating initial test data...");
        generate_inital_test_data().await;
        messages = get_intial_test_data().await;
    }

    println!("{:?}", messages);
}

async fn get_intial_test_data() -> Json<Vec<MessageResponse>> {
    get_messages_handler(Json(MessageRequest {
        channel_id: Some(1),
        message_id_offset: Some(0),
        author_id: None,
        content: None,
        message_id: None,
    })).await
}

async fn generate_inital_test_data() {
    let fake_message_1 = MessageRequest {
        channel_id: Some(1),
        message_id: Some(1),
        author_id: Some(1),
        content: Some("Hello".to_string()),
        message_id_offset: None,
    };

    let fake_message_2 = MessageRequest {
        channel_id: Some(1),
        message_id: Some(2),
        author_id: Some(1),
        content: Some("World".to_string()),
        message_id_offset: None,
    };

    let fake_message_3 = MessageRequest {
        channel_id: Some(1),
        message_id: Some(3),
        author_id: Some(1),
        content: Some("!".to_string()),
        message_id_offset: None,
    };

    add_message_handler(Json(fake_message_1)).await;
    add_message_handler(Json(fake_message_2)).await;
    add_message_handler(Json(fake_message_3)).await;
}

async fn get_messages_handler(Json(body): Json<MessageRequest>) -> Json<Vec<MessageResponse>> {
    let session = connect_to_scylla().await.unwrap();
    let stmt = session.prepare("SELECT 
        channel_id, message_id, author_id, content 
        FROM messages.messages 
        WHERE channel_id = ? AND message_id > ? LIMIT 10").await.unwrap();
    let channel_id = body.channel_id.unwrap_or(0);
    let message_id_offset = body.message_id_offset.unwrap_or(0);

    if let Some(rows) = session
        .execute(&stmt, (channel_id, message_id_offset))
        .await
        .unwrap()
        .rows
    {
        let messages: Vec<MessageResponse> = rows
            .into_typed::<(i64, i64, i64, String)>()
            .map(|row| {
                let (channel_id, message_id, author_id, content) = row.unwrap();
                MessageResponse {
                    channel_id,
                    message_id,
                    author_id,
                    content,
                }
            })
            .collect();
        Json(messages)
    } else {
        Json(vec![])
    }
}

async fn add_message_handler(Json(body): Json<MessageRequest>) -> impl IntoResponse {
    let session = connect_to_scylla().await.unwrap();
    let stmt = session.prepare("INSERT INTO messages.messages 
        (channel_id, message_id, author_id, content) 
        VALUES (?, ?, ?, ?)").await.unwrap();
    let channel_id = body.channel_id.unwrap_or(0);
    let message_id = body.message_id.unwrap_or(0);
    let author_id = body.author_id.unwrap_or(0);
    let content = body.content.unwrap_or("".to_string());

    session
        .execute(&stmt, (channel_id, message_id, author_id, content))
        .await
        .unwrap();

    Response::builder()
        .status(StatusCode::CREATED)
        .body(Body::from("Message added successfully"))
        .unwrap()
}

async fn delete_message_handler(Json(body): Json<MessageRequest>) -> impl IntoResponse {
    let session = connect_to_scylla().await.unwrap();
    let stmt = session.prepare("DELETE FROM messages.messages 
        WHERE channel_id = ? AND message_id = ?").await.unwrap();
    let channel_id = body.channel_id.unwrap_or(0);
    let message_id = body.message_id.unwrap_or(0);

    session
        .execute(&stmt, (channel_id, message_id))
        .await
        .unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Message deleted successfully"))
        .unwrap()
}

async fn edit_message_handler(Json(body): Json<MessageRequest>) -> impl IntoResponse {
    let session = connect_to_scylla().await.unwrap();
    let stmt = session.prepare("UPDATE messages.messages 
        SET content = ? 
        WHERE channel_id = ? AND message_id = ?").await.unwrap();
    let channel_id = body.channel_id.unwrap_or(0);
    let message_id = body.message_id.unwrap_or(0);
    let content = body.content.unwrap_or("".to_string());

    session
        .execute(&stmt, (content, channel_id, message_id))
        .await
        .unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Message updated successfully"))
        .unwrap()
}

async fn connect_to_scylla() -> Result<Arc<Session>, NewSessionError> {
    let host = "172.20.0.2";
    let dc = "datacenter1";
    let usr = "scylla";
    let pwd = "scylla";
    
    println!("Connecting to {} ...", host);
    let default_policy = DefaultPolicy::builder()
        .prefer_datacenter(dc.to_string())
        .token_aware(true)
        .permit_dc_failover(false)
        .build();

    let profile = ExecutionProfile::builder()
        .load_balancing_policy(default_policy)
        .build();

    let handle = profile.into_handle();

    let session: Session = SessionBuilder::new()
        .known_node(host)
        .default_execution_profile_handle(handle)
        .compression(Some(Compression::Lz4))
        .user(usr, pwd)
        .build()
        .await?;

    

    let session = Arc::new(session);
    println!("Connected successfully! Policy: TokenAware(DCAware())");
    
    Ok(session)
}

async fn init_db() -> Result<Arc<Session>, NewSessionError> {
    let session = connect_to_scylla().await?;
    init_keyspace(session.clone()).await?;
    init_table(session.clone()).await?;
    
    Ok(session)
}

async fn init_keyspace(session: Arc<Session>) -> Result<(), QueryError> {
    let keyspace = "messages";
    let query = format!("CREATE KEYSPACE IF NOT EXISTS {} WITH REPLICATION = {{'class': 'SimpleStrategy', 'replication_factor': 1}}", keyspace);

    session.query(query, &[]).await?;
    println!("Keyspace {} created successfully!", keyspace);
    
    Ok(())
}

async fn init_table(session: Arc<Session>) -> Result<(), QueryError> {
    let table = "messages";
    let query = format!("CREATE TABLE IF NOT EXISTS messages.messages (
        channel_id BIGINT,
        message_id BIGINT,
        author_id BIGINT,
        content TEXT,
        PRIMARY KEY (channel_id, message_id)
    )
    WITH CLUSTERING ORDER BY (message_id DESC);");

    session.query(query, &[]).await?;
    println!("Table {} created successfully!", table);
    
    Ok(())
}