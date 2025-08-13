mod common;
use common::*;

async_test!(e2e_auth_signin, || async {
	let client = create_test_client();
	let auth = client.auth();

	let email = std::env::var("TEST_USER_EMAIL").ok();
	let password = std::env::var("TEST_USER_PASSWORD").ok();

	if let (Some(email), Some(password)) = (email, password) {
		match auth.sign_in_with_email_and_password(&email, &password).await {
			Ok(resp) => {
				println!("✅ Sign in OK");
				if let Some(session) = resp.session {
					println!("   access: {}..", &session.access_token[..20.min(session.access_token.len())]);
				}
			}
			Err(e) => println!("❌ Sign in failed: {}", e),
		}
	} else {
		println!("⚠️ TEST_USER_EMAIL/TEST_USER_PASSWORD not provided; skipping sign-in");
	}

	println!("   is_authenticated: {}", auth.is_authenticated());
});
