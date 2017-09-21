const rpio = require('rpio')

// Eventually expose Pin(1, 2, 3).doThing() functionality, but not until this part is done

const PinState = Object.freeze({
	unknown: -1,
	closed: 0,
	open: 1
})

const Pin = class Pin {

	constructor(number) {
		this.number = number
		this.state = PinState.unknown
	}

	get isOpen() {
		return this.state == PinState.open
	}

	get isClosed() {
		return this.state == PinState.closed
	}

	open(force = false) {
		return new Promise((resolve, reject) => {
			if(this.isOpen && !force) return resolve(this)
			try {
				rpio.open(this.number, rpio.OUTPUT)
				this.state = PinState.open
				return resolve(this)
			} catch(e) {
				this.state = PinState.unknown
				return reject(e)
			}
		})
	}

	close(then, force = false) {
		return new Promise((resolve, reject) => {
			if(this.isClosed && !force) return resolve(this)
			if(then === undefined) then = rpio.PIN_RESET
			try {
				rpio.close(this.number, then)
				return resolve(this)
			} catch(e) {
				return reject(e)
			}
		})
	}

	write(value) {
		return new Promise((reject, resolve) => {
			return this.open().then(pin => {
				rpio.write(this.number, !!value ? rpio.HIGH : rpio.LOW)
				return pin
			})
			.then(pin => pin.close(rpio.PIN_PRESERVE))
			.then(resolve)
			.catch(reject)
		})
	}

	setHighFor(duration, perSecond = 1000) {
		return new Promise((reject, resolve) => {
			return this.open()
			.then(pin => {
				rpio.write(pin.number, rpio.HIGH)
				let ms = duration * 1000 / perSecond
				setTimeout(() => {
					pin.close().then(resolve).catch(reject)
				}, ms)
			})
			.catch(reject)
		})
	}

	doPWM(pulseWidth, frequency, duration) {
		return new Promise((resolve, reject) => {
			return this.open().then(pin => {
				let cycles = Math.ceil(frequency * duration / 1000)
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
				return pin
			}).then(pin => pin.close()).then(resolve).catch(reject)
		})
	}

	pwm(pulseWidth, frequency, duration) {
		return new Promise((resolve, reject) => {
			if(this.number !== 12) {
				return reject(new Error('Pin 12 is the only pin supporting hardware PWM on this board.'))
			}
			rpio.open(this.number, rpio.PWM)
			this.state = PinState.unknown
			rpio.pwmSetClockDivider(64)
			rpio.pwmSetRange(this.number, 1024)

			let cycles = Math.ceil(frequency * duration / 1000)
			duration = cycles / frequency * 1000 // ms
			let antiPulseWidth = (duration - cycles * pulseWidth) / cycles // Premature optimization is the root of all evil.
			if(antiPulseWidth <= 0) {
				throw new Error(`Using pulse width of ${pulseWidth} ms and frequency of ${frequency} Hz (${frequency / 1000} kHz) leaves no room for setting the pin low. Since this is almost certainly not what you wanted to do, aborting here.`)
			}
			for(let i = 0; i < cycles; i++) {
				rpio.pwmSetData(this.number, 1024)
				rpio.msleep(pulseWidth)
				rpio.write(this.number, 0)
				rpio.msleep(antiPulseWidth)
			}
			return resolve(this)
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