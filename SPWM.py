from math import sqrt
from math import pi
from math import atan
from math import cos

import numpy as np
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
    for freq in f:
        # 
        phi.append(180/pi*atan(2*pi*freq*L/R)) # converting to degrees here
    return phi



#***********************************#
#             PART 1                #
#***********************************#

T0 = 0.02
f_c = 5000
f_m = 50
dt = 1e-6

t = arange(0, T0, dt)

carrier_signal = 1/2 * signal.sawtooth(2*pi*f_c*t, width=1) + 1/2 # adding 1/2 to go from 1 to 0
"""
Width of the rising ramp as a proportion of the total cycle.
Default is 1, producing a rising ramp, while 0 produces a falling
ramp.  `width` = 0.5 produces a triangle wave.
"""
message_signal = []
spwm_signal = []
count = 0
for time in t:
   message_signal.append(0.99/2 * cos(2*pi*f_m*time) + 0.99/2)
   if(message_signal[count] >= carrier_signal[count]):
       spwm_signal.append(1)
   else:
       spwm_signal.append(0)
   count += 1

# plt.figure()
# plt.plot(1000*t, carrier_signal)
# plt.plot(1000*t, message_signal)

# plt.title("The Carrier and Message Signals vs. Time (ms)")
# plt.xlabel("Time (ms)")
# plt.legend(["Carrier Signal", "Message Signal"])

# plt.figure()

# plt.title("The SPWM Signal vs. Time (ms)")
# plt.plot(1000*t, spwm_signal)
# plt.xlabel("Time (ms)")
# plt.legend(["SPWM Signal"])

# plt.show()

#***********************************#
#             PART 2                #
#***********************************#

# dc_component = 1/T0 * integral(spwm_signal, dt)
# coeffRange = 1000
# harmRange = 100
# coeff = []

# for n in range(1,coeffRange+1):
#     res_list = [spwm_signal[i]* cos(2*pi*f_m*n*t[i]) for i in range(len(spwm_signal))]
#     coeff.append(2/T0 * integral(res_list, dt))

# harmonic_100 = []
# harmonic_100[0] = dc_component

# for n in range(1, harmRange+1):
#     harmonic_100.append(coeff[n]*cos(2*pi*n*f_m*t[n]))

# plt.figure()
# plt.plot(1000*t, spwm_signal)
# plt.title("SPWM Signal vs. Time (ms)")

# plt.figure()
# pltResult = []
# for time in t:
#     pltResult.append(coeff[1]*cos(2*pi*f_m*time) + dc_component)
# plt.plot(1000*t, pltResult)
# plt.title("First Coefficient vs. Time (ms)")

# plt.figure()
# plt.plot(1000*t, harmonic_100)
# plt.title("The First 100 Harmonics vs. Time")

# plt.show()


#***********************************#
#             PART 3                #
#***********************************#

R = 0.5 # Ohms
L = 0.0005 # Henries

f = np.linspace(0,1000)

M = magnitude(f, R, L)
P = phi(f, R, L)

print(M)

plt.figure()
plt.plot(f, M)
plt.legend(["Magnitude"])
plt.show()

plt.figure()
plt.plot(f, P)
plt.legend(["Phase (deg)"])
plt.show()




#***********************************#
#             PART 4                #
#***********************************#

out_sig_1 = [0]
