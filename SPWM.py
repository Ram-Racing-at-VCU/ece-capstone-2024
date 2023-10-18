from math import sqrt
from math import pi
from math import atan

import numpy as np
from numpy import arange

from scipy import signal

def integral(x, dt):
# Function: integral
# Input parameters: Value vector (x), time increment (dt)
# Output: Integral of x (I)
# Description: Calculates the integral of an arbitrary input signal for
# the period of time the signal was defined for. Returns a single value.

    I = 0
    for i in len(x):
        I = I + x(i)*dt

    return I

def magnitude(f, R, L):
# Function: magnitude
# Input parameters: Frequency Vector (f), Resistance (R), Inductance (L)
# Output: RL circuit magnitude (M)
# Description: Generates the amplitdue spectra of an RL lowpass filter for
# given parameters and frequency range.

    timeRange = range(0, len(f))
    M = []
    for t in timeRange:
        M[t] = R/(sqrt(R**2 + (2*pi*f[t])^2)*L**2)
    return M

def phi(f, R, L):
# Function: phase
# Input parameters: Frequency Vector (f), Resistance (R), Inductance (L)
# Output: RL circuit phase (phi)
# Description: Generates the phase spectra of an RL lowpass filter for
# given parameters and frequency range.
    timeRange = range(0, len(f))
    phi = []
    for t in timeRange:
        phi[t] = atan(2*pi*f[t]*L/R)
    return phi



#***********************************#
#             PART 1                #
#***********************************#

T0 = 0.02
f_c = 5000
f_m = 50
dt = 1e-6

t = arange(0, T0, dt)

carrier_signal = 1/2 * signal.sawtooth(2*pi*f_c*t)
