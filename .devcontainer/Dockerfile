# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.209.6/containers/javascript-node/.devcontainer/base.Dockerfile

# [Choice] Node.js version (use -bullseye variants on local arm64/Apple Silicon): 16, 14, 12, 16-bullseye, 14-bullseye, 12-bullseye, 16-buster, 14-buster, 12-buster
ARG VARIANT="16-bullseye"
FROM mcr.microsoft.com/vscode/devcontainers/javascript-node:0-${VARIANT}

# [Optional] Uncomment this section to install additional OS packages.
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    locales \
    tree \
    fonts-powerline \
    # linuxbrew-wrapper \
    # set up locale
    && locale-gen en_US.UTF-8 && fc-cache -vf

ENV DEBIAN_FRONTEND=dialog
ENV SHELL=/bin/zsh

# [Optional] Uncomment if you want to install an additional version of node using nvm
ARG EXTRA_NODE_VERSION=14
RUN su node -c "source /usr/local/share/nvm/nvm.sh && nvm install ${EXTRA_NODE_VERSION}"

# Use Yarn 2
# RUN yarn set version berry

ARG USER="node"
ARG GROUP="node"

USER ${USER}

# Make variables available in Shell Env
ENV USER=${USER}
ENV GROUP=${GROUP}

# Install powerlevel10k
RUN git clone --depth=1 https://github.com/romkatv/powerlevel10k.git ${ZSH_CUSTOM:-$HOME/.oh-my-zsh/custom}/themes/powerlevel10k

RUN git clone https://github.com/zsh-users/zsh-autosuggestions ${ZSH_CUSTOM:=~/.oh-my-zsh/custom}/plugins/zsh-autosuggestions
RUN git clone https://github.com/zsh-users/zsh-syntax-highlighting.git ${ZSH_CUSTOM:=~/.oh-my-zsh/custom}/plugins/zsh-syntax-highlighting
RUN git clone https://github.com/zsh-users/zsh-completions ${ZSH_CUSTOM:=~/.oh-my-zsh/custom}/plugins/zsh-completions

ADD --chown=${USER}:${GROUP} .zshrc /home/${USER}

# Configure Linuxbrew
# RUN brew update || true
# # Install handy plugins
# RUN brew install zsh-autosuggestions zsh-syntax-highlighting


# [Optional] Uncomment if you want to install more global node modules
# RUN su node -c "npm install -g <your-package-list-here>"

RUN npm install -g near-cli

ENV PATH=/cargo/bin:/rust/bin:$PATH

# https://reposhub.com/rust/development-tools/rust-lang-rustup.html
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly --no-modify-path
# RUN rustup toolchain install nightly --allow-downgrade --profile minimal --component
# RUN rustup target add wasm32-unknown-unknown

# RUN source /home/${USER}/.cargo/env
# https://www.marcelbelmont.com/post/getting_started_with_rustlang/
# RUN echo "export PATH="$HOME/.cargo/bin:$PATH"" >> ~/.zshrc

# Restore cache ownership
# RUN sudo chown -R ${USER}:${GROUP} ~/.cache
