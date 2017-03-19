const router = require('express').Router()

router.get('/status', (req, res) => {
	res.json({
		alive : true,
		failedRouters : req.app.locals.failedRouters
	})
})

module.exports = router