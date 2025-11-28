use dioxus::prelude::*;
use todoapp_transfer::{CreateTodoRequest, Priority, TodoResponse, UpdateTodoRequest};
use tracing::{error, info};
#[cfg(not(target_arch = "wasm32"))]
use tracing_subscriber::EnvFilter;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

const API_BASE: &str = "http://localhost:3000/api";

fn main() {
    init_tracing();
    dioxus::launch(App);
}

fn init_tracing() {
    #[cfg(target_arch = "wasm32")]
    {
        use tracing_subscriber::prelude::*;

        let wasm_layer = tracing_wasm::WASMLayer::new(tracing_wasm::WASMLayerConfig::default());
        tracing_subscriber::registry().with(wasm_layer).init();
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
            )
            .with_target(false)
            .init();
    }
}

#[component]
fn App() -> Element {
    let mut todos = use_signal(|| Vec::<TodoResponse>::new());
    let mut loading = use_signal(|| true);
    let mut error_msg = use_signal(|| Option::<String>::None);

    // Load todos on mount
    use_effect(move || {
        spawn(async move {
            match fetch_todos().await {
                Ok(fetched_todos) => {
                    todos.set(fetched_todos);
                    loading.set(false);
                }
                Err(e) => {
                    error!(error = %e, "Failed to load todos");
                    error_msg.set(Some(format!("Failed to load todos: {}", e)));
                    loading.set(false);
                }
            }
        });
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        div { class: "min-h-screen bg-gray-100 py-8 px-4",
            div { class: "max-w-3xl mx-auto",
                // Header
                div { class: "text-center mb-12",
                    h1 { class: "text-5xl font-bold text-gray-900 mb-3",
                        "✓ Todo App"
                    }
                    p { class: "text-gray-600 text-lg",
                        "Organize your tasks efficiently"
                    }
                }

                if let Some(err) = error_msg() {
                    div { class: "bg-red-50 border-l-4 border-red-500 text-red-700 p-4 rounded-lg mb-6 shadow",
                        "⚠️ {err}"
                    }
                }

                AddTodoForm {
                    on_todo_added: move |_| {
                        spawn(async move {
                            if let Ok(fetched_todos) = fetch_todos().await {
                                todos.set(fetched_todos);
                            }
                        });
                    }
                }

                if loading() {
                    div { class: "text-center py-8",
                        p { class: "text-gray-600", "Loading..." }
                    }
                } else {
                    TodoList {
                        todos: todos(),
                        on_todo_changed: move |_| {
                            spawn(async move {
                                if let Ok(fetched_todos) = fetch_todos().await {
                                    todos.set(fetched_todos);
                                }
                            });
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn AddTodoForm(on_todo_added: EventHandler<()>) -> Element {
    let mut title = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut priority = use_signal(|| Priority::Medium);
    let mut submitting = use_signal(|| false);

    let on_submit = move |_| {
        if title().trim().is_empty() {
            return;
        }

        let todo_title = title();
        let todo_desc = if description().trim().is_empty() {
            None
        } else {
            Some(description())
        };
        let todo_priority = priority();

        submitting.set(true);

        spawn(async move {
            let req = CreateTodoRequest {
                title: todo_title,
                description: todo_desc,
                due_date: None,
                priority: todo_priority,
            };

            match create_todo(req).await {
                Ok(_) => {
                    title.set(String::new());
                    description.set(String::new());
                    priority.set(Priority::Medium);
                    on_todo_added.call(());
                }
                Err(e) => {
                    error!(error = %e, "Failed to create todo");
                }
            }
            submitting.set(false);
        });
    };

    rsx! {
        div { class: "bg-white rounded-2xl shadow-lg p-8 mb-8 border border-gray-100",
            h2 { class: "text-2xl font-bold text-gray-800 mb-6 flex items-center gap-2",
                span { "➕" }
                "Add New Task"
            }

            form {
                onsubmit: on_submit,
                prevent_default: "onsubmit",

                div { class: "mb-5",
                    label { class: "block text-sm font-semibold text-gray-700 mb-2", "Task Title" }
                    input {
                        r#type: "text",
                        class: "w-full px-4 py-3 border-2 border-gray-200 rounded-xl focus:outline-none focus:border-blue-500 focus:ring-4 focus:ring-blue-50 transition-all",
                        value: "{title}",
                        oninput: move |e| title.set(e.value()),
                        placeholder: "What needs to be done?"
                    }
                }

                div { class: "mb-5",
                    label { class: "block text-sm font-semibold text-gray-700 mb-2", "Description (optional)" }
                    textarea {
                        class: "w-full px-4 py-3 border-2 border-gray-200 rounded-xl focus:outline-none focus:border-blue-500 focus:ring-4 focus:ring-blue-50 transition-all resize-none",
                        value: "{description}",
                        oninput: move |e| description.set(e.value()),
                        placeholder: "Add more details...",
                        rows: "3"
                    }
                }

                div { class: "mb-6",
                    label { class: "block text-sm font-semibold text-gray-700 mb-2", "Priority Level" }
                    select {
                        class: "w-full px-4 py-3 border-2 border-gray-200 rounded-xl focus:outline-none focus:border-blue-500 focus:ring-4 focus:ring-blue-50 transition-all bg-white",
                        onchange: move |e| {
                            let p = match e.value().as_str() {
                                "Low" => Priority::Low,
                                "High" => Priority::High,
                                _ => Priority::Medium,
                            };
                            priority.set(p);
                        },
                        option { value: "Low", "🟢 Low Priority" }
                        option { value: "Medium", selected: true, "🟡 Medium Priority" }
                        option { value: "High", "🔴 High Priority" }
                    }
                }

                button {
                    r#type: "submit",
                    class: "w-full bg-blue-500 text-white font-semibold px-6 py-4 rounded-lg hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed shadow-md",
                    disabled: submitting(),
                    if submitting() { "Adding..." } else { "Add Task" }
                }
            }
        }
    }
}

#[component]
fn TodoList(todos: Vec<TodoResponse>, on_todo_changed: EventHandler<()>) -> Element {
    if todos.is_empty() {
        return rsx! {
            div { class: "bg-white rounded-2xl shadow-lg p-12 text-center border border-gray-100",
                div { class: "text-6xl mb-4", "📝" }
                h3 { class: "text-xl font-semibold text-gray-800 mb-2",
                    "No tasks yet!"
                }
                p { class: "text-gray-500",
                    "Create your first task above to get started"
                }
            }
        };
    }

    rsx! {
        div { class: "space-y-4",
            for todo in todos {
                TodoItem {
                    key: "{todo.id}",
                    todo: todo.clone(),
                    on_changed: move |_| on_todo_changed.call(())
                }
            }
        }
    }
}

#[component]
fn TodoItem(todo: TodoResponse, on_changed: EventHandler<()>) -> Element {
    let mut editing = use_signal(|| false);

    let priority_color = match todo.priority {
        Priority::High => "border-l-red-400 bg-red-50",
        Priority::Medium => "border-l-yellow-400 bg-yellow-50",
        Priority::Low => "border-l-green-400 bg-green-50",
    };

    let priority_icon = match todo.priority {
        Priority::High => "🔴",
        Priority::Medium => "🟡",
        Priority::Low => "🟢",
    };

    let priority_text = match todo.priority {
        Priority::High => "High",
        Priority::Medium => "Medium",
        Priority::Low => "Low",
    };

    let priority_badge_color = match todo.priority {
        Priority::High => "bg-red-100 text-red-700 border-red-200",
        Priority::Medium => "bg-yellow-100 text-yellow-700 border-yellow-200",
        Priority::Low => "bg-green-100 text-green-700 border-green-200",
    };

    let created_at_str = todo.created_at.format("%b %d, %Y at %H:%M").to_string();

    if editing() {
        return rsx! {
            EditTodoForm {
                todo: todo.clone(),
                on_save: move |_| {
                    editing.set(false);
                    on_changed.call(());
                },
                on_cancel: move |_| editing.set(false)
            }
        };
    }

    rsx! {
        div { class: "bg-white rounded-lg shadow-md border-l-4 {priority_color} p-6",
            div { class: "flex items-start gap-4",
                // Checkbox
                div { class: "pt-1",
                    input {
                        r#type: "checkbox",
                        class: "w-5 h-5 cursor-pointer",
                        checked: todo.completed,
                        onchange: move |_| {
                            let todo_id = todo.id;
                            let new_completed = !todo.completed;
                            spawn(async move {
                                let req = UpdateTodoRequest {
                                    title: None,
                                    description: None,
                                    due_date: None,
                                    priority: None,
                                    completed: Some(new_completed),
                                };
                                if update_todo(todo_id, req).await.is_ok() {
                                    on_changed.call(());
                                }
                            });
                        }
                    }
                }

                // Content
                div { class: "flex-1 min-w-0",
                    div { class: "flex items-center gap-3 mb-2 flex-wrap",
                        h3 {
                            class: if todo.completed { "text-2xl font-bold text-gray-400 line-through" } else { "text-2xl font-bold text-gray-900" },
                            "{todo.title}"
                        }
                        span {
                            class: "px-3 py-1 text-sm font-semibold rounded-full border-2 {priority_badge_color}",
                            "{priority_icon} {priority_text}"
                        }
                    }

                    if let Some(desc) = &todo.description {
                        p {
                            class: if todo.completed { "text-gray-400 mb-3 line-through" } else { "text-gray-700 mb-3" },
                            "{desc}"
                        }
                    }

                    div { class: "flex items-center text-sm text-gray-500",
                        span { class: "mr-1", "🕐" }
                        "{created_at_str}"
                    }
                }

                // Actions
                div { class: "flex gap-2",
                    button {
                        class: "px-3 py-2 text-sm bg-blue-500 text-white rounded hover:bg-blue-600",
                        onclick: move |_| editing.set(true),
                        "Edit"
                    }
                    button {
                        class: "px-3 py-2 text-sm bg-red-500 text-white rounded hover:bg-red-600",
                        onclick: move |_| {
                            let todo_id = todo.id;
                            spawn(async move {
                                if delete_todo(todo_id).await.is_ok() {
                                    on_changed.call(());
                                }
                            });
                        },
                        "Delete"
                    }
                }
            }
        }
    }
}

#[component]
fn EditTodoForm(
    todo: TodoResponse,
    on_save: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut title = use_signal(|| todo.title.clone());
    let mut description = use_signal(|| todo.description.clone().unwrap_or_default());
    let mut priority = use_signal(|| todo.priority.clone());
    let mut submitting = use_signal(|| false);

    let on_submit = move |_| {
        if title().trim().is_empty() {
            return;
        }

        let todo_id = todo.id;
        let new_title = title();
        let new_desc = if description().trim().is_empty() {
            None
        } else {
            Some(Some(description()))
        };
        let new_priority = priority();

        submitting.set(true);

        spawn(async move {
            let req = UpdateTodoRequest {
                title: Some(new_title),
                description: new_desc,
                due_date: None,
                priority: Some(new_priority),
                completed: None,
            };

            match update_todo(todo_id, req).await {
                Ok(_) => on_save.call(()),
                Err(e) => eprintln!("Failed to update todo: {}", e),
            }
            submitting.set(false);
        });
    };

    rsx! {
        div { class: "bg-white rounded-lg shadow-md p-6 border-2 border-blue-500",
            h3 { class: "text-xl font-bold text-gray-800 mb-4",
                "Edit Task"
            }
            form {
                onsubmit: on_submit,
                prevent_default: "onsubmit",

                div { class: "mb-4",
                    label { class: "block text-sm font-semibold text-gray-700 mb-2", "Title" }
                    input {
                        r#type: "text",
                        class: "w-full px-4 py-3 border-2 border-gray-300 rounded-xl focus:outline-none focus:border-blue-500 focus:ring-4 focus:ring-blue-100 transition-all",
                        value: "{title}",
                        oninput: move |e| title.set(e.value())
                    }
                }

                div { class: "mb-4",
                    label { class: "block text-sm font-semibold text-gray-700 mb-2", "Description" }
                    textarea {
                        class: "w-full px-4 py-3 border-2 border-gray-300 rounded-xl focus:outline-none focus:border-blue-500 focus:ring-4 focus:ring-blue-100 transition-all resize-none",
                        value: "{description}",
                        oninput: move |e| description.set(e.value()),
                        rows: "2"
                    }
                }

                div { class: "mb-5",
                    label { class: "block text-sm font-semibold text-gray-700 mb-2", "Priority" }
                    select {
                        class: "w-full px-4 py-3 border-2 border-gray-300 rounded-xl focus:outline-none focus:border-blue-500 focus:ring-4 focus:ring-blue-100 transition-all bg-white",
                        value: match priority() {
                            Priority::Low => "Low",
                            Priority::Medium => "Medium",
                            Priority::High => "High",
                        },
                        onchange: move |e| {
                            let p = match e.value().as_str() {
                                "Low" => Priority::Low,
                                "High" => Priority::High,
                                _ => Priority::Medium,
                            };
                            priority.set(p);
                        },
                        option { value: "Low", "🟢 Low" }
                        option { value: "Medium", "🟡 Medium" }
                        option { value: "High", "🔴 High" }
                    }
                }

                div { class: "flex gap-3",
                    button {
                        r#type: "submit",
                        class: "flex-1 bg-gradient-to-r from-blue-600 to-indigo-600 text-white font-semibold px-6 py-3 rounded-xl hover:from-blue-700 hover:to-indigo-700 disabled:opacity-50 transform transition-all shadow-lg",
                        disabled: submitting(),
                        if submitting() { "💾 Saving..." } else { "💾 Save" }
                    }
                    button {
                        r#type: "button",
                        class: "flex-1 bg-gray-200 text-gray-700 font-semibold px-6 py-3 rounded-xl hover:bg-gray-300 transform transition-all",
                        onclick: move |_| on_cancel.call(()),
                        "❌ Cancel"
                    }
                }
            }
        }
    }
}

// API functions

async fn fetch_todos() -> Result<Vec<TodoResponse>, String> {
    let client = reqwest::Client::new();
    info!("Fetching todos from API");
    let response = client
        .get(&format!("{}/todos", API_BASE))
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, "Request to fetch todos failed");
            e.to_string()
        })?;

    response.json::<Vec<TodoResponse>>().await.map_err(|e| {
        error!(error = %e, "Failed to deserialize todos");
        e.to_string()
    })
}

async fn create_todo(req: CreateTodoRequest) -> Result<TodoResponse, String> {
    let client = reqwest::Client::new();
    info!(title = %req.title, "Creating todo via API");
    let response = client
        .post(&format!("{}/todos", API_BASE))
        .json(&req)
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, "Request to create todo failed");
            e.to_string()
        })?;

    response.json::<TodoResponse>().await.map_err(|e| {
        error!(error = %e, "Failed to deserialize created todo");
        e.to_string()
    })
}

async fn update_todo(id: uuid::Uuid, req: UpdateTodoRequest) -> Result<TodoResponse, String> {
    let client = reqwest::Client::new();
    info!(%id, "Updating todo via API");
    let response = client
        .put(&format!("{}/todos/{}", API_BASE, id))
        .json(&req)
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, %id, "Request to update todo failed");
            e.to_string()
        })?;

    response.json::<TodoResponse>().await.map_err(|e| {
        error!(error = %e, %id, "Failed to deserialize updated todo");
        e.to_string()
    })
}

async fn delete_todo(id: uuid::Uuid) -> Result<(), String> {
    let client = reqwest::Client::new();
    info!(%id, "Deleting todo via API");
    client
        .delete(&format!("{}/todos/{}", API_BASE, id))
        .send()
        .await
        .map_err(|e| {
            error!(error = %e, %id, "Request to delete todo failed");
            e.to_string()
        })?;

    Ok(())
}
