"""
Code taken from https://pypi.org/project/PyLTSpice/
PyPi Documentation

pip install PyLTSpice
^^^^ Prerequisite ^^^^

"""
from PyLTSpice import SimRunner
from PyLTSpice import AscEditor

import ltspice
import matplotlib.pyplot as plt
import numpy as np
import os

"""
Changeable parameter for the simulation
MOST CHANGES SHOULD BE MADE IN LTSPICE

"""
netlist = AscEditor('./SimpleSim.asc')
# Change parameters of voltage source iteratively
voltRange = range(5,21) # +1 for the simulation
currNode = "V(Out)"
fileName = "SimpleSim"

"""
Default parameter for the simulation
"""
def processing_data(raw_filename, log_filename):
        '''This is a call back function that just prints the filenames'''
        print("Simulation Raw file is %s. The log is %s" % (raw_filename, log_filename))
# Force another simulatior
simulator = r"C:\Users\caleb\AppData\Local\Programs\ADI\LTspice\LTspice.exe"

# select spice model
LTC = SimRunner(output_folder='./temp', parallel_sims=8)


for supply_voltage in voltRange:
    netlist.set_component_value('V1', f"PULSE(0 {supply_voltage} 0 1n 1n 0.5m 1m 0)") 
    print("simulating circuit with pulse Voltage: ", supply_voltage)
    LTC.run(netlist, callback=processing_data)

# Sim Statistics
print('Successful/Total Simulations: ' + str(LTC.okSim) + '/' + str(LTC.runno))


for index in range(1,16):
    l = ltspice.Ltspice(os.path.dirname(__file__)+f'\\temp\{fileName}_{str(index)}.raw') 
    # Make sure that the .raw file is located in the correct path
    l.parse() 

    time = l.get_time()
    V_source = l.get_data(f'{currNode}')
    V_cap_max = []

    plt.plot(time, V_source) #label=f"{voltRange[index]}V")
    plt.xlabel("Time (s)")
    plt.ylabel(f"Voltage (V) at {currNode}")

   
voltRangeList = [*voltRange]
plt.legend(voltRangeList)
plt.title(f"Voltage Output at {currNode} (V) vs. Time (s)")
plt.grid()
plt.show()
LTC.file_cleanup()

exit(0)