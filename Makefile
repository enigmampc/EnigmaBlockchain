PACKAGES=$(shell go list ./... | grep -v '/simulation')
VERSION := $(shell echo $(shell git describe --tags) | sed 's/^v//')
COMMIT := $(shell git log -1 --format='%H')
LEDGER_ENABLED ?= true
BINDIR ?= $(GOPATH)/bin

SGX_MODE ?= HW
BRANCH ?= develop
DEBUG ?= 0
DOCKER_TAG ?= latest

ifeq ($(SGX_MODE), HW)
	ext := hw
else ifeq ($(SGX_MODE), SW)
	ext := sw
else
$(error SGX_MODE must be either HW or SW)
endif

build_tags = netgo
ifeq ($(LEDGER_ENABLED),true)
  ifeq ($(OS),Windows_NT)
    GCCEXE = $(shell where gcc.exe 2> NUL)
    ifeq ($(GCCEXE),)
      $(error "gcc.exe not installed for ledger support, please install or set LEDGER_ENABLED=false")
    else
      build_tags += ledger
    endif
  else
    UNAME_S = $(shell uname -s)
    ifeq ($(UNAME_S),OpenBSD)
      $(warning "OpenBSD detected, disabling ledger support (https://github.com/cosmos/cosmos-sdk/issues/1988)")
    else
      GCC = $(shell command -v gcc 2> /dev/null)
      ifeq ($(GCC),)
        $(error "gcc not installed for ledger support, please install or set LEDGER_ENABLED=false")
      else
        build_tags += ledger
      endif
    endif
  endif
endif

ifeq ($(WITH_CLEVELDB),yes)
  build_tags += gcc
endif
build_tags += $(BUILD_TAGS)
build_tags := $(strip $(build_tags))

whitespace :=
whitespace += $(whitespace)
comma := ,
build_tags_comma_sep := $(subst $(whitespace),$(comma),$(build_tags))

ldflags = -X github.com/cosmos/cosmos-sdk/version.Name=EnigmaBlockchain \
	-X github.com/cosmos/cosmos-sdk/version.ServerName=enigmad \
	-X github.com/cosmos/cosmos-sdk/version.ClientName=enigmacli \
	-X github.com/cosmos/cosmos-sdk/version.Version=$(VERSION) \
	-X github.com/cosmos/cosmos-sdk/version.Commit=$(COMMIT) \
	-X "github.com/cosmos/cosmos-sdk/version.BuildTags=$(build_tags)"

ifeq ($(WITH_CLEVELDB),yes)
  ldflags += -X github.com/cosmos/cosmos-sdk/types.DBBackend=cleveldb
endif
ldflags += -s -w
ldflags += $(LDFLAGS)
ldflags := $(strip $(ldflags))

BUILD_FLAGS := -tags "$(build_tags)" -ldflags '$(ldflags)'

all: build_all

vendor:
	cargo vendor third_party/vendor --manifest-path third_party/build/Cargo.toml

go.sum: go.mod
	@echo "--> Ensure dependencies have not been modified"
	GO111MODULE=on go mod verify

xgo_build_enigmad: go.sum
	xgo --go latest --targets $(XGO_TARGET) $(BUILD_FLAGS) github.com/enigmampc/EnigmaBlockchain/cmd/enigmad

xgo_build_enigmacli: go.sum
	xgo --go latest --targets $(XGO_TARGET) $(BUILD_FLAGS) github.com/enigmampc/EnigmaBlockchain/cmd/enigmacli

build_local_no_rust:
	cp go-cosmwasm/target/release/libgo_cosmwasm.so go-cosmwasm/api
#   this pulls out ELF symbols, 80% size reduction!
	go build -mod=readonly $(BUILD_FLAGS) ./cmd/enigmad
	go build -mod=readonly $(BUILD_FLAGS) ./cmd/enigmacli

build_linux: vendor
	$(MAKE) -C go-cosmwasm build-rust
	cp go-cosmwasm/target/release/libgo_cosmwasm.so go-cosmwasm/api
#   this pulls out ELF symbols, 80% size reduction!
	go build -mod=readonly $(BUILD_FLAGS) ./cmd/enigmad
	go build -mod=readonly $(BUILD_FLAGS) ./cmd/enigmacli

#build_local_no_rust:
#   this pulls out ELF symbols, 80% size reduction!
#	go build -mod=readonly $(BUILD_FLAGS) ./cmd/enigmad
#	go build -mod=readonly $(BUILD_FLAGS) ./cmd/enigmacli

build_windows:
	# CLI only 
	$(MAKE) xgo_build_enigmacli XGO_TARGET=windows/amd64

build_macos:
	# CLI only 
	$(MAKE) xgo_build_enigmacli XGO_TARGET=darwin/amd64

build_arm_linux:
	# CLI only 
	$(MAKE) xgo_build_enigmacli XGO_TARGET=linux/arm64

build_all: build_linux build_windows build_macos build_arm_linux

