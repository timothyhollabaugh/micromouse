from typing import *
import control

# ta: 44.8ms
# final_v: 0.0008253731343283582 ticks per ms per output power


def calc_motor_tf(ta: float, final_v: float):
    s = control.tf('s')
    tf = final_v / (ta * s + 1)
    return tf


def calc_pid_tf(p: float, i: float, d: float):
    s = control.tf('s')
    return p + (i / s) + (d * s)


def tune_motors(ta: float, final_v: float) -> Tuple[float, float, float]:
    motor_tf = calc_motor_tf(ta, final_v)
    pid_tf = calc_pid_tf()
