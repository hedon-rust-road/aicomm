setup:
	cd chat/chat_server && cargo run &
	cd chat/notify_server && cargo run &
	cd chatapp && cargo tauri dev
stop:
	@for PORT in 6687 6688; do \
		PID=$$(lsof -ti :$$PORT); \
		if [ -n "$$PID" ]; then \
			echo "Killing process on port $$PORT (PID: $$PID)"; \
			kill -9 $$PID; \
		else \
			echo "No process found on port $$PORT"; \
		fi; \
	done

restart:
	make stop
	make setup
