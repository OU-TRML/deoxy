const router = require('express').Router()
const Pin = require('../lib/pin')

router.get('/', (req, res) => {
	res.render('light-up', {
		pretty: true
	})
})

router.post('/', (req, res) => {
	let states = [!!req.body.L1, !!req.body.L2, !!req.body.L3]
	let pins = [11, 13, 15]
	console.log(`Applying states (${states}) to pins (${pins}).`)
	let Pin = require('../lib/pin')
	for(let i = 0; i < pins.length; i++) {
		Pin.write(pins[i], states[i])
	}
	res.redirect('/')
})

module.exports = router