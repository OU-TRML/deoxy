const express = require('express')

const fs = require('fs')
const path = require('path')

const bodyParser = require('body-parser')

const configureApp = (app, callback) => {

	app.use(bodyParser.urlencoded())
	app.use(bodyParser.json())
	let routerDirectory = path.join(__dirname, 'routes')

	return fs.readdir(routerDirectory, (err, files) => {
		if(err) { throw err }
		if(!files.length) {
			return callback(null, app)
		}
		let errors = []
		let failedRouters = []
		for(let i = 0; i < files.length; i++) {
			try {
				let router = require(path.join(routerDirectory, files[i]))
				// TODO: Verify that this is a valid Express router
				app.use(router)
			} catch (e) {
				let name = files[i]
				failedRouters.push(name)
				let error = new Error(`Failed to load router at routes/${name}.`)
				error.err = e
				error.error = e
				errors.push(error)
			}
		}
		app.locals.failedRouters = failedRouters
		return callback(errors.length ? errors[0] : null, app) // TODO: Return array of errors?
	})
}

module.exports = () => {
	let a = express()
	configureApp(a, (err, app) => {
		if(err) {
			return console.error(err) // TODO: Handle this better
		}
	})
	return a
}