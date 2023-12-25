# Opengyro

A fly by wire system for RC planes. TODO: Think of a better name.

Designed for an Adafruit Feather M4 connected to a [custom featherwing](https://github.com/MarkGhebrial/feather-fc).

## Features

### TODO (in approximate order of priority):
 - Failsafe mode
 - PID controller implementation (should be pretty easy)
 - Angular rate control mode
   - Use the raw gyro data for this? I don't know anything about sensor filtering, but dRehmFlight uses a Madgwick filter so I'll prabably do the same. The `ahrs` crate implements the Madgwick filter, so I'll probably use that.
 - Angular position control mode
   - A filter of some kind would definitely be required.
 - Flight envelope control mode
   - This would prevent the pilot's inputs from causing the plane to leave its flight envelope (i.e. no up elevator if the plane is pitched up more than 15 degrees).
 - Method to switch between control modes
   - A switch on the transmitter would toggle between passthru mode and fly-by-wire.
   - The button on the flight computer would switch between the different fly-by-wire modes.

### In Progress
 - Communication with IMU
   - I'm using my own fork of the `icm20948_driver` crate. TODO: Consider using the marginally more mature `icm20948` crate.
   - Oddly, the I2C peripheral seems to hang the microcontroller when attempting to communicate with an address that has no device connected. As a result, my code hangs when I unplug the IMU, which means that the plane would crash if the IMU gets unplugged during flight (the appropriate behavior would be to fallback to passthru mode).
   - TODO: Add a trait so that my code can be generic over different IMUs or drivers.

### Done
 - Pass through control mode (i.e. fly-by-wire disabled, control inputs directly correspond to servo positions)
 - "Robust" communication with reciever
   - I am using [this OrangeRx DSMX/DSM2 reciever](https://hobbyking.com/en_us/orangerx-r110xl-dsmx-dsm2-compatible-satellite-receiver.html)
   - Protocol specification: https://spektrumrc.com/ProdInfo/Files/Remote%20Receiver%20Interfacing%20Rev%20A.pdf
   - I'm still not convinced that my implementation is entirely reliable. Since I am using DMA, we recieve the UART data in chunks instead of byte-by-byte. We need byte-by-byte data in order to reliably tell when a new packet has arrived (since the start and stop of a packet is indicated by a 11 or 22ms gap in transmission). If the DMA buffer starts reading data mid-packet, then the period between DMA transactions will still be 11 or 22ms, but instead of each one containing a single packet, they will contain the end of one packet and the beginning of the next.
     - This should only be a problem when the microcontroller boots. If the UART begins recieving data during the transmission of a packet (instead of during the time between packets), then we will not receive DMA transactions that are "aligned" with the start and end of each packet.
       - If the microcontroller somehow resets while in flight, then there is a chance that it will encounter this problem and crash the plane.
       - This bug also means that, opposite to the usual advice, it is better to turn on the reciever before the transmitter.

Whoops, I accidentally made a kanban board in the README.

## Long-term goals
 - Hardware agnostic (i.e. adding support for a different microcontroller would just be a matter of implementing some hal traits)
 - Return to home mode using a gps
 - Waypoint missions using a gps
 - Autolanding using an IR camera and IR leds on the runway
   - Of course the poor ATSAMD51 can't handle any computer vision algrithms, so a Raspberry Pi would be needed.