#!/bin/bash

set -xeu

name="$1"

sudo /usr/sbin/ejabberdctl unregister "$name" localhost
