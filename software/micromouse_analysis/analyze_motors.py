import serial
from more_itertools import *
import matplotlib.pyplot as plt
from matplotlib import collections


def step_motor(s, before_time, step_time, after_time):
    s.write(b'time report on\n')
    s.write(b'motor left report on\n')

    start_time = None

    step = 0

    motor_positions = []

    while True:
        line = s.readline()
        print(line)
        if b',' in line and b':' in line:
            time = None
            position = None

            words = line.split(b',')
            for word in words:
                parts = word.split(b':')
                if parts[0] == b'T':
                    time = int(parts[1])
                elif parts[0] == b'LM':
                    position = int(parts[1])

            if start_time is None:
                start_time = time
            else:
                if time is not None and position is not None:
                    motor_positions.append({
                        'time': time,
                        'position': position,
                        'step': step,
                    })

                if step == 0 and time - start_time > before_time:
                    s.write(b'motor left set 10000\n')
                    start_time = time
                    step = 1
                elif step == 1 and time - start_time > step_time:
                    s.write(b'motor left set 0\n')
                    start_time = time
                    step = 2
                elif step == 2 and time - start_time > after_time:
                    s.write(b'motor left report off\n')
                    s.write(b'time report off\n')
                    break

    return motor_positions


def calc_velocity(positions):
    current_position, next_position = positions
    delta_time = next_position['time'] - current_position['time']
    delta_position = next_position['position'] - current_position['position']
    return {
        'time': current_position['time'],
        'step': current_position['step'],
        'velocity': delta_position / delta_time,
    }


def to_velocity(motor_positions):
    return list(map(calc_velocity, windowed(motor_positions, 2)))


def graph(ax, data, key):
    times = list(map(lambda d: d['time'], data))
    datas = list(map(lambda d: d[key], data))
    steps = list(map(lambda d: d['step'], data))

    steps0 = list(map(lambda s: s == 0, steps))
    steps1 = list(map(lambda s: s == 1, steps))
    steps2 = list(map(lambda s: s == 2, steps))

    steps0_collection = collections.BrokenBarHCollection.span_where(times, ymin=min(datas), ymax=max(datas),
                                                                    where=steps0,
                                                                    facecolor='yellow',
                                                                    alpha=0.5)

    steps1_collection = collections.BrokenBarHCollection.span_where(times, ymin=min(datas), ymax=max(datas),
                                                                    where=steps1,
                                                                    facecolor='green',
                                                                    alpha=0.5)

    steps2_collection = collections.BrokenBarHCollection.span_where(times, ymin=min(datas), ymax=max(datas),
                                                                    where=steps2,
                                                                    facecolor='red',
                                                                    alpha=0.5)

    ax.plot(times, datas)
    ax.add_collection(steps0_collection)
    ax.add_collection(steps1_collection)
    ax.add_collection(steps2_collection)


s = serial.Serial(port='/dev/ttyUSB0', baudrate=230400, timeout=1)

p = step_motor(s, 100, 1500, 400)

v = to_velocity(p)

fig, (ax1, ax2) = plt.subplots(1, 2)
graph(ax1, v, 'velocity')
graph(ax2, p, 'position')

plt.show()
