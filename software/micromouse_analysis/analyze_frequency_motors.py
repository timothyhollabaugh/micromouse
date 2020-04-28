import serial
import matplotlib.pyplot as plt
import math
import scipy.optimize
import numpy as np
from more_itertools import *


def frequency_motor(s, run_time, frequency, gain):
    s.write(b'time report on\n')
    s.write(b'motor left report on\n')

    start_time = None
    p = None

    times = []
    positions = []
    powers = []

    while True:
        line = s.readline()
        if b',' in line and b':' in line:
            time = None
            position = None

            words = line.split(b',')
            for word in words:
                parts = word.split(b':')
                if parts[0] == b'T':
                    try:
                        time = int(parts[1])
                    except ValueError:
                        print('Error parsing {} from {}'.format(parts[1], line))
                elif parts[0] == b'LM':
                    try:
                        position = int(parts[1])
                    except ValueError:
                        print('Error parsing {} from {}'.format(parts[1], line))

            if start_time is None:
                start_time = time
            else:
                time = time - start_time
                if time is not None and position is not None:
                    times.append(time)
                    positions.append(position)
                    powers.append(p)

                    if time % 10 == 0:
                        p = gain / 2 + gain / 2 * math.sin(2 * math.pi * time * frequency)
                        s.write(b'motor left set %d\n' % p)

                    if time >= run_time:
                        s.write(b'motor left set 0\n')
                        s.write(b'motor left report off\n')
                        s.write(b'time report off\n')
                        break

    return times, positions, powers


s = serial.Serial('/dev/ttyUSB0', 230400, timeout=1)


def calc_velocity(d):
    (last_time, last_position), (next_time, next_position) = d
    return (next_position - last_position) - (next_time - last_time)


times, positions, powers = frequency_motor(s, 5000, 0.002, 10000)

times, positions, powers = unzip(filter(lambda d: d[0] >= 1000, zip(times, positions, powers)))

times = list(times)
positions = list(positions)
powers = list(powers)

velocities = list(map(calc_velocity, windowed(zip(times, positions), 2)))

velocities.append(last(velocities))


def sine(t, offset, gain, frequency, phase):
    return offset + gain * np.sin(2*math.pi*t*frequency + phase)


popt, pcov = scipy.optimize.curve_fit(sine, times, velocities, p0=[3, 2.5, 0.002, 0])

print(popt)
print(pcov)

fit_velocities = list(map(lambda t: sine(t, *popt), times))

plt.plot(
    #times, velocities,
    times, list(map(lambda p: p / 1000 if p is not None else None, powers)),
    times, fit_velocities,
    linewidth=1
)
plt.show()
