const rpio = require('rpio')

// Eventually expose Pin(1, 2, 3).doThing() functionality, but not until this part is done

const Pin = class Pin {

	constructor(number) {
		this.number = number
	}

	write(value) {
		return new Promise((resolve, reject) => {
			try {
				rpio.open(this.number, rpio.OUTPUT)
				rpio.write(this.number, !!value ? rpio.HIGH : rpio.LOW)
				rpio.close(this.number, rpio.PIN_PRESERVE)
				return resolve(this)
			} catch(e) {
				return reject(e)
			}
		})
	}

	setHighFor(duration, perSecond = 1000) {
		return new Promise((resolve, reject) => {
			try {
				rpio.open(this.number, rpio.OUTPUT)
				rpio.write(this.number, rpio.HIGH)
				let ms = duration * perSecond / 1000
				setTimeout((resolve, reject) => {
					try {
						rpio.close(this.number, rpio.PIN_RESET)
						return resolve(this)
					} catch(e) {
						return reject(e)
					}
				}, ms)
			} catch(e) {
				return reject(e)
			}
		})
	}

	doPWM(pulseWidth, frequency, duration) {
		return new Promise((resolve, reject) => {
			try {
				rpio.open(this.number, rpio.OUTPUT)
				let cycles = Math.ceil(frequency * duration * perSecond / 1000)
				duration = cycles / frequency * 1000 // ms
				let antiPulseWidth = (duration - cycles * pulseWidth) / cycles // Premature optimization is the root of all evil.
				if(antiPulseWidth <= 0) {
					throw new Error(`Using pulse width of ${pulseWidth} ms and frequency of ${frequency} Hz (${frequency / 1000} kHz) leaves no room for setting the pin low. Since this is almost certainly not what you wanted to do, aborting here.`)
				}
				for(let i = 0; i < cycles; i++) {
					rpio.write(this.number, rpio.HIGH)
					rpio.msleep(pulseWidth)
					rpio.write(this.number, rpio.LOW)
					rpio.msleep(antiPulseWidth)
				}
				rpio.close(this.number, rpio.PIN_RESET)
				return resolve(this)
			} catch(e) {
				return reject(e)
			}
		})
	}

	static write(pin, value) {
		return new Pin(pin).write(value)
	}

	static doPWM(pin, pulseWidth, frequency, duration) {
		return new Pin(pin).doPWM(pulseWidth, frequency, duration)
	}

	static setHighFor(pin, duration) {
		return new Pin(pin).setHighFor(duration)
	}

}

module.exports = Pin