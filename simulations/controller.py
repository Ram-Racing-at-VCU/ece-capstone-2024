import numpy as np
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
    setpoint: ArrayLike

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

    def __call__(self, measurement: float) -> float:
        self.e = self.reference - measurement
        self.accumulation += self.e

        P = self.proportional * self.e
        I = self.integral * (self.accumulation) * self.dt
        D = self.derivative * (self.e - self.ek_1) / self.dt
        self.control_signal = P+I+D

        self.ek_1 = self.reference - measurement
        return self.control_signal