import ltspice
import matplotlib.pyplot as plt
import numpy as np
import os

l = ltspice.Ltspice(os.path.dirname(__file__)+'/Draft1.raw') 
# Make sure that the .raw file is located in the correct path
l.parse() 

time = l.get_time()
V_source = l.get_data('V(n004)')
V_cap_max = []

#print(V_source)

plt.plot(time, V_source)
plt.xlabel("Time (s)")
plt.ylabel("Voltage (V)")

#plt.xlim((0, 1e-3))
#plt.ylim((-15, 15))
plt.grid()
plt.show()

exit(0)