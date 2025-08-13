mod common;
use common::*;

async_test!(e2e_preflight_health_and_version, || async {
	let client = create_test_client();

	match client.health_check().await {
		Ok(true) => println!("✅ Health check OK"),
		Ok(false) => println!("⚠️ Health check returned non-success status"),
		Err(e) => println!("❌ Health check error: {}", e),
	}

	match client.version().await {
		Ok(info) => {
			println!("✅ Version endpoint OK ({} keys)", info.len());
			if let Some(v) = info.get("info") {
				println!("   info: {}", v);
			}
		}
		Err(e) => println!("❌ Version endpoint error: {}", e),
	}
});
