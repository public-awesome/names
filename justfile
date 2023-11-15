e2e-test:
	#!/usr/bin/env bash
	START_DIR=$(pwd)
	cd typescript/packages/e2e-tests
	yarn install
	yarn test

deploy-local:
	#!/usr/bin/env bash
	TEST_ADDRS=`jq -r '.[].address' ./typescript/packages/e2e-tests/configs/test_accounts.json | tr '\n' ' '`
	docker kill stargaze || true
	docker volume rm -f stargaze_data
	docker run --rm -d --name stargaze \
		-e DENOM=ustars \
		-e CHAINID=testing \
		-e GAS_LIMIT=75000000 \
		-p 1317:1317 \
		-p 26656:26656 \
		-p 26657:26657 \
		-p 9090:9090 \
		--mount type=volume,source=stargaze_data,target=/root \
		publicawesome/stargaze:12.0.0-alpha.1 /data/entry-point.sh $TEST_ADDRS

deploy-local-arm:
	#!/usr/bin/env bash
	TEST_ADDRS=`jq -r '.[].address' ./typescript/packages/e2e-tests/configs/test_accounts.json | tr '\n' ' '`
	docker kill stargaze || true
	docker volume rm -f stargaze_data
	docker run --rm -d --name stargaze \
		-e DENOM=ustars \
		-e CHAINID=testing \
		-e GAS_LIMIT=75000000 \
		-p 1317:1317 \
		-p 26656:26656 \
		-p 26657:26657 \
		-p 9090:9090 \
		--mount type=volume,source=stargaze_data,target=/root \
		--platform linux/amd64 \
		publicawesome/stargaze:12.0.0-alpha.1 /data/entry-point.sh $TEST_ADDRS

artifacts:
    mkdir -p artifacts

clear-artifacts:
    rm -rf artifacts
    mkdir -p artifacts

optimize:
    sh scripts/optimize.sh

