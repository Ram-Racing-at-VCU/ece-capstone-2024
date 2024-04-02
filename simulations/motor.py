import numpy as np
from dataclasses import dataclass
from typing import Tuple
from numpy.typing import ArrayLike
from controller import *

@dataclass
class DCMotorConfig:
        r:   float
        l:   float
        b: float
        j: float
        k_m:   float
        k_t:   float

        def to_dict(self) -> dict:

                return self.__dict__.copy()
        

class DCMotorDynamics:

        def __init__(self, motor_config: DCMotorConfig) -> None:
                self.A = np.array([[    -motor_config.r / motor_config.l,    0,  -motor_config.k_m / motor_config.l],
                                [                0,                 0,              1               ],
                                [  motor_config.k_t / motor_config.j,   0,  -motor_config.b / motor_config.j  ]], float)
                
                self.B = np.array([[1 / motor_config.l], [0], [0]], float)

                self.motor_config = motor_config

        def __call__(self, x: ArrayLike, u: float) -> ArrayLike:
                return self.A @ x + self.B * u
        
        def __str__(self) -> str:
                """
                Method: DCMotor.__str__
                Input: self
                Output: str
                Description: Prints out system parameters used, and system model generated from
                system parameters. Useful for system diagnostics.
                """
                return ("System Parameters: \n\n" + str(self.motor_config.to_dict()) + "\n\n"
                        + "System Model:\n\nx_dot = Ax + Bu\n\n"
                        + "State Matrices:\n\n"
                        + "A =\n\n" + np.array_str(self.A, precision = 2) + "\n\n"
                        + "B =\n\n" + np.array_str(self.B, precision = 2))
        
class DCMotor:

        def __init__(self, config: DCMotorConfig):
                self.config = config
                self.dynamics = DCMotorDynamics(config)

        def output(self, x: ArrayLike, C: ArrayLike) -> float:
                return np.dot(C, x)
        

def rk2_step(dyn_func: callable,
             u_func: callable,
             x: ArrayLike,
             t: float,
             delta_t: float) -> ArrayLike:
        
        f1 = dyn_func(x, u_func(t));
        f2 = dyn_func(x + f1 * delta_t / 2, u_func(t) + delta_t / 2)
        return x + f2*delta_t

def simulate_pid(plant: DCMotor,
                controller: PIDController,
                x_0: ArrayLike,
                t_0: float,
                t_f: float,
                delta_t: float,
                C: ArrayLike) -> Tuple:
        
        t = np.arange(t_0, t_f, delta_t)
        x = [[[x_0[0]], [x_0[1]], [x_0[2]]]]
        u = [0]

        for i, t_curr in enumerate(t[:-1]):
                u_func = lambda t_curr, measurement = plant.output(x[i], C): controller(measurement, t_curr)
                x.append(rk2_step(plant.dynamics, u_func, x[i], t_curr, delta_t))
                u.append(u_func(t_curr)[0][0])

        return (t, plant.output(x, C), u)

def simulate_lqr(plant: DCMotor,
                controller: StateFeedbackRegulator,
                x_0: ArrayLike,
                t_0: float,
                t_f: float,
                delta_t: float,
                C: ArrayLike) -> Tuple:
        
        t = np.arange(t_0, t_f, delta_t)
        x = [[[x_0[0]], [x_0[1]], [x_0[2]]]]
        u = [0]

        for i, t_curr in enumerate(t[:-1]):
                u_func = lambda t_curr, state = x[i]: controller(state, t_curr)
                x.append(rk2_step(plant.dynamics, u_func, x[i], t_curr, delta_t))
                u.append(u_func(t_curr)[0][0])

        return (t, plant.output(x, C), u)