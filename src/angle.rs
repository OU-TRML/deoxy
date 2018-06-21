use std::f64::consts::PI;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

/// An angle.
///
/// The underlying storage type for angles is `f64`. Angles are stored in degrees.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Angle {
    /// The angle's measure, in degrees.
    measure: f64,
}

#[inline(always)]
fn rad_from_deg(rad: f64) -> f64 {
    rad * PI / 180.0
}

#[inline(always)]
fn deg_from_rad(rad: f64) -> f64 {
    rad * 180.0 / PI
}

impl Angle {
    /// Creates a new angle with the given measure (in degrees; for radians, see
    /// [`with_rad_measure`](#method.with_rad_measure)).
    ///
    /// # Examples
    /// ```
    /// # extern crate deoxy;
    /// # use deoxy::motion::Angle;
    /// let eighth = 22.5;
    /// let angle = Angle::with_measure(eighth);
    /// assert_eq!(angle.measure(), eighth);
    /// ```
    pub fn with_measure(measure: f64) -> Self {
        Self { measure }
    }
    /// Creates a new angle with the given measure (in radians).
    ///
    /// For degrees, see [`with_measure`](#method.with_measure).
    ///
    /// # Examples
    /// ```
    /// # extern crate deoxy;
    /// # use deoxy::motion::Angle;
    /// use std::f64::consts::PI;
    /// let half = PI;
    /// let angle = Angle::with_rad_measure(half);
    /// assert_eq!(angle.measure(), 180.0);
    /// ```
    pub fn with_rad_measure(rad: f64) -> Self {
        Self {
            measure: deg_from_rad(rad),
        }
    }
    /// Returns the measure of this angle (in degrees; for radians, see
    /// [`rad_measure`](#method.rad_measure)).
    pub fn measure(self) -> f64 {
        self.measure
    }
    /// Returns the measure of this angle in radians.
    ///
    /// For degrees, see [`measure`](#method.measure).
    ///
    /// # Examples
    /// ```
    /// # extern crate deoxy;
    /// # use deoxy::motion::Angle;
    /// use std::f64::consts::PI;
    /// let unit = Angle::with_measure(360.0);
    /// assert_eq!(unit.rad_measure(), 2.0 * PI);
    /// ```
    pub fn rad_measure(self) -> f64 {
        rad_from_deg(self.measure)
    }

    /// Returns an angle with identically zero measure.
    /// # Examples
    /// ```
    /// # extern crate deoxy;
    /// # use deoxy::motion::Angle;
    /// let zero = Angle::zero();
    /// assert_eq!(zero.measure(), 0.0);
    /// assert_eq!(zero.rad_measure(), 0.0);
    /// ```
    pub fn zero() -> Self {
        Self::with_measure(0.0)
    }
}

impl Default for Angle {
    fn default() -> Self {
        Self::with_measure(0.0)
    }
}

impl Sub<Angle> for Angle {
    type Output = Angle;
    fn sub(self, other: Angle) -> Self::Output {
        Self::with_measure(self.measure() - other.measure())
    }
}

impl Add<Angle> for Angle {
    type Output = Angle;
    fn add(self, other: Angle) -> Self::Output {
        Self::with_measure(self.measure() + other.measure())
    }
}

impl Div<Angle> for Angle {
    type Output = f64;
    fn div(self, other: Angle) -> Self::Output {
        self.measure() / other.measure()
    }
}

impl Mul<f64> for Angle {
    type Output = Angle;
    fn mul(self, multiplier: f64) -> Self::Output {
        Self::with_measure(self.measure() * multiplier)
    }
}

impl Mul<Angle> for f64 {
    type Output = Angle;
    fn mul(self, angle: Angle) -> Self::Output {
        angle.mul(self)
    }
}

impl fmt::Display for Angle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}°", self.measure())
    }
}
