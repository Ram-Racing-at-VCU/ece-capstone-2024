import ltspice
import matplotlib.pyplot as plt
import numpy as np
import os

# from EditorTest import voltRange

currNode = "V(Out)"

for index in range(1,16):
    l = ltspice.Ltspice(os.path.dirname(__file__)+f'\\temp\SimpleSim_{str(index)}.raw') 
    # Make sure that the .raw file is located in the correct path
    l.parse() 

    time = l.get_time()
    V_source = l.get_data(f'{currNode}')
    V_cap_max = []

    plt.plot(time, V_source) #label=f"{voltRange[index]}V")
    plt.xlabel("Time (s)")
    plt.ylabel(f"Voltage (V) at {currNode}")
    

# plt.legend()
plt.title(f"Voltage Output at {currNode} (V) vs. Time (s)")
plt.grid()
plt.show()
exit(0)