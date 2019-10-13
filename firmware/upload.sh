#!/bin/bash

cd firmware

arm-none-eabi-gdb -q target/thumbv7em-none-eabihf/release/micromouse -x openocd.gdb

