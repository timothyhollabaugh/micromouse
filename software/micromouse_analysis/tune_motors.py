from typing import *
import control
import matplotlib.pyplot as plt

# ta: 44.8ms
# final_v: 0.0008253731343283582 ticks per ms per output power


def calc_motor_tf(ta: float, final_v: float):
    s = control.tf('s')
    tf = final_v / (ta * s + 1)
    return tf


def calc_delay_tf(t):
    num, den = control.pade(t, 1)
    tf = control.tf(num, den)
    return tf


def calc_pid_tf(p: float, i: float, d: float):
    s = control.tf('s')
    return p + (i / s) + (d * s)


def tune_motors(ta: float, final_v: float, target: float) -> Tuple[float, float, float]:
    motor_tf = calc_motor_tf(ta, final_v)

    print(motor_tf)

    print(control.zero(motor_tf))
    print(control.pole(motor_tf))

    delay_tf = calc_delay_tf(10)

    print(delay_tf)

    # From manual tuning
    pid_tf = calc_pid_tf(4000, 5, 15000)

    #pid_tf = calc_pid_tf(10000, 0, 0)

    print(pid_tf)

    forward_tf = pid_tf * delay_tf * motor_tf

    system_tf = forward_tf / (1 + forward_tf)

    print(system_tf)

    control.pzmap(system_tf)
    print(control.zero(system_tf))
    print(control.pole(system_tf))

    times, values = control.step_response(target * system_tf, range(0, 500))

    fig, ax = plt.subplots(1, 1)
    ax.plot(times, values)

    plt.show()


if __name__ == '__main__':
    tune_motors(44.8, 0.0008253731343283582, 4)