use std::time::Duration;

/// Encodes the system configuration.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "use_serde", derive(Deserialize, Serialize))]
pub struct Config {
    /// The pump configuration.
    pub pump: PumpConfig,
    /// The motor configurations.
    pub motors: Vec<MotorConfig>,
    /// The administrative users of the machine.
    #[cfg_attr(feature = "use_serde", serde(default))]
    pub admins: Vec<String>,
}

/// Specifies a single motor.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "use_serde", derive(Deserialize, Serialize))]
pub struct MotorConfig {
    /// The pin associated with this motor.
    pub pin: u16,
    /// An optional label for the motor (perhaps the buffer associated with it?).
    #[cfg_attr(feature = "use_serde", serde(skip_serializing_if = "Option::is_none"))]
    pub label: Option<String>,
    /// The characteristic period of the motor.
    pub period: Duration,
    /// The limits of acceptable signal length.
    pub range: [Duration; 2],
}

/// Encodes the pump configuration.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "use_serde", derive(Deserialize, Serialize))]
pub struct PumpConfig {
    /// The pins used for the pump, in order from 0–3.
    pub pins: [u16; 4],
    /// If true, the pump's "forward" direction will be the reverse direction
    #[cfg_attr(feature = "use_serde", serde(default, alias = "reverse"))]
    pub invert: bool,
}
