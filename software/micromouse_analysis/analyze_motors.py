import serial


def step_motor(s, before_time, step_time, after_time):
    s.write(b'time report on\n')
    s.write(b'motor left report on\n')

    start_time = None

    step = 0

    motor_positions = []

    while True:
        line = s.readline()
        if b',' in line and b':' in line:
            time = None
            motor = None

            words = line.split(b',')
            for word in words:
                parts = word.split(b':')
                if parts[0] == b'T':
                    time = int(parts[1])
                elif parts[0] == b'LM':
                    motor = int(parts[1])

            if start_time is None:
                start_time = time
            else:
                if time is not None and motor is not None:
                    motor_positions.append((time, motor, step))

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


s = serial.Serial(port='/dev/ttyUSB0', baudrate=230400, timeout=1)

p = step_motor(s, 1000, 5000, 5000)

print(p)
