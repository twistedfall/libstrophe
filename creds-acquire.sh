#!/bin/bash

set -xeu

name="$1"
pass="$2"

sudo /usr/sbin/ejabberdctl register "$name" localhost "$pass"
