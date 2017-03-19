var gpio 

try {
	gpio = require('pi-gpio')
} catch (e) {
	if(process.env.NODE_ENV === 'production') { throw e }
	console.error(e)
	console.log('Loading pi-gpio module failed; loading stub instead.')
	gpio = {
		open : (pin, cb) => cb(),
		write : (pin, value, cb) => cb(),
		close : (pin) => { }
	}
}

const write = (pin, value) => {
	gpio.open(pin, (err) => {
		if(err) { throw err }
		gpio.write(pin, !!value, () => gpio.close(pin))
	})
}

module.exports = {
	write: write,
	HIGH: 1,
	LOW: 0
}