# Simple usage with a mounted data directory:
# > docker build -t enigma .
# > docker run -it -p 26657:26657 -p 26656:26656 -v ~/.enigmad:/root/.enigmad -v ~/.enigmacli:/root/.enigmacli enigma enigmad init
# > docker run -it -p 26657:26657 -p 26656:26656 -v ~/.enigmad:/root/.enigmad -v ~/.enigmacli:/root/.enigmacli enigma enigmad start
FROM baiduxlab/sgx-rust:1804-1.1.2 AS build-env-rust-go

ENV PATH="/root/.cargo/bin:$PATH"
ENV GOROOT=/usr/local/go
ENV GOPATH=/go/
ENV PATH=$PATH:/usr/local/go/bin:$GOPATH/bin


RUN curl -O https://dl.google.com/go/go1.14.2.linux-amd64.tar.gz
RUN tar -C /usr/local -xzf go1.14.2.linux-amd64.tar.gz
# Set working directory for the build

WORKDIR /go/src/github.com/enigmampc/EnigmaBlockchain/

ARG SGX_MODE=SW
ENV SGX_MODE=${SGX_MODE}
ENV MITIGATION_CVE_2020_0551=LOAD

COPY third_party/build third_party/build

# Add source files
COPY go-cosmwasm/ go-cosmwasm/
COPY cosmwasm/ cosmwasm/

WORKDIR /go/src/github.com/enigmampc/EnigmaBlockchain/

COPY Makefile Makefile

RUN make clean
RUN make vendor

WORKDIR /go/src/github.com/enigmampc/EnigmaBlockchain/go-cosmwasm
RUN . /opt/sgxsdk/environment && env && MITIGATION_CVE_2020_0551=LOAD SGX_MODE=${SGX_MODE} make build-rust

# Set working directory for the build
WORKDIR /go/src/github.com/enigmampc/EnigmaBlockchain

# Add source files
COPY go-cosmwasm go-cosmwasm
COPY x x
COPY types types
COPY app.go .
COPY go.mod .
COPY go.sum .
COPY cmd cmd
COPY Makefile .

# COPY /go/src/github.com/enigmampc/EnigmaBlockchain/go-cosmwasm/libgo_cosmwasm.so go-cosmwasm/api

RUN . /opt/sgxsdk/environment && env && MITIGATION_CVE_2020_0551=LOAD SGX_MODE=${SGX_MODE} make build_local_no_rust

# Final image
FROM cashmaney/enigma-sgx-base

ARG SGX_MODE=SW
ENV SGX_MODE=${SGX_MODE}

ARG SECRET_NODE_TYPE=BOOTSTRAP
ENV SECRET_NODE_TYPE=${SECRET_NODE_TYPE}

ENV SCRT_ENCLAVE_DIR=/usr/lib/

# workaround because paths seem kind of messed up
RUN cp /opt/sgxsdk/lib64/* /usr/lib/ -r

# Install ca-certificates
WORKDIR /root

# Copy over binaries from the build-env
COPY --from=build-env-rust-go /go/src/github.com/enigmampc/EnigmaBlockchain/go-cosmwasm/target/release/libgo_cosmwasm.so /usr/lib/
COPY --from=build-env-rust-go /go/src/github.com/enigmampc/EnigmaBlockchain/go-cosmwasm/librust_cosmwasm_enclave.signed.so /usr/lib/
COPY --from=build-env-rust-go /go/src/github.com/enigmampc/EnigmaBlockchain/enigmad /usr/bin/enigmad
COPY --from=build-env-rust-go /go/src/github.com/enigmampc/EnigmaBlockchain/enigmacli /usr/bin/enigmacli

# COPY ./packaging_docker/devnet_init.sh .
COPY ./packaging_docker/wasmi-sgx-test.sh .
COPY ./packaging_docker/bootstrap_init.sh .
COPY ./packaging_docker/node_init.sh .
COPY ./packaging_docker/startup.sh .

RUN chmod +x /usr/bin/enigmad
RUN chmod +x /usr/bin/enigmacli
RUN chmod +x wasmi-sgx-test.sh
RUN chmod +x bootstrap_init.sh
RUN chmod +x startup.sh
RUN chmod +x node_init.sh


RUN mkdir -p /root/.enigmad/.compute/
RUN mkdir -p /root/.sgx_secrets/
RUN mkdir -p /root/.enigmad/.node/
# COPY ./packaging_docker/seed.json /root/.enigmad/.compute/seed.json

COPY ./packaging_docker/node_key.json .
COPY api_key.txt /root/
COPY spid.txt /root/

# Run enigmad by default, omit entrypoint to ease using container with enigmacli
ENTRYPOINT ["/bin/bash", "startup.sh"]