const express = require('express')

const router = express.Router()

router.use((req, res, next) =>  res.json({ failedRouters: req.app.locals.failedRouters })) // 404 handler. TODO: Implement properly

module.exports = router