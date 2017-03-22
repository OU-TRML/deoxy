const router = require('express').Router()
const Pin = require('../lib/pin')

router.get('/', (req, res) => {
	res.render('light-up', {
		pretty: true
	})
})

router.post('/', (req, res) => {
	let duration = req.body.duration
	let states = [!!req.body.L1, !!req.body.L2, !!req.body.L3]
	let pins = [11, 13, 15]
	console.log(`Applying states (${states}) to pins (${pins})${duration ? (" for duration " + duration + " ms") : ""}.`)
	let Pin = require('../lib/pin')
	if(duration) {
		for(let i = 0; i < pins.length; i++) {
			if(states[i]) {
				Pin.setHighFor(pins[i], duration)
			}
		}
	} else {
		for(let i = 0; i < pins.length; i++) {
			Pin.write(pins[i], states[i])
		}
	}
	res.redirect('/')
})

module.exports = router