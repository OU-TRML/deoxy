var rpio

try {
	rpio = require('rpio')
} catch (e) {
	if(process.env.NODE_ENV === 'production') { throw e }
	console.error(e)
	console.log('Loading rpio module failed; loading stub instead.')
	rpio = {
		open : (pin, direction, defaultState) => {},
		write : (pin, state) => {},
		close : (pin) => {},
		HIGH : 1,
		LOW : 0,
		INPUT : 0,
		OUTPUT : 1
	}
}

const write = (pin, value) => {
	rpio.open(pin, rpio.OUTPUT)
	rpio.write(pin, value ? rpio.HIGH : rpio.LOW)
	rpio.close(pin, rpio.PIN_PRESERVE)
}

const setHighFor = (pin, ms) => {
	rpio.open(pin, rpio.OUTPUT)
	rpio.write(pin, rpio.HIGH)
	setTimeout(((pin) => {
		rpio.close(pin, rpio.PIN_RESET)
	}).bind(null, pin), ms)
}

const doPWM = (pin, pulseWidth, frequency, duration) => { // pulseWidth and duration in ms, frequency in Hz
	rpio.open(pin, rpio.OUTPUT)
	let cycles = Math.ceil(frequency * duration / 1000)
	duration = cycles / frequency
	let antiPulseWidth = (duration - cycles * pulseWidth) / cycles // Premature optimization is the root of all evil.
	for(let i = 0; i < cycles; i++) {
		rpio.write(pin, rpio.HIGH)
		rpio.msleep(pulseWidth)
		rpio.write(pin, rpio.LOW)
		rpio.msleep(antiPulseWidth)
	}
	rpio.close(pin, rpio.PIN_RESET)
}

module.exports = {
	write : write,
	setHighFor : setHighFor,
	doPWM : doPWM
}