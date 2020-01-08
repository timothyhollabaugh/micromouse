#!/bin/bash

openocd -f ./openocd.cfg &

arm-none-eabi-gdb -q ../target/thumbv7em-none-eabihf/release/micromouse_firmware -x openocd.gdb

killall openocd