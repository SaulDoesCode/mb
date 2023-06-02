use actix_web::{get, post, delete, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use rhyzome_heed::{Rhyzome, Relation, Node};

#[derive(Debug, Serialize, Deserialize)]
struct Post {
    id: String,
    content: String,
    zone: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreatePostRequest {
    content: String,
    zone: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RelationQuery {
    relation_name: String,
    from_node_id: String,
    to_node_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RelationQueryResponse {
    relation_name: String,
    relations: Vec<Relation>,
}

struct TokenManager {
    tokens_rhyzome: Rhyzome,
    admin_password: String,
}

impl TokenManager {
    fn new(tokens_rhyzome: Rhyzome, admin_password: String) -> Self {
        Self {
            tokens_rhyzome,
            admin_password,
        }
    }

    fn generate_token(&self, permission: &str) -> Result<String, Box<dyn std::error::Error>> {
        let token = generate_token_id();
        self.tokens_rhyzome.store_node(&Node::new(token.clone(), permission.into()))?;
        Ok(token)
    }

    fn validate_token(
        &self,
        token: &str,
        required_permission: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let node = self.tokens_rhyzome.retrieve_node(token)?;
        match node {
            Some(node) if node.content == required_permission => {
                self.tokens_rhyzome.delete_node(token)?;
                Ok(())
            }
            _ => Err("Invalid token or insufficient permissions".into()),
        }
    }
}

fn generate_token_id() -> String {
    // Generate a unique token ID (you can use any suitable method here)
    // For simplicity, we're using a random 8-character alphanumeric string
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    thread_rng().sample_iter(&Alphanumeric).take(8).collect()
}

#[post("/posts")]
async fn create_post(
    payload: web::Json<CreatePostRequest>,
    rhyzome: web::Data<Rhyzome>,
    token_manager: web::Data<TokenManager>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    let authorization_header = req.headers().get("Authorization");
    let token = match authorization_header {
        Some(header_value) => {
            let header_str = header_value.to_str().unwrap_or("");
            // Extract the token from the header (e.g., "Bearer TOKEN_VALUE")
            let token_parts: Vec<&str> = header_str.split_whitespace().collect();
            if token_parts.len() == 2 {
                token_parts[1].to_owned()
            } else {
                return HttpResponse::Unauthorized().body("Unauthorized");
            }
        }
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    let permission = "create";

    // Validate the token and required permission
    match token_manager.validate_token(&token, permission) {
        Ok(()) => {
            let post_id = generate_token_id();
            let post = Post {
                id: post_id.clone(),
                content: payload.content.clone(),
                zone: payload.zone.clone(),
            };
            rhyzome
                .store_node(&Node::new(post_id, serde_json::to_vec(&post).unwrap()))
                .unwrap();
            HttpResponse::Ok().body("Post created successfully")
        }
        Err(e) => {
            eprintln!("Failed to validate token: {:?}", e);
            HttpResponse::Unauthorized().body("Unauthorized")
        }
    }
}

#[get("/posts/{id}")]
async fn get_post(
    web::Path(id): web::Path<String>,
    rhyzome: web::Data<Rhyzome>,
    token_manager: web::Data<TokenManager>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    let authorization_header = req.headers().get("Authorization");
    let token = match authorization_header {
        Some(header_value) => {
            let header_str = header_value.to_str().unwrap_or("");
            // Extract the token from the header (e.g., "Bearer TOKEN_VALUE")
            let token_parts: Vec<&str> = header_str.split_whitespace().collect();
            if token_parts.len() == 2 {
                token_parts[1].to_owned()
            } else {
                return HttpResponse::Unauthorized().body("Unauthorized");
            }
        }
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    let permission = "edit";

    // Validate the token and required permission
    match token_manager.validate_token(&token, permission) {
        Ok(()) => {
            match rhyzome.retrieve_node(&id) {
                Ok(Some(node)) => {
                    let post: Post = serde_json::from_slice(&node.content).unwrap();
                    HttpResponse::Ok().json(post)
                }
                Ok(None) => HttpResponse::NotFound().body("Post not found"),
                Err(e) => {
                    eprintln!("Failed to retrieve post: {:?}", e);
                    HttpResponse::InternalServerError().body("Failed to retrieve post")
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to validate token: {:?}", e);
            HttpResponse::Unauthorized().body("Unauthorized")
        }
    }
}

#[delete("/posts/{id}")]
async fn delete_post(
    web::Path(id): web::Path<String>,
    rhyzome: web::Data<Rhyzome>,
    token_manager: web::Data<TokenManager>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    let authorization_header = req.headers().get("Authorization");
    let token = match authorization_header {
        Some(header_value) => {
            let header_str = header_value.to_str().unwrap_or("");
            // Extract the token from the header (e.g., "Bearer TOKEN_VALUE")
            let token_parts: Vec<&str> = header_str.split_whitespace().collect();
            if token_parts.len() == 2 {
                token_parts[1].to_owned()
            } else {
                return HttpResponse::Unauthorized().body("Unauthorized");
            }
        }
        None => return HttpResponse::Unauthorized().body("Unauthorized"),
    };

    let permission = "edit";

    // Validate the token and required permission
    match token_manager.validate_token(&token, permission) {
        Ok(()) => {
            match rhyzome.delete_node(&id) {
                Ok(true) => HttpResponse::Ok().body("Post deleted successfully"),
                Ok(false) => HttpResponse::NotFound().body("Post not found"),
                Err(e) => {
                    eprintln!("Failed to delete post: {:?}", e);
                    HttpResponse::InternalServerError().body("Failed to delete post")
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to validate token: {:?}", e);
            HttpResponse::Unauthorized().body("Unauthorized")
        }
    }
}

#[post("/relations")]
async fn query_relations(
    payload: web::Json<RelationQuery>,
    rhyzome: web::Data<Rhyzome>,
) -> impl Responder {
    let relation_name = payload.relation_name.clone();
    let from_node_id = payload.from_node_id.clone();
    let to_node_id = payload.to_node_id.clone();

    match rhyzome.query_relations(&relation_name, &from_node_id, &to_node_id) {
        Ok(relations) => {
            let response = RelationQueryResponse {
                relation_name,
                relations,
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            eprintln!("Failed to query relations: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to query relations")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create a new Rhyzome instance using Heed for storing posts
    let rhyzome = Rhyzome::new("./rhyzome.heed").unwrap();

    // Create a separate Rhyzome instance for storing tokens
    let tokens_rhyzome = Rhyzome::new("./tokens-rhyzome.heed").unwrap();

    // Initialize token manager
    let token_manager = TokenManager::new(tokens_rhyzome, "admin_password123".to_owned());

    HttpServer::new(move || {
        App::new()
            .data(rhyzome.clone())
            .data(token_manager.clone())
            .service(create_post)
            .service(get_post)
            .service(delete_post)
            .service(query_relations)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
