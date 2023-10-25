# from math import sqrt
# from math import pi
# from math import atan
# from math import cos

import numpy as np
from numpy import sqrt as sqrt
from numpy import pi as pi
from numpy import arctan as atan
from numpy import cos as cos
from numpy import arange

import matplotlib.pyplot as plt

from scipy import signal

def integral(x, dt):
# Function: integral
# Input parameters: Value vector (x), time increment (dt)
# Output: Integral of x (I)
# Description: Calculates the integral of an arbitrary input signal for
# the period of time the signal was defined for. Returns a single value.

    I = 0
    for item in x:
        I = I + item*dt
    return I

def magnitude(f, R, L):
# Function: magnitude
# Input parameters: Frequency Vector (f), Resistance (R), Inductance (L)
# Output: RL circuit magnitude (M)
# Description: Generates the amplitdue spectra of an RL lowpass filter for
# given parameters and frequency range.

    M = []
    if type(f) == int:
        return (R/(sqrt(R**2 + (2*pi*f)**2 * L**2)))
    for freq in f:
        M.append(R/(sqrt(R**2 + (2*pi*freq)**2 * L**2)))
    return M

def phi(f, R, L):
# Function: phase
# Input parameters: Frequency Vector (f), Resistance (R), Inductance (L)
# Output: RL circuit phase (phi)
# Description: Generates the phase spectra of an RL lowpass filter for
# given parameters and frequency range. (In degrees!!)
    phi = []
    if type(f) == int:
        return atan(2*pi*f*L/R)
    for freq in f:
        phi.append(atan(2*pi*freq*L/R)) # converting to degrees here
    return phi



#***********************************#
#             PART 1                #
#***********************************#

T0 = 0.02
f_c = 5000
f_m = 50
dt = 1e-6

t = arange(0, T0, dt)
carrier_signal = 1/2 * signal.sawtooth(2*pi*f_c*t, width=0.5) + 1/2 # adding 1/2 to go from 1 to 0
"""
Width of the rising ramp as a proportion of the total cycle.
Default is 1, producing a rising ramp, while 0 produces a falling
ramp.  `width` = 0.5 produces a triangle wave.
"""
message_signal = 0.99/2 * cos(2*pi*f_m*t) + 0.99/2

# spwm_signal = np.empty(len(t))
spwm_signal = message_signal >= carrier_signal

plt.figure()
plt.plot(1000*t, carrier_signal)
plt.plot(1000*t, message_signal)

plt.title("The Carrier and Message Signals vs. Time (ms)")
plt.xlabel("Time (ms)")
plt.legend(["Carrier Signal", "Message Signal"])

plt.figure()

plt.title("The SPWM Signal vs. Time (ms)")
plt.plot(1000*t, spwm_signal)
plt.xlabel("Time (ms)")
plt.legend(["SPWM Signal"])

# plt.show()

#***********************************#
#             PART 2                #
#***********************************#

dc_component = 1/T0 * integral(spwm_signal, dt)
coeffRange = 1000
harmRange = 100
coeff = []

for n in range(0, coeffRange):
    res_list = spwm_signal * cos(2*pi*f_m*n*t)
    test = integral(res_list, dt)
    coeff.append(2/T0 * test)
harmonics = np.array(dc_component)


# Calculating and plotting the first 100 harmonics
for n in range(1, harmRange):
    harmonics =  harmonics + coeff[n] * cos(2*pi*n*f_m*t)
plt.figure()
plt.plot(1000*t, harmonics)
plt.title("The First 100 Harmonics vs. Time")

harmRange = 1000
harmonics = np.array(dc_component)
# Calculating and plotting the first 1000 harmonics
for n in range(1, harmRange):
    harmonics =  harmonics + coeff[n] * cos(2*pi*n*f_m*t)
plt.figure()
plt.plot(1000*t, harmonics)
plt.title("The First 1000 Harmonics vs. Time")

plt.figure()
pltResult = []
for time in t:
    pltResult.append(coeff[1]*cos(2*pi*f_m*time) + dc_component)
plt.plot(1000*t, pltResult)
plt.title("First Coefficient vs. Time (ms)")


#***********************************#
#             PART 3                #
#***********************************#

R = 0.5 # Ohms
L = 0.0005 # Henries

# f = np.linspace(0,1000)

# M = magnitude(f, R, L)
# P = phi(f, R, L)

# plt.figure()
# plt.plot(f, M)
# plt.legend(["Magnitude"])
# plt.show()

# plt.figure()
# plt.plot(f, 180/pi*P)
# plt.legend(["Phase (deg)"])
# plt.show()




#***********************************#
#             PART 4                #
#***********************************#

out_sig_1 = 0
print(len(coeff))
for n in range(0,coeffRange):
    out_sig_1 = out_sig_1 + magnitude(n*f_m, R, L)*coeff[n]*cos(2*pi*n*f_m*t + phi(n*f_m, R, L))

plt.figure()
plt.plot(1000*t, out_sig_1)
plt.title("Output simulation with R = 0.5 \Omega and L = 0.5 mH")

L = 0.00005
out_sig_2 = 0
print(len(coeff))
for n in range(0,coeffRange):
    out_sig_2 = out_sig_2 + magnitude(n*f_m, R, L)*coeff[n]*cos(2*pi*n*f_m*t + phi(n*f_m, R, L))

plt.figure()
plt.plot(1000*t, out_sig_2)
plt.title("Output simulation with R = 0.5 \Omega and L = 0.05 mH")
plt.show()