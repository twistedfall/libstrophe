#!/bin/bash

set -xev

# on installation ejabberd generates a new certificate with the following string therein:
#   root@$(hostname -s).$(hostname -d)
# for Azure (hence Github Actions) it results in something like:
#   root@fv-az180-197.aibneqh1mxpuhl3tnnuy2ifbce.bx.internal.cloudapp.net
# which is longer than 64 chars allowed by openssl, thus shorten it here
sudo hostname localhost

sudo apt-get update
sudo apt-get -Vy install makepasswd autoconf libtool libssl-dev libxml2-dev build-essential ejabberd

build_dir=~/build
jabber_username=libstrophe
jabber_host=localhost
jabber_pwd="$(makepasswd)"

mkdir -p "$build_dir"
curl -L "https://github.com/strophe/libstrophe/archive/$PKG_VERSION.tar.gz" | tar -xzC "$build_dir"
pushd "$build_dir/libstrophe-$PKG_VERSION"
./bootstrap.sh
./configure --prefix=/usr
sudo make -j"$(nproc)" install
popd
sudo systemctl start ejabberd
echo -n "$jabber_username@$jabber_host" > jid.txt
echo -n "$jabber_pwd" > password.txt
sudo ejabberdctl register "$jabber_username" "$jabber_host" "$jabber_pwd"
