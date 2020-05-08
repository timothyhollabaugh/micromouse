import matplotlib.pyplot as plt
from bezier._py_curve_helpers import get_curvature, evaluate_hodograph
from bezier.curve import Curve
from more_itertools import *
import pint
import numpy as np
import math as m

u = pint.UnitRegistry()
u.setup_matplotlib()


def curvature_at_t(curve, t):
    curve, units = curve
    tangent_vec = evaluate_hodograph(t, curve.nodes)
    curvature = get_curvature(curve.nodes, tangent_vec, t)
    return curvature * u.radians / units


def length_at_t(curve, t):
    curve, units = curve
    curve_to_t = curve.specialize(0, t)
    return curve_to_t.length * units


def calc_accelerations(d):
    (last_time, next_time), (last_velocity, next_velocity) = d

    delta_time = next_time - last_time
    delta_velocity = next_velocity - last_velocity
    acceleration = delta_velocity / delta_time

    center_time = (next_time + last_time) / 2

    return center_time, acceleration


def map_units(f, i):
    results_with_units = list(map(f, i))
    results_without_units = list(map(lambda x: x.magnitude, results_with_units))
    return np.array(results_without_units) * results_with_units[0].units


def extract_units(i):
    results_with_units = list(i)
    results_without_units = list(map(lambda x: x.magnitude, results_with_units))
    return np.array(results_without_units) * results_with_units[0].units


def acceleration_curve(fig, wheelbase: float, wheel_radius: float, mass: float, curve, linear_velocity: float):
    ss = np.linspace(0, 1, 1000)

    ls = map_units(lambda t: length_at_t(curve, t), ss)

    ts = ls / linear_velocity

    curvatures = map_units(lambda t: curvature_at_t(curve, t), ss)

    angular_velocities = map_units(lambda curvature: curvature * linear_velocity, curvatures)

    left_velocities = map_units(lambda angular_velocity: linear_velocity - angular_velocity * wheelbase / 2, angular_velocities)
    right_velocities = map_units(lambda angular_velocity: linear_velocity + angular_velocity * wheelbase / 2, angular_velocities)

    left_acc_ts, left_accelerations = unzip(map(calc_accelerations, zip(windowed(ts, 2), windowed(left_velocities, 2))))

    left_acc_ts = extract_units(left_acc_ts)
    left_accelerations = extract_units(left_accelerations)

    right_acc_ts, right_accelerations = unzip(
        map(calc_accelerations, zip(windowed(ts, 2), windowed(right_velocities, 2))))

    right_acc_ts = extract_units(right_acc_ts)
    right_accelerations = extract_units(right_accelerations)

    left_angular_acceleration = left_accelerations / (wheel_radius / u.radian)
    right_angular_acceleration = right_accelerations / (wheel_radius / u.radian)

    rotational_inertia = mass * wheel_radius**2 / 2

    left_torque = rotational_inertia * left_angular_acceleration
    right_torque = rotational_inertia * right_angular_acceleration

    (ax1, ax2), (ax3, ax4) = fig.subplots(2, 2)

    ts.ito_base_units()
    curvatures.ito_base_units()
    angular_velocities.ito_base_units()
    left_velocities.ito_base_units()
    right_velocities.ito_base_units()
    left_accelerations.ito_base_units()
    right_accelerations.ito_base_units()
    left_acc_ts.ito_base_units()
    right_acc_ts.ito_base_units()
    left_angular_acceleration.ito_base_units()
    right_angular_acceleration.ito_base_units()
    left_torque.ito(u.newton * u.meter)
    right_torque.ito(u.newton * u.meter)

    print(max(left_torque))

    ax1.plot(left_acc_ts, left_torque)
    ax1.plot(right_acc_ts, right_torque)
    ax2.plot(ts, curvatures)
    ax2l = ax2.twinx()
    ax2l.plot(ts, angular_velocities)
    ax4.plot(ts, left_velocities)
    ax4.plot(ts, right_velocities)
    ax3.plot(left_acc_ts, left_accelerations)
    ax3.plot(right_acc_ts, right_accelerations)
    ax3l = ax3.twinx()
    ax3l.plot(left_acc_ts, left_angular_acceleration)
    ax3l.plot(right_acc_ts, right_angular_acceleration)


def corner_curve(r, offset):
    units = r.units
    r = r.magnitude

    r1 = r * 0.5
    r2 = r * 0.3

    nodes = [
        [0, 0, 0, r2, r1, r + offset.to(units).magnitude],
        [r - offset.magnitude, r1, r2, 0, 0, 0]
    ]

    return Curve(nodes, degree=5), units


fig1, ax1 = plt.subplots(1, 1)
fig2 = plt.figure()

c = corner_curve(90 * u.mm, 12 * u.mm)

c[0].plot(1000, ax=ax1)

acceleration_curve(fig2, 72 * u.mm, 16 * u.mm, 87 * u.g, c, 0.4 * u.m / u.s)

plt.show()
