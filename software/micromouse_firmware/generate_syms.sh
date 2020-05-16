#!/bin/bash

arm-none-eabi-nm -S -n ../target/thumbv7em-none-eabihf/release/micromouse_firmware | sort -r > syms
