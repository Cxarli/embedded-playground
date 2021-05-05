# embedded-playground

A (small) playground project where I explore embedded software development in Rust.

## Hardware

I'm not affiliated with these websites; I just think they offer neat stuff for a reasonable price.

* [STM32F103C8T6 ("Blue pill")](https://opencircuit.nl/Product/STM32-ARM-development-board-STM32F103C8T6)
* [ST-Link/V2](https://opencircuit.nl/Product/ST-Link-V2-STM8-STM32-programmer)
* [Logic Analyzer 24Mhz8CH](https://opencircuit.nl/Product/USB-Logic-Analyzer-8-kanaals)
* [DS18B20 temperature probe](https://opencircuit.nl/Product/DS18B20-Temperatuur-sensor-probe-1-meter)
* [RobotDyn Matrix LED 8x8 module, MAX7219](https://opencircuit.nl/Product/LED-Matrix-rood-8x8-module-32x32mm-MAX7219)
* [Generic 4x4 keypad](https://opencircuit.nl/Product/4-x-4-keypad-paneel)
* [16x2 LCD display](https://opencircuit.nl/Product/16x2-Karakters-lcd-module-blauw-5V)
* [I2C-LCD interface](https://opencircuit.nl/Product/I2C-LCD-interface-module)
* [BH1750FVI light sensor](https://opencircuit.nl/Product/BH1750FVI-Digitale-licht-sensor-module-GY-302)
* [DHT11 humidity and temperature sensor](https://opencircuit.nl/Product/DHT11-Luchtvochtigheid-temperatuur-sensor)
* [PCF8591 AD / DA converter with photoresistor, thermistor and potentiometer](https://opencircuit.nl/Product/PCF8591-AD-DA-Converter-module)

* [Pinecil](https://pine64.com/product/pinecil-smart-mini-portable-soldering-iron/)

## Setup

```sh
rustup default nightly
rustup target add thumbv7m-none-eabi
paru -S gdb-multiarch openocd

# these are optional and only used if you
# have the signal analyzer
paru -S pulseview sigrok-firmware-fx2lafw
```

## Run

```sh
# Make sure you have OpenOCD running in another terminal
~/embedded-playground $ sudo openocd

# Run the project
~/embedded-playground $ cargo run
```

## Connections

Power the breadboard by the host 5V (9) and GND (3) pins from the ST-Link.
Unless explicitly stated otherwise, connect all GND and VCC pins from the
modules to the GND and VCC lines of the breadboard.

Some of these are chosen arbitrarily, some are chosen because they require
for example I2C pins. Since everything is Rust, feel free to move pins.
The compiler will throw errors if you chose the wrong pins. Also see the
[pinout](https://opencircuit.shop/resources/content/ad8542259ac19/crop/1900-950/STM32-ARM-development-board-STM32F103C8T6.webp)
for a better overview of what possibilities there are.

Keep in mind I generally prefer to choose 5V lines over 3V ones.

### ST-Link

**Watch out**: the pins don't align: `STLv2:DGCV <-> STM32F1xx:GCDV`

* DIO - ST-link SWDIO
* DCLK - ST-link SWCLK

### Logic Analyzer

* PB12 - Ch0 : shared with temperature probe

### Numpad

* PA15 - Row 0
* PB3 - Col 0
* PB4 - Col 1
* PB5 - Col 2  (TODO: this is a 3V line, change to a 5V one)

### Matrix

* PB7 - SDI matrix
* PB8 - CS matrix (TODO: consider using a different pin to free the PB8/PB9 I2C pair)
* PB6 - SCL matrix

### Temperature probe

* PB12 - DQ temp. probe
