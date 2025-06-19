FROM rust:latest

# Install Dependencies
RUN apt-get update \
  && apt-get install -y --no-install-recommends \
     pkg-config \
     libssl-dev \
     bash \
     git \
  && rm -rf /var/lib/apt/lists/*

ARG USER_NAME=rustuser
ARG USER_ID=1000
ARG GROUP_ID=1000
RUN groupadd -g ${GROUP_ID} ${USER_NAME} \
    && useradd -m -u ${USER_ID} -g ${USER_NAME} ${USER_NAME}

WORKDIR /workspace

USER ${USER_NAME}

CMD ["/bin/bash"]
