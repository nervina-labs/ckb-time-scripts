ENVIRONMENT := debug

all: 
	capsule build

simulators: simulator/natives-index-state simulator/natives-info
	mkdir -p build/$(ENVIRONMENT)
	cp target/$(ENVIRONMENT)/ckb-time-index-state-type-sim build/$(ENVIRONMENT)/ckb-time-index-state-type-sim
	cp target/$(ENVIRONMENT)/ckb-time-info-type-sim build/$(ENVIRONMENT)/ckb-time-info-type-sim

simulator/natives-index-state:
	CARGO_INCREMENTAL=0 RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort" RUSTDOCFLAGS="-Cpanic=abort" cargo build -p natives-index-state

simulator/natives-info:
	CARGO_INCREMENTAL=0 RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort" RUSTDOCFLAGS="-Cpanic=abort" cargo build -p natives-info

test: simulators
	cargo test -p tests
	./scripts/run_sim_tests.sh $(ENVIRONMENT)

coverage: test
	zip -0 build/$(ENVIRONMENT)/ccov.zip `find . \( -name "ckb-time-scripts-sim*.gc*" \) -print`
	grcov build/$(ENVIRONMENT)/ccov.zip -s . -t lcov --llvm --branch --ignore-not-existing --ignore "/*" -o build/$(ENVIRONMENT)/lcov.info
	genhtml -o build/$(ENVIRONMENT)/coverage/ --rc lcov_branch_coverage=1 --show-details --highlight --ignore-errors source --legend build/$(ENVIRONMENT)/lcov.info

clean:	
	cargo clean
	rm -rf build/$(ENVIRONMENT)

.PHONY: all simulators test coverage clean