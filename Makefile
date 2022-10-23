.PHONY: lint types

lint:
	cargo clippy --all-targets -- -D warnings

types:
	cd contracts/marketplace && cargo schema && cd ../..
	cd contracts/name-minter && cargo schema && cd ../..
	cd contracts/sg721-name && cargo schema && cd ../..
	cd contracts/whitelist-updatable && cargo schema && cd ../..
	cd ts && yarn codegen && cd ..
