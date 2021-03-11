# embedded-playground

A small playground project where I explore embedded software development in Rust

## Hardware

* STM32F103C8T6 ("Blue pill")
* RobotDyn Matrix LED 8x8 module, MAX7219
* Generic 4x4 keypad
* DS18B20 temperature probe
* ST-Link/V2

## Connections

Power the breadboard by the host 5V (9) and GND (3) pins from the ST-Link.

* DIO - ST-link SWDIO
* DCLK - ST-link SWCLK
* PB12 - Row 0 keypad
* PB13 - Col 0 keypad
* PB14 - Col 1 keypad
* PB15 - Col 2 keypad
* PB7 - SDI matrix
* PB8 - CS matrix
* PB6 - SCL matrix
* PA15 - DQ temp. probe
