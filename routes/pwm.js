const router = require('express').Router()
const Pin = require('../lib/pin')

router.get('/pwm', (req, res, next) => {
	let query = req.query || {}
	let target = new Pin(query.pin || 7)
	let pulseWidth = query.width || 1.5
	let frequency = query.frequency || 2
	let duration = query.duration || 5000
	target.doPWM(pulseWidth, frequency, duration).then(() => {
		res.json({
			width: pulseWidth,
			frequency: frequency,
			duration: duration
		})
	}).catch(next)
})

module.exports = router