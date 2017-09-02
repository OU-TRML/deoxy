const router = require('express').Router()
const Pin = require('../lib/pin')

router.get('/pwm', (req, res, next) => {
	let target = new Pin(req.query.pin || 7)
	let pulseWidth = req.query.width || 1.5
	let frequency = req.query.frequency || 1/3000
	let duration = req.frequency.duration || 5000
	target.doPWM(pulseWidth, frequency, duration).then(() => {
		res.json({
			width: pulseWidth,
			frequency: frequency,
			duration: duration
		})
	}).catch(next)
})

module.exports = router