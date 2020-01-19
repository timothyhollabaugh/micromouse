#!/bin/bash

stty -F $1 230400 cs8 raw -echo -echoe -echok
websocat -b -n ws-listen:0.0.0.0:8080 - < $1 > $1