deb: build_linux
    ifneq ($(UNAME_S),Linux)
		exit 1
    endif
	rm -rf /tmp/EnigmaBlockchain

	mkdir -p /tmp/EnigmaBlockchain/deb/usr/local/bin
	mv -f ./enigmacli /tmp/EnigmaBlockchain/deb/usr/local/bin/enigmacli
	mv -f ./enigmad /tmp/EnigmaBlockchain/deb/usr/local/bin/enigmad
	chmod +x /tmp/EnigmaBlockchain/deb/usr/local/bin/enigmad /tmp/EnigmaBlockchain/deb/usr/local/bin/enigmacli

	mkdir -p /tmp/EnigmaBlockchain/deb/usr/local/lib
	cp -f ./go-cosmwasm/api/libgo_cosmwasm.so ./go-cosmwasm/librust_cosmwasm_enclave.signed.so /tmp/EnigmaBlockchain/deb/usr/local/lib/
	chmod +x /tmp/EnigmaBlockchain/deb/usr/local/lib/lib*.so

	mkdir -p /tmp/EnigmaBlockchain/deb/DEBIAN
	cp ./packaging_ubuntu/control /tmp/EnigmaBlockchain/deb/DEBIAN/control
	printf "Version: " >> /tmp/EnigmaBlockchain/deb/DEBIAN/control
	git describe --tags | tr -d v >> /tmp/EnigmaBlockchain/deb/DEBIAN/control
	echo "" >> /tmp/EnigmaBlockchain/deb/DEBIAN/control
	cp ./packaging_ubuntu/postinst /tmp/EnigmaBlockchain/deb/DEBIAN/postinst
	chmod 755 /tmp/EnigmaBlockchain/deb/DEBIAN/postinst
	cp ./packaging_ubuntu/postrm /tmp/EnigmaBlockchain/deb/DEBIAN/postrm
	chmod 755 /tmp/EnigmaBlockchain/deb/DEBIAN/postrm
	dpkg-deb --build /tmp/EnigmaBlockchain/deb/ .
	-rm -rf /tmp/EnigmaBlockchain

rename_for_release:
	-rename "s/windows-4.0-amd64/v${VERSION}-win64/" *.exe
	-rename "s/darwin-10.6-amd64/v${VERSION}-osx64/" *darwin*

sign_for_release: rename_for_release
	sha256sum enigma-blockchain*.deb > SHA256SUMS
	-sha256sum enigmad-* enigmacli-* >> SHA256SUMS
	gpg -u 91831DE812C6415123AFAA7B420BF1CB005FBCE6 --digest-algo sha256 --clearsign --yes SHA256SUMS
	rm -f SHA256SUMS

release: sign_for_release
	rm -rf ./release/
	mkdir -p ./release/
	cp enigma-blockchain_*.deb ./release/
	cp enigmacli-* ./release/
	cp enigmad-* ./release/
	cp SHA256SUMS.asc ./release/

clean:
	-rm -rf /tmp/EnigmaBlockchain
	-rm -f ./enigmacli*
	-rm -f ./enigmad*
	-rm -f ./librust_cosmwasm_enclave.signed.so 
	-rm -f ./x/compute/internal/keeper/librust_cosmwasm_enclave.signed.so 
	-rm -f ./go-cosmwasm/api/libgo_cosmwasm.so
	-rm -f ./enigma-blockchain*.deb
	-rm -f ./SHA256SUMS*
	-rm -rf ./third_party/vendor/
	-rm -rf ./.sgx_secrets/*
	-rm -rf ./x/compute/internal/keeper/.sgx_secrets/*
	-rm -rf ./x/compute/internal/keeper/*.der
	-rm -rf ./x/compute/internal/keeper/*.so
	$(MAKE) -C go-cosmwasm clean-all
	$(MAKE) -C cosmwasm/lib/wasmi-runtime clean
# docker build --build-arg SGX_MODE=HW --build-arg SECRET_NODE_TYPE=NODE -f Dockerfile.testnet -t cashmaney/secret-network-node:azuretestnet .
build-azure:
	docker build -f Dockerfile.azure -t cashmaney/secret-network-node:azuretestnet .

build-testnet:
	docker build --build-arg SGX_MODE=HW --build-arg SECRET_NODE_TYPE=BOOTSTRAP -f Dockerfile.testnet -t cashmaney/secret-network-bootstrap:testnet  .
	docker build --build-arg SGX_MODE=HW --build-arg SECRET_NODE_TYPE=NODE -f Dockerfile.testnet -t cashmaney/secret-network-node:testnet .

docker_bootstrap:
	docker build --build-arg SGX_MODE=${SGX_MODE} --build-arg SECRET_NODE_TYPE=BOOTSTRAP -t enigmampc/secret-network-bootstrap-${ext}:${DOCKER_TAG} .

docker_node:
	docker build --build-arg SGX_MODE=${SGX_MODE} --build-arg SECRET_NODE_TYPE=NODE -t enigmampc/secret-network-node-${ext}:${DOCKER_TAG} .
# while developing:
build-enclave:
	$(MAKE) -C cosmwasm/lib/wasmi-runtime 

# while developing:
clean-enclave:
	$(MAKE) -C cosmwasm/lib/wasmi-runtime clean 

sanity-test:
	SGX_MODE=SW $(MAKE) build_linux
	cp ./cosmwasm/lib/wasmi-runtime/librust_cosmwasm_enclave.signed.so .
	SGX_MODE=SW ./cosmwasm/lib/sanity-test.sh
	
sanity-test-hw:
	$(MAKE) build_linux
	cp ./cosmwasm/lib/wasmi-runtime/librust_cosmwasm_enclave.signed.so .
	./cosmwasm/lib/sanity-test.sh

callback-sanity-test:
	SGX_MODE=SW $(MAKE) build_linux
	cp ./cosmwasm/lib/wasmi-runtime/librust_cosmwasm_enclave.signed.so .
	SGX_MODE=SW ./cosmwasm/lib/callback-test.sh

build-test-contract:
	$(MAKE) -C ./x/compute/internal/keeper/testdata/test-contract

go-tests: build-test-contract
	SGX_MODE=SW $(MAKE) build_linux
	cp ./cosmwasm/lib/wasmi-runtime/librust_cosmwasm_enclave.signed.so ./x/compute/internal/keeper
	SGX_MODE=SW go test -p 1 -v ./x/compute/internal/...