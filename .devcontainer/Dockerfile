#############
# Node.js
#############
FROM ubuntu:jammy as nodejs_tools_context

WORKDIR /node

# Pinned versions of stuff we pull in
ENV BABEL_CLI_VERSION=v7.17.10
ENV BABEL_CORE_VERSION=v7.18.2
ENV BABEL_POLYFILL_VERSION=v7.12.1
ENV BABEL_PRESET_ENV=v7.18.2
ENV BABEL_PRESET_MINIFY_VERSION=v0.5.2
ENV LINKINATOR_VERSION=v2.0.5
ENV MARKDOWN_SPELLCHECK_VERSION=v1.3.1
ENV NODEJS_VERSION=18.14.2
ENV SASS_LINT_VERSION=v1.13.1
ENV SASS_VERSION=v1.52.1
ENV SVGO_VERSION=v1.3.2
ENV SVGSTORE_CLI_VERSION=v1.3.2
ENV TSLINT_VERSION=v6.1.3
ENV TYPESCRIPT_VERSION=v4.7.2

RUN apt-get update && apt-get install -y --no-install-recommends \
    wget ca-certificates

RUN set -eux; \
    case $(uname -m) in \
    x86_64) NODEJS_TAR=node-v${NODEJS_VERSION}-linux-x64.tar.gz;; \
    aarch64) NODEJS_TAR=node-v${NODEJS_VERSION}-linux-arm64.tar.gz;; \
    *) echo "unsupported architecture"; exit 1 ;; \
    esac; \
    wget -nv -O /tmp/${NODEJS_TAR} https://nodejs.org/download/release/v${NODEJS_VERSION}/${NODEJS_TAR}; \
    tar -xzf /tmp/${NODEJS_TAR} --strip-components=1 -C /usr/local

ADD https://nodejs.org/download/release/v${NODEJS_VERSION}/node-v${NODEJS_VERSION}-headers.tar.gz /tmp
RUN tar -xzf /tmp/node-v${NODEJS_VERSION}-headers.tar.gz --strip-components=1 -C /usr/local

RUN npm init -y
RUN npm install --omit=dev --global \
    sass@"${SASS_VERSION}" \
    sass-lint@"${SASS_LINT_VERSION}" \
    typescript@"${TYPESCRIPT_VERSION}" \
    tslint@"${TSLINT_VERSION}" \
    markdown-spellcheck@"${MARKDOWN_SPELLCHECK_VERSION}" \
    svgstore-cli@"${SVGSTORE_CLI_VERSION}" \
    svgo@"${SVGO_VERSION}" \
    @babel/core@"${BABEL_CORE_VERSION}" \
    @babel/cli@"${BABEL_CLI_VERSION}" \
    @babel/preset-env@"${BABEL_PRESET_ENV_VERSION}" \
    linkinator@"${LINKINATOR_VERSION}"

RUN npm install --omit=dev --save-dev \
    babel-preset-minify@${BABEL_PRESET_MINIFY_VERSION}

RUN npm install --save-dev \
    @babel/polyfill@${BABEL_POLYFILL_VERSION}

# Clean up stuff we don't need in the final image
RUN rm -rf /usr/local/sbin
RUN rm -rf /usr/local/share

FROM public.ecr.aws/docker/library/rust:bullseye as builder

COPY --from=nodejs_tools_context /usr/local/bin /usr/local/bin
COPY --from=nodejs_tools_context /usr/local/lib/node_modules /usr/local/lib/node_modules
COPY --from=nodejs_tools_context /node/node_modules /node_modules

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    apt-get install -y protobuf-compiler && \
    apt-get install -y cmake curl && \
    apt-get install -y htop iftop sysstat procps lsof net-tools && \
    update-ca-certificates && \
    # rust
    rustup toolchain install nightly-2023-01-26 && \
    rustup default nightly-2023-01-26 && \
    rustup target add x86_64-unknown-linux-gnu && \
    rustc --print cfg && \
    rustc -C target-cpu=native --print cfg
