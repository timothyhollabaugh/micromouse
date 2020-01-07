#!/bin/bash

cd firmware

openocd -f ./openocd.cfg &

arm-none-eabi-gdb -q target/thumbv7em-none-eabihf/release/micromouse -x openocd.gdb

killall openocd