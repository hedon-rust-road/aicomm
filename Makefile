setup:
	cd chat/chat_server && cargo run &
	cd chat/notify_server && cargo run &
	cd chatapp && cargo tauri dev