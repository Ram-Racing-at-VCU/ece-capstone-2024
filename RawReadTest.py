
"""
Code taken from https://pypi.org/project/PyLTSpice/
PyPi Documentation

pip install PyLTSpice
^^^^ Prerequisite ^^^^

"""
from PyLTSpice import RawRead

from matplotlib import pyplot as plt

LTR = RawRead("SimpleSim.raw")

print(LTR.get_trace_names())
print(LTR.get_raw_property())

IR1 = LTR.get_trace("V(n001)")
x = LTR.get_trace('time')  # Gets the time axis
steps = LTR.get_steps()
for step in range(len(steps)):

   plt.plot(x.get_wave(step), IR1.get_wave(step), label=steps[step])

plt.xlabel("Time (s)")
plt.ylabel("Voltage (V)")
plt.legend(["Control Signal"])  # order a legend
plt.show()