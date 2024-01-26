from dataclasses import dataclass
import numpy as np
import matplotlib.pyplot as plt
from numpy.typing import ArrayLike
from typing import Tuple

@dataclass
class MotorParameters:
    """
    Class: MotorParameters
    Input: none
    Output: none
    Description: Stores the system parameters of the BLDC Motor Object.
    """
    r: float               # Phase resistance [Ohm]
    l: float               # Phase inductance [Henry]
    k_v: float             # Back emf constant [V/rpm]
    k_t: float             # Torque constant [Nm/A]
    b: float               # Rotational damping [Nms/deg]
    J: float               # Rotational Inertia [kg m^2]
    static_friction: float # Static friction coefficient [unitless]
    n_poles: int           # Number of poles [unitless]
