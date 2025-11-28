use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use std::sync::Arc;
use todoapp_model::{Priority as ModelPriority, Todo, TodoDb};
use todoapp_transfer::{
    CreateTodoRequest, ErrorResponse, Priority, TodoResponse, UpdateTodoRequest,
};
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db: Arc<TodoDb>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer())
        .init();

    info!("Starting todoapp backend");

    // Initialize database
    let db = TodoDb::new("./data").expect("Failed to open database");
    let state = AppState { db: Arc::new(db) };

    // Build API router
    let api_router = Router::new()
        .route("/todos", get(list_todos))
        .route("/todos", post(create_todo))
        .route("/todos/{id}", get(get_todo))
        .route("/todos/{id}", put(update_todo))
        .route("/todos/{id}", delete(delete_todo))
        .with_state(state);

    // Build main router with CORS and static file serving
    let app = Router::new()
        .nest("/api", api_router)
        .fallback_service(ServeDir::new("crates/todoapp-frontend/dist"))
        .layer(
            CorsLayer::permissive()
                .allow_origin("http://localhost:8080".parse::<HeaderValue>().unwrap()),
        )
        .layer(TraceLayer::new_for_http());

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to port 3000");

    info!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

// Handlers

async fn list_todos(State(state): State<AppState>) -> Result<Json<Vec<TodoResponse>>, AppError> {
    info!("Listing todos");
    let todos = state.db.get_all()?;
    let responses: Vec<TodoResponse> = todos.into_iter().map(todo_to_response).collect();
    Ok(Json(responses))
}

async fn create_todo(
    State(state): State<AppState>,
    Json(req): Json<CreateTodoRequest>,
) -> Result<(StatusCode, Json<TodoResponse>), AppError> {
    info!(title = %req.title, "Creating todo");
    let todo = Todo::new(
        req.title,
        req.description,
        req.due_date,
        priority_to_model(req.priority),
    );
    state.db.insert(&todo)?;
    Ok((StatusCode::CREATED, Json(todo_to_response(todo))))
}

async fn get_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TodoResponse>, AppError> {
    info!(%id, "Fetching todo");
    let todo = state
        .db
        .get(&id)?
        .ok_or_else(|| AppError::NotFound(format!("Todo with id {} not found", id)))?;
    Ok(Json(todo_to_response(todo)))
}

async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTodoRequest>,
) -> Result<Json<TodoResponse>, AppError> {
    info!(%id, "Updating todo");
    let mut todo = state
        .db
        .get(&id)?
        .ok_or_else(|| AppError::NotFound(format!("Todo with id {} not found", id)))?;

    // Update fields
    todo.update(
        req.title,
        req.description,
        req.due_date,
        req.priority.map(priority_to_model),
    );

    // Handle completed status separately
    if let Some(completed) = req.completed {
        if completed {
            todo.mark_completed();
        } else {
            todo.mark_incomplete();
        }
    }

    state.db.update(&todo)?;
    Ok(Json(todo_to_response(todo)))
}

async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    info!(%id, "Deleting todo");
    let existed = state.db.delete(&id)?;
    if existed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound(format!("Todo with id {} not found", id)))
    }
}

// Helper functions

fn todo_to_response(todo: Todo) -> TodoResponse {
    TodoResponse {
        id: todo.id,
        title: todo.title,
        description: todo.description,
        due_date: todo.due_date,
        priority: model_priority_to_transfer(todo.priority),
        completed: todo.completed,
        created_at: todo.created_at,
        updated_at: todo.updated_at,
    }
}

fn priority_to_model(priority: Priority) -> ModelPriority {
    match priority {
        Priority::Low => ModelPriority::Low,
        Priority::Medium => ModelPriority::Medium,
        Priority::High => ModelPriority::High,
    }
}

fn model_priority_to_transfer(priority: ModelPriority) -> Priority {
    match priority {
        ModelPriority::Low => Priority::Low,
        ModelPriority::Medium => Priority::Medium,
        ModelPriority::High => Priority::High,
    }
}

// Error handling

enum AppError {
    DatabaseError(anyhow::Error),
    NotFound(String),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::DatabaseError(err) => {
                error!(error = %err, "database error while handling request");
                (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            }
            AppError::NotFound(msg) => {
                error!(message = %msg, "resource not found");
                (StatusCode::NOT_FOUND, msg)
            }
        };

        (status, Json(ErrorResponse::new(message))).into_response()
    }
}
