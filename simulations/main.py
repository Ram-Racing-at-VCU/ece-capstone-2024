import numpy as np
import matplotlib.pyplot as plt
from motor import *
from controller import *

def main():
        cfg = DCMotorConfig(1.8, 0.85e-2, 0.035, 0.032, 4.6, 6.2)
        dc_motor = DCMotor(cfg)

        controller_cfg = PIDControllerConfig(100, 0., 0., 0.001, 1.)
        pid_controller = PIDController(controller_cfg)

        t, y = simulate(dc_motor, pid_controller, [1., 0., 0.], 0, 10., 0.001, np.array([[0., 1., 0.]]))
        y = y[0].T[0]
        plt.plot(t, y)
        plt.show()


if __name__ == "__main__":
        main()
