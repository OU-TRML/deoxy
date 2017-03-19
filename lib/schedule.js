const schedule = require('node-schedule')

var exports = {}

exports.scheduleEventsForTimes = (events, times, callback) => {
	let length = events.length
	if(times.length != length) {
		return callback(new Error(`Nonsensical mismatching dimensions supplied in arguments to scheduleEventsForTimes (${length} and ${times.length}).`))
	}
	let jobs = []
	for(let i = 0; i < length; i++) {
		let time = times[i]
		let event = events[i]
		// TODO: Sanitize times and events
		let job = schedule.scheduleJob(time, event)
		jobs.push(job)
	}
	return callback(null, jobs)
}

exports.scheduleEventForTime = (event, time, callback) => exports.scheduleEventsForTimes([event], [time], callback)

exports.scheduleEventsForIntervals = (events, intervals, callback) => {
	let now = Date.now()
	let times = []
	for (let i = 0; i < intervals.length; i++) {
		let interval = intervals[i]
		let target = now + interval
		times.push(new Date(target))
	}
	return exports.scheduleEventsForTimes(events, times, callback)
}

exports.scheduleEventForInterval = (event, interval, callback) => exports.scheduleEventsForIntervals([event], [interval], callback)

module.exports = exports