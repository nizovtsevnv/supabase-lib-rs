//! Dioxus integration example for Supabase client
//!
//! This example demonstrates how to use the Supabase client within a Dioxus web application.
//! It shows authentication, database operations, and state management patterns.
//!
//! To run this example:
//! ```bash
//! # Add to Cargo.toml dev-dependencies:
//! # dioxus = "0.6"
//! # dioxus-web = "0.6"
//! #
//! # Then build for web:
//! # dx build --release --target web
//! ```

use serde::{Deserialize, Serialize};
// Remove unused import

#[cfg(target_arch = "wasm32")]
use dioxus::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Todo {
    id: Option<i32>,
    title: String,
    completed: bool,
    created_at: Option<String>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
struct SupabaseService {
    client: Client,
}

#[cfg(target_arch = "wasm32")]
impl SupabaseService {
    fn new(url: &str, key: &str) -> supabase::Result<Self> {
        let client = supabase::Client::new(url, key)?;
        Ok(Self { client })
    }

    async fn sign_in(&self, email: &str, password: &str) -> supabase::Result<()> {
        self.client
            .auth()
            .sign_in_with_email_and_password(email, password)
            .await?;
        Ok(())
    }

    async fn sign_out(&self) -> supabase::Result<()> {
        self.client.auth().sign_out().await
    }

    async fn get_todos(&self) -> supabase::Result<Vec<Todo>> {
        self.client
            .database()
            .from("todos")
            .select("*")
            .order("created_at", OrderDirection::Descending)
            .execute::<Todo>()
            .await
    }

    async fn create_todo(&self, title: &str) -> supabase::Result<Todo> {
        let new_todo = Todo {
            id: None,
            title: title.to_string(),
            completed: false,
            created_at: None,
        };

        let result = self
            .client
            .database()
            .insert("todos")
            .values(new_todo)?
            .returning("*")
            .execute::<Todo>()
            .await?;

        result
            .into_iter()
            .next()
            .ok_or_else(|| supabase::Error::database("No todo returned from insert"))
    }

    async fn update_todo(&self, id: i32, completed: bool) -> supabase::Result<()> {
        self.client
            .database()
            .update("todos")
            .set("completed", completed)?
            .eq("id", &id.to_string())
            .execute::<serde_json::Value>()
            .await?;
        Ok(())
    }

    async fn delete_todo(&self, id: i32) -> supabase::Result<()> {
        self.client
            .database()
            .delete("todos")
            .eq("id", &id.to_string())
            .execute()
            .await?;
        Ok(())
    }

    async fn subscribe_to_todos(&self) -> supabase::Result<String> {
        self.client.realtime().connect().await?;

        self.client
            .realtime()
            .channel("todos")
            .table("todos")
            .subscribe(|message| {
                web_sys::console::log_1(&format!("Realtime update: {:?}", message).into());
            })
            .await
    }
}

#[cfg(target_arch = "wasm32")]
#[component]
fn App() -> Element {
    // Supabase service
    let supabase = use_signal(|| -> Option<SupabaseService> {
        match SupabaseService::new("https://your-project.supabase.co", "your-anon-key") {
            Ok(service) => Some(service),
            Err(e) => {
                web_sys::console::log_1(&format!("Failed to initialize Supabase: {}", e).into());
                None
            }
        }
    });

    // App state
    let authenticated = use_signal(|| false);
    let todos = use_signal(|| Vec::<Todo>::new());
    let loading = use_signal(|| false);
    let error_message = use_signal(|| String::new());

    // Form state
    let email = use_signal(|| String::new());
    let password = use_signal(|| String::new());
    let new_todo_title = use_signal(|| String::new());

    // Load todos when authenticated
    use_effect(move || {
        if authenticated() && supabase().is_some() {
            let service = supabase().unwrap();
            spawn(async move {
                loading.set(true);
                match service.get_todos().await {
                    Ok(fetched_todos) => {
                        todos.set(fetched_todos);
                        error_message.set(String::new());
                    }
                    Err(e) => {
                        error_message.set(format!("Failed to load todos: {}", e));
                    }
                }
                loading.set(false);
            });
        }
    });

    // Sign in handler
    let handle_sign_in = move |_| {
        if let Some(service) = supabase() {
            let email_val = email();
            let password_val = password();
            spawn(async move {
                loading.set(true);
                match service.sign_in(&email_val, &password_val).await {
                    Ok(_) => {
                        authenticated.set(true);
                        error_message.set(String::new());

                        // Subscribe to realtime updates
                        if let Err(e) = service.subscribe_to_todos().await {
                            web_sys::console::log_1(
                                &format!("Failed to subscribe to realtime: {}", e).into(),
                            );
                        }
                    }
                    Err(e) => {
                        error_message.set(format!("Sign in failed: {}", e));
                    }
                }
                loading.set(false);
            });
        }
    };

    // Sign out handler
    let handle_sign_out = move |_| {
        if let Some(service) = supabase() {
            spawn(async move {
                if let Err(e) = service.sign_out().await {
                    error_message.set(format!("Sign out failed: {}", e));
                } else {
                    authenticated.set(false);
                    todos.set(Vec::new());
                }
            });
        }
    };

    // Create todo handler
    let handle_create_todo = move |_| {
        if let Some(service) = supabase() {
            let title = new_todo_title();
            if !title.is_empty() {
                spawn(async move {
                    match service.create_todo(&title).await {
                        Ok(new_todo) => {
                            let mut current_todos = todos();
                            current_todos.insert(0, new_todo);
                            todos.set(current_todos);
                            new_todo_title.set(String::new());
                            error_message.set(String::new());
                        }
                        Err(e) => {
                            error_message.set(format!("Failed to create todo: {}", e));
                        }
                    }
                });
            }
        }
    };

    // Toggle todo completion
    let handle_toggle_todo = move |id: i32, completed: bool| {
        if let Some(service) = supabase() {
            spawn(async move {
                match service.update_todo(id, !completed).await {
                    Ok(_) => {
                        let mut current_todos = todos();
                        if let Some(todo) = current_todos.iter_mut().find(|t| t.id == Some(id)) {
                            todo.completed = !completed;
                        }
                        todos.set(current_todos);
                        error_message.set(String::new());
                    }
                    Err(e) => {
                        error_message.set(format!("Failed to update todo: {}", e));
                    }
                }
            });
        }
    };

    // Delete todo
    let handle_delete_todo = move |id: i32| {
        if let Some(service) = supabase() {
            spawn(async move {
                match service.delete_todo(id).await {
                    Ok(_) => {
                        let current_todos = todos();
                        let filtered_todos: Vec<Todo> = current_todos
                            .into_iter()
                            .filter(|t| t.id != Some(id))
                            .collect();
                        todos.set(filtered_todos);
                        error_message.set(String::new());
                    }
                    Err(e) => {
                        error_message.set(format!("Failed to delete todo: {}", e));
                    }
                }
            });
        }
    };

    rsx! {
        div { class: "container mx-auto p-6 max-w-md",
            h1 { class: "text-3xl font-bold mb-6 text-center",
                "Supabase + Dioxus Todo App"
            }

            // Error display
            if !error_message().is_empty() {
                div { class: "bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4",
                    {error_message()}
                }
            }

            // Loading indicator
            if loading() {
                div { class: "text-center mb-4",
                    "Loading..."
                }
            }

            // Authentication section
            if !authenticated() {
                div { class: "mb-6 p-4 border rounded",
                    h2 { class: "text-xl font-semibold mb-4", "Sign In" }

                    input {
                        class: "w-full p-2 mb-2 border rounded",
                        r#type: "email",
                        placeholder: "Email",
                        value: "{email()}",
                        oninput: move |e| email.set(e.value())
                    }

                    input {
                        class: "w-full p-2 mb-4 border rounded",
                        r#type: "password",
                        placeholder: "Password",
                        value: "{password()}",
                        oninput: move |e| password.set(e.value())
                    }

                    button {
                        class: "w-full bg-blue-500 text-white p-2 rounded hover:bg-blue-600",
                        onclick: handle_sign_in,
                        disabled: loading(),
                        "Sign In"
                    }
                }
            } else {
                // Authenticated content
                div {
                    div { class: "flex justify-between items-center mb-6",
                        h2 { class: "text-xl font-semibold", "Your Todos" }

                        button {
                            class: "bg-red-500 text-white px-4 py-2 rounded hover:bg-red-600",
                            onclick: handle_sign_out,
                            "Sign Out"
                        }
                    }

                    // Add new todo
                    div { class: "mb-6 flex gap-2",
                        input {
                            class: "flex-1 p-2 border rounded",
                            placeholder: "New todo...",
                            value: "{new_todo_title()}",
                            oninput: move |e| new_todo_title.set(e.value()),
                            onkeypress: move |e| {
                                if e.key() == "Enter" {
                                    handle_create_todo(())
                                }
                            }
                        }

                        button {
                            class: "bg-green-500 text-white px-4 py-2 rounded hover:bg-green-600",
                            onclick: handle_create_todo,
                            disabled: new_todo_title().is_empty(),
                            "Add"
                        }
                    }

                    // Todo list
                    div { class: "space-y-2",
                        for todo in todos() {
                            div {
                                key: "{todo.id.unwrap_or(0)}",
                                class: "flex items-center gap-3 p-3 border rounded hover:bg-gray-50",

                                input {
                                    r#type: "checkbox",
                                    checked: todo.completed,
                                    onchange: move |_| {
                                        if let Some(id) = todo.id {
                                            handle_toggle_todo(id, todo.completed);
                                        }
                                    }
                                }

                                span {
                                    class: if todo.completed { "flex-1 line-through text-gray-500" } else { "flex-1" },
                                    "{todo.title}"
                                }

                                button {
                                    class: "text-red-500 hover:text-red-700",
                                    onclick: move |_| {
                                        if let Some(id) = todo.id {
                                            handle_delete_todo(id);
                                        }
                                    },
                                    "Delete"
                                }
                            }
                        }

                        if todos().is_empty() {
                            div { class: "text-center text-gray-500 py-8",
                                "No todos yet. Add one above!"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    dioxus::launch(App);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("This example is designed for WASM with Dioxus. Use 'dx build --target web'");
}
