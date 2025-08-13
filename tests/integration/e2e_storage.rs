use crate::common::*;

async_test!(e2e_storage_flow, || async {
	let config = TestConfig::from_env();
	let client = create_test_client();
	let storage = client.storage();

	let bucket = std::env::var("TEST_BUCKET").unwrap_or_else(|_| "avatars".to_string());
	println!("bucket: {}", bucket);

	// Try create bucket with admin client if service role is available via a new client
	if let Ok(sr) = std::env::var("SUPABASE_SERVICE_ROLE_KEY") {
		let admin = supabase::Client::new_with_service_role(&config.url, &config.key, &sr).expect("admin client");
		let admin_storage = admin.storage();
		let _ = admin_storage.create_bucket(&bucket, &bucket, true).await;
	}

	let bytes = bytes::Bytes::from_static(b"hello world");
	let key = "test/hello.txt";

	let _ = storage.upload(&bucket, key, bytes.clone(), Some(supabase::storage::FileOptions{
		content_type: Some("text/plain".to_string()),
		cache_control: None,
		upsert: true,
	})).await;

	let _ = storage.list(&bucket, Some("test/")).await;
	let _ = storage.download(&bucket, key).await;
	let _ = storage.create_signed_url(&bucket, key, 3600).await;
});
