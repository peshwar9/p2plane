mod dag;
mod message;
mod transaction;

use narwhal::p2plane::network::{Node, NodeConfig};
use crate::message::TransactionMessage;
use crate::dag::DAG;
use crate::transaction::Transaction;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use clap::Parser;
use serde::{Serialize, Deserialize};
use axum::{
    routing::post,
    Router,
    Json,
    extract::State,
    routing::get,
};
use std::net::SocketAddr;
use log::{info, error};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "8000")]
    port: u16,

    #[arg(long)]
    bootstrap: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TransactionRequest {
    data: String,
}

#[derive(Clone)]
struct ApiState {
    node: Arc<Mutex<Node<TransactionMessage>>>,
    dag: Arc<Mutex<DAG>>,
}

async fn handle_transaction(
    State(state): State<ApiState>,
    Json(req): Json<TransactionRequest>,
) -> Json<serde_json::Value> {
    info!("Received transaction request with data: {}", req.data);

    let parents = {
        let dag = state.dag.lock().await;
        let parents = dag.get_all_transactions()
            .iter()
            .take(2)
            .map(|tx| tx.id.clone())
            .collect::<Vec<_>>();
        info!("Selected parents: {:?}", parents);
        parents
    };

    let tx = Transaction::new(req.data, parents);
    info!("Created transaction with ID: {}", tx.id);
    let message = TransactionMessage::new(tx.clone());



    // First add to local DAG
    {
        let mut dag = state.dag.lock().await;
        dag.add_transaction(tx.clone());
        info!("Added transaction to local DAG");
    }

    // Return success response before broadcasting
   let response = Json(serde_json::json!({
        "status": "success",
        "transaction_id": tx.id,
        "message": "Transaction broadcast and added to DAG"
    }));

        // Then broadcast the message asynchronously
        tokio::spawn(async move {
            let mut node = state.node.lock().await;
            if let Err(e) = node.broadcast_message(message).await {
                error!("Failed to broadcast transaction: {}", e);
            } else {
                info!("Successfully broadcast transaction");
            }
        });
        
        response
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = Args::parse();

    let config = NodeConfig {
        listen_addr: format!("/ip4/127.0.0.1/tcp/{}", args.port).parse()?,
        bootstrap_addr: args.bootstrap.map(|addr| addr.parse()).transpose()?,
    };

    println!("Starting node on {}", config.listen_addr);
    if let Some(ref bootstrap) = config.bootstrap_addr {
        println!("Connecting to bootstrap node: {}", bootstrap);
    } else {
        println!("Running as bootstrap node");
    }

    let node = Arc::new(Mutex::new(Node::<TransactionMessage>::new(config).await?));
    let dag = Arc::new(Mutex::new(DAG::new()));
    
    // Setup API state
    let api_state = ApiState {
        node: node.clone(),
        dag: dag.clone(),
    };

    // Setup HTTP server
    let api_port = args.port + 1000; // API port will be node port + 1000
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/transaction", post(handle_transaction))
        .with_state(api_state);

    // Spawn HTTP server
    let addr = SocketAddr::from(([127, 0, 0, 1], api_port));
    println!("API server listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    // Start both the HTTP server and the node
    tokio::select! {
        _ = axum::serve(listener, app) => {},
        _ = async {
            let mut node = node.lock().await;
            node.start().await
        } => {},
    }

    Ok(())
}
