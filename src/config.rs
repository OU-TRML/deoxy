use std::time::Duration;

/// Encodes the system configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// The pump configuration.
    pub pump: PumpConfig,
    /// The motor configurations.
    pub motors: Vec<MotorConfig>,
}

/// Specifies a single motor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MotorConfig {
    /// The pin associated with this motor.
    pub pin: u16,
    /// An optional label for the motor (perhaps the buffer associated with it?).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The characteristic period of the motor.
    pub period: Duration,
    /// The limits of acceptable signal length.
    pub range: [Duration; 2],
}

/// Encodes the pump configuration.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PumpConfig {
    /// The pins used for the pump, in order from 0–3.
    pub pins: [u16; 4],
}
