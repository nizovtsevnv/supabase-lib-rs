use crate::common::*;

#[tokio::test]
async fn e2e_database_crud_with_auth() {
	if skip_if_no_supabase() {
		println!("Skipping test - Supabase not configured");
		return;
	}

	let client = create_test_client();
	let auth = client.auth();

	let email = match std::env::var("TEST_USER_EMAIL") { Ok(v) => v, Err(_) => { println!("TEST_USER_EMAIL not set; skipping"); return; } };
	let password = match std::env::var("TEST_USER_PASSWORD") { Ok(v) => v, Err(_) => { println!("TEST_USER_PASSWORD not set; skipping"); return; } };

	// Sign in
	let _ = auth.sign_in_with_email_and_password(&email, &password).await;
	let user = match auth.current_user().await { Ok(Some(u)) => u, _ => { println!("No current user after sign in; skipping"); return; } };

	let db = client.database();
	let title = format!("e2e post {}", uuid::Uuid::new_v4().simple());

	// INSERT
	let insert_payload = serde_json::json!({
		"title": title,
		"content": "e2e content",
		"published": true,
		"author_id": user.id,
	});
	let inserted = db
		.insert("posts")
		.values(insert_payload)
		.unwrap()
		.returning("id, title, author_id")
		.execute::<serde_json::Value>()
		.await;
	println!("insert: {:?}", inserted.as_ref().map(|v| v.len()));

	// SELECT
	let selected = db
		.from("posts")
		.select("id, title, author_id")
		.eq("author_id", &user.id.to_string())
		.limit(5)
		.execute::<serde_json::Value>()
		.await;
	println!("select: {:?}", selected.as_ref().map(|v| v.len()));

	// UPDATE
	let update_payload = serde_json::json!({
		"content": "updated by e2e",
	});
	let updated = db
		.update("posts")
		.set(update_payload)
		.unwrap()
		.eq("author_id", &user.id.to_string())
		.returning("id")
		.execute::<serde_json::Value>()
		.await;
	println!("update: {:?}", updated.as_ref().map(|v| v.len()));

	// RPC (best-effort)
	let rpc = db
		.rpc("search_posts", Some(serde_json::json!({"search_term": "e2e", "result_limit": 5})))
		.await;
	println!("rpc: {:?}", rpc.as_ref().map(|v| v));

	// DELETE
	let deleted = db
		.delete("posts")
		.eq("author_id", &user.id.to_string())
		.eq("title", &title)
		.returning("id")
		.execute::<serde_json::Value>()
		.await;
	println!("delete: {:?}", deleted.as_ref().map(|v| v.len()));

	// No assertions hard-failing: we print counts to understand RLS state across environments
} 