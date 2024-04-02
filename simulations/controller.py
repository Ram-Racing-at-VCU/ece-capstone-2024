import numpy as np
import numpy.linalg as la
import control
from dataclasses import dataclass
from numpy.typing import ArrayLike

class DummyController:
        def __init__(self):
                self.value  = 0.

        def __call__(self, measurement: float):
                return self.value
        
@dataclass
class PIDControllerConfig:
    Kp: float
    Ki: float
    Kd: float
    delta_t: float
    setpoint: callable

class PIDController:
    def __init__(self, cfg: PIDControllerConfig):
        self.proportional = cfg.Kp
        self.integral = cfg.Ki
        self.derivative = cfg.Kd
        self.dt = cfg.delta_t
        self.reference = cfg.setpoint
        self.control_signal = 0
        self.ek = 0
        self.ek_1 = 0
        self.accumulation = 0

    def __call__(self, measurement: float, t: float) -> float:
        self.e = self.reference(t) - measurement
        self.accumulation += self.e

        P = self.proportional * self.e
        I = self.integral * (self.accumulation) * self.dt
        D = self.derivative * (self.e - self.ek_1) / self.dt
        self.control_signal = P+I+D

        self.ek_1 = self.reference(t) - measurement
        return self.control_signal
    

class StateFeedbackRegulator:
    def __init__(self, K: ArrayLike, k_f: float, reference: callable):
        self.K = K
        self.k_f = k_f
        self.reference = reference

    def __call__(self, measurement: float, t: float) -> float:
        return self.k_f * self.reference(t) - self.K @ measurement
    
def feedfoward_gain(A: ArrayLike,
                    B: ArrayLike,
                    C: ArrayLike,
                    K: ArrayLike) -> float:
     return -1 / (C @ (la.inv(A - B@K) @ B))

