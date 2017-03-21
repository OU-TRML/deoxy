var rpio

try {
	rpio = require('rpio')
} catch (e) {
	if(process.env.NODE_ENV === 'production') { throw e }
	console.error(e)
	console.log('Loading pi-gpio module failed; loading stub instead.')
	rpio = {
		open : (pin, direction, defaultState) => {},
		write : (pin, state) => {},
		close : (pin) => {}
	}
}

const write = (pin, value) => {
	rpio.open(pin, rpio.OUTPUT)
	rpio.write(pin, !!value)
	rpio.close(pin, rpio.PIN_PRESERVE)
}

module.exports = {
	write: write,
	HIGH: 1,
	LOW: 0
}