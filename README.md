# Opengyro

A fly by wire system for R/C planes.

Designed for an Adafruit Feather M4 connected to a custom featherwing.

## Features

### TODO (order of priority):
 - Failsafe mode
 - Communication with gyro
 - PID controller implementation (should be pretty easy)
 - Angular rate control mode
 - Angular position control mode
 - Method to switch between control modes

### In Progress
 - Robust communication with reciever
   - I am using [this OrangeRx DSMX/DSM2 reciever](https://hobbyking.com/en_us/orangerx-r110xl-dsmx-dsm2-compatible-satellite-receiver.html)
   - Protocol specification: https://spektrumrc.com/ProdInfo/Files/Remote%20Receiver%20Interfacing%20Rev%20A.pdf

### Done
 - Pass through control mode (i.e. fly-by-wire disabled, control inputs directly correspond to servo positions)
 - Flaky communication with reciever
   - I have implemented the protocol, but I have not had much luck reliably detecting the start of a packet, so my impementation is not 100% flight-worthy 
