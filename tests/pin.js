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
})