const chai = require('chai')
const should = chai.should()

const schedule = require('../lib/schedule')

describe('schedule', () => {
	describe('#scheduleEventsForTimes', () => {
		it('should abort and return an error when given different dimensions for times and events', (done) => {
			let foo = () => {}
			let bar = () => {}
			schedule.scheduleEventsForTimes([foo, bar], [new Date()], (err, jobs) => {
				should.exist(err)
				should.not.exist(jobs)
				done()
			})
		})
		it('should return the scheduled jobs as the second argument to the callback function', (done) => {
			let foo = () => {}
			let bar = () => {}
			schedule.scheduleEventsForTimes([foo, bar], [new Date(Date.now() + 1000), new Date(Date.now() + 2000)], (err, jobs) => {
				should.not.exist(err)
				should.exist(jobs)
				jobs.should.have.length(2)
				done()
			})
		})
	})
	describe('#scheduleEventForTime', () => {
		it('should return the scheduled job as the second argument to the callback function', (done) => {
			let foo = () => {}
			schedule.scheduleEventForTime(foo, new Date(Date.now() + 1000), (err, job) => {
				should.exist(job)
				done()
			})
		})
	})
	describe('#scheduleEventsForIntervals', () => {
		it('should abort and return an error when given different dimensions for times and events', (done) => {
			let foo = () => {}
			let bar = () => {}
			schedule.scheduleEventsForIntervals([foo, bar], [1000], (err, jobs) => {
				should.exist(err)
				should.not.exist(jobs)
				done()
			})
		})
		it('should return the scheduled jobs as the second argument to the callback function', (done) => {
			let foo = () => {}
			let bar = () => {}
			schedule.scheduleEventsForIntervals([foo, bar], [1000, 2000], (err, jobs) => {
				should.exist(jobs)
				should.not.exist(err)
				jobs.should.have.length(2)
				done()
			})
		})
	})
	describe('#scheduleEventForInterval', () => {
		it('should return the scheduled job as the second argument to the callback function', (done) => {
			let foo = () => {}
			schedule.scheduleEventForInterval(foo, 1000, (err, job) => {
				should.exist(job)
				should.not.exist(err)
				done()
			})
		})
	})
})