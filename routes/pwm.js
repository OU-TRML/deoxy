const router = require('express').Router()
const Pin = require('../lib/pin')

router.get('/pwm', (req, res, next) => {
	let query = req.query || {}
	let target = new Pin(query.pin ? parseInt(query.pin) : 7)
	let pulseWidth = query.width ? parseFloat(query.width) : 1.5
	let frequency = query.frequency ? parseInt(query.frequency) : 50
	let duration = query.duration ? parseInt(query.duration) : 5000
	target.doPWM(pulseWidth, frequency, duration).then(() => {
		res.json({
			width: pulseWidth,
			frequency: frequency,
			duration: duration
		})
	}).catch(next)
})

module.exports = router