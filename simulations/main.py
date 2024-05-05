import numpy as np
import matplotlib.pyplot as plt
from motor import *
from controller import *
import random

def unit_step(t: float) -> float:
    """
    Routine: unit_step
    Inputs: t (float)
    Output: float
    Descrption: Step function implementation in Python
    """
    return 1.0 if t >= 0 else 0


def unit_impulse(t: float) -> float:
    return 1. if t == 0 else 0

def main():
    cfg = DCMotorConfig(1.8, 0.85e-2, 0.035, 0.032, 4.6, 6.2)
    motor_dynamics = DCMotorDynamics(cfg)
    dc_motor = DCMotor(cfg)

    r = lambda t: 13*(0.5*unit_step(t) - 0.2*unit_step(t-1) + 0.1*unit_step(t-3) - 0.4*unit_step(t-3.5) + 0.7*unit_step(t-4) - 0.3*unit_step(t-6) + 0.1*unit_step(t-7) - 0.2*unit_step(t-8) + 0.4*unit_step(t-9.9))

    controller_cfg = PIDControllerConfig(30., 10., 0., 0.001, r)
    pid_controller = PIDController(controller_cfg)

    Q_x = np.array([[0, 0, 0],
                    [0, 50, 0],
                    [0, 0, 0]])

    K, _, _ = control.lqr(motor_dynamics.A, motor_dynamics.B, Q_x, 1/(12**2))
    k_f = feedfoward_gain(motor_dynamics.A, motor_dynamics.B, np.array([[0, 1, 0]]), K)
    lqr_controller = StateFeedbackRegulator(K, k_f, r)

    t, y_lqr, u_lqr = simulate_lqr(dc_motor, lqr_controller, [1., 0., 0.], 0., 10., 0.001, np.array([[0., 1., 0.]]))
    t, y_pid, u_pid = simulate_pid(dc_motor, pid_controller, [1., 0., 0.], 0., 10., 0.001, np.array([[0., 1., 0.]]))
    
    y_lqr = y_lqr[0].T[0]
    y_pid = y_pid[0].T[0]

    plt.rcParams['font.family'] = "Arial"
    plt.rcParams['font.size'] = 14
    plt.figure(1)
    plt.plot(t, [r(dt) for _, dt in enumerate(t)], "--k", label = "Desired Acceleration")
    plt.plot(t, y_pid, "-r", label = "PID Controller")
    plt.plot(t, y_lqr, "-b", label = "LQR Controller")
    plt.xlabel("Time [s]", fontdict={"fontfamily": "Arial"})
    plt.ylabel(r"Acceleration [$\frac{m}{s^2}$]", fontdict={"fontfamily": "Arial"})
    plt.title("PID vs. LQR Performance", weight="bold", fontdict={"fontfamily": "Arial", "fontsize": 18})
    plt.xlim((0, 10))
    plt.legend()
    plt.grid()
    plt.show()


    # plt.figure(2)
    # plt.plot(t, u_pid)
    # plt.plot(t, u_lqr)
    # plt.plot(t, np.array(u_pid[0]).T[0])
    # plt.plot(t, u_lqr)

if __name__ == "__main__":
        main()
