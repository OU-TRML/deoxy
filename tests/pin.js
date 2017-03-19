const chai = require('chai')
const should = chai.should()

const pin = require('../lib/pin')

describe('pin', () => {
	describe('#write', () => {
		it('should take two arguments: the pin number and the target value', (done) => {
			pin.write.length.should.equal(2)
			done()
		})
	})
	it('should export HIGH and LOW constants (equal to 1 and 0, respectively)', (done) => {
		pin.HIGH.should.equal(1)
		pin.LOW.should.equal(0)
		done()
	})
})