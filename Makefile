do d: copy build-remote
copy cp c:
	rsync -avzhe ssh --exclude target --exclude Cargo.lock --exclude .git . $(DEOXY_HOST):~/deoxy/ --delete
build-remote:
	ssh $(DEOXY_HOST) -t 'cd ~/deoxy && make build'
run-remote:
	ssh $(DEOXY_HOST) -t 'cd ~/deoxy && make run'
build b:
	cargo build
run:
	cargo run
doc docs:
	cargo doc --open
