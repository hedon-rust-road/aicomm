setup:
	cd chat/chat_server && cargo run &
	cd chat/notify_server && cargo run &
	cd chat/bot_server && cargo run &
	cd chat/analytics-server && cargo run &
	cd chatapp && cargo tauri dev
stop:
	@for PORT in 6687 6688 6689 6690; do \
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

DOCKER=docker #or podman
PWD=$(shell pwd)

build-docker:
	$(DOCKER) build -t chat-server:latest --build-arg APP_NAME=chat-server --build-arg APP_PORT=6688 -f Dockerfile .
	$(DOCKER) build -t notify-server:latest --build-arg APP_NAME=notify-server --build-arg APP_PORT=6687 -f Dockerfile .
	$(DOCKER) build -t bot-server:latest --build-arg APP_NAME=bot --build-arg APP_PORT=6689 -f Dockerfile .
	$(DOCKER) build -t analytics-server:latest --build-arg APP_NAME=analytics-server --build-arg APP_PORT=6690 -f Dockerfile .

run-docker: kill-docker
	$(DOCKER) run --entrypoint /app/chat-server --env OPENAI_API_KEY=$(OPENAI_API_KEY) --name chat -d -p 6688:6688 --mount type=bind,source=$(PWD)/fixtures/chat.yml,target=/app/chat.yml,readonly chat-server:latest
	$(DOCKER) run --entrypoint /app/notify-server --name notify -d -p 6687:6687 --mount type=bind,source=$(PWD)/fixtures/notify.yml,target=/app/notify.yml,readonly notify-server:latest
	$(DOCKER) run --entrypoint /app/bot --env OPENAI_API_KEY=$(OPENAI_API_KEY) --name bot -d -p 6686:6686 --mount type=bind,source=$(PWD)/fixtures/bot.yml,target=/app/bot.yml,readonly bot-server:latest
	$(DOCKER) run --entrypoint /app/analytics-server --name analytics -d -p 6690:6690 --mount type=bind,source=$(PWD)/fixtures/analytics.yml,target=/app/analytics.yml,readonly analytics-server:latest

kill-docker:
	-$(DOCKER) kill chat notify bot analytics || true
	-$(DOCKER) rm chat notify bot analytics || true
	docker ps