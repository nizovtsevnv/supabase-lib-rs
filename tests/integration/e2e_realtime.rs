use crate::common::*;

async_test!(e2e_realtime_structural, || async {
	let client = create_test_client();
	let realtime = client.realtime();

	let _ = realtime.connect().await;
	let _ = realtime.channel("posts");
	let _ = realtime.disconnect().await;
});
