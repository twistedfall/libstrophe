#!/bin/bash

set -xeu

# on installation ejabberd generates a new certificate with the following string therein:
#   root@$(hostname -s).$(hostname -d)
# for Azure (hence Github Actions) it results in something like:
#   root@fv-az180-197.aibneqh1mxpuhl3tnnuy2ifbce.bx.internal.cloudapp.net
# which is longer than 64 chars allowed by openssl, thus shorten it here
sudo hostname localhost

sudo apt-get update
sudo apt-get -Vy install makepasswd autoconf libtool libssl-dev libxml2-dev build-essential ejabberd

build_dir=~/build

mkdir -p "$build_dir"
curl -L "https://github.com/strophe/libstrophe/archive/$LIBSTROPHE_VERSION.tar.gz" | tar -xzC "$build_dir"
if [[ "$LIBSTROPHE_VERSION" == "0.13.1" ]]; then
	# https://github.com/strophe/libstrophe/commit/9fef4b7d024b99aac9101bfa8b45cf78eef6508b
	patch -p1 -d "$build_dir/libstrophe-$LIBSTROPHE_VERSION" < ci/0.13.1-tls-segfault.patch
fi
cd "$build_dir/libstrophe-$LIBSTROPHE_VERSION"
./bootstrap.sh
./configure --prefix=/usr
sudo make -j"$(nproc)" install
cd -

sudo sed -ri 's/starttls_required: true/starttls_required: false\n    starttls: true/' /etc/ejabberd/ejabberd.yml
sudo systemctl restart ejabberd
