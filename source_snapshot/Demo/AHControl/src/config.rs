// Modified from Pollen Robotics AmazingHand (Apache-2.0).
// Local changes add serde configuration, position limits, and soft-grasp settings.

use serde::Deserialize;
use std::{fs, path::Path};

const DEFAULT_MIN_POSITION_RAD: f64 = -std::f64::consts::FRAC_PI_2;
const DEFAULT_MAX_POSITION_RAD: f64 = std::f64::consts::FRAC_PI_2;

#[derive(Clone, Debug)]
pub struct HandConfig {
    pub fingers: FingersConfig,
    pub soft_grasp: SoftGraspConfig,
    pub shutdown_action: ShutdownAction,
}

#[derive(Deserialize)]
struct RawHandConfig {
    #[serde(rename = "Fingers", default)]
    fingers: Option<FingersConfig>,
    #[serde(default)]
    motors: Vec<FingerConfig>,
    #[serde(default)]
    soft_grasp: SoftGraspConfig,
    #[serde(default)]
    shutdown_action: ShutdownAction,
}

impl<'de> Deserialize<'de> for HandConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = RawHandConfig::deserialize(deserializer)?;
        // The historic facet TOML parser accepted `[Fingers]` plus root `[[motors]]`.
        // Also accept standard serde TOML's `[[Fingers.motors]]` spelling.
        let fingers = match raw.fingers {
            Some(fingers) if !fingers.motors.is_empty() => fingers,
            _ => FingersConfig { motors: raw.motors },
        };
        Ok(Self {
            fingers,
            soft_grasp: raw.soft_grasp,
            shutdown_action: raw.shutdown_action,
        })
    }
}

impl HandConfig {
    pub fn from_path(path: impl AsRef<Path>) -> eyre::Result<Self> {
        Ok(toml::from_str(&fs::read_to_string(path)?)?)
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FingersConfig {
    #[serde(default)]
    pub motors: Vec<FingerConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FingerConfig {
    pub finger_name: String,
    pub motor1: MotorConfig,
    pub motor2: MotorConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MotorConfig {
    pub id: u8,
    pub offset: f64,
    #[serde(default)]
    pub invert: bool,
    pub model: String,
    #[serde(default = "default_min_position_rad")]
    pub min_position_rad: f64,
    #[serde(default = "default_max_position_rad")]
    pub max_position_rad: f64,
}

impl MotorConfig {
    pub fn command_position(&self, model_position: f64) -> f64 {
        let position =
            (model_position + self.offset).clamp(self.min_position_rad, self.max_position_rad);
        if self.invert {
            -position
        } else {
            position
        }
    }
}

fn default_min_position_rad() -> f64 {
    DEFAULT_MIN_POSITION_RAD
}
fn default_max_position_rad() -> f64 {
    DEFAULT_MAX_POSITION_RAD
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShutdownAction {
    #[default]
    Hold,
    Open,
    TorqueOff,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FaultAction {
    #[default]
    Hold,
    Backoff,
    TorqueOff,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct SoftGraspConfig {
    pub enabled: bool,
    pub feedback_hz: u32,
    pub command_hz: u32,
    pub goal_speed_rad_s: f64,
    pub load_filter_alpha: f64,
    pub contact_confirm_samples: u32,
    pub release_confirm_samples: u32,
    pub max_position_step_rad: f64,
    pub hold_deadband: f64,
    pub communication_timeout_ms: u64,
    pub baseline_samples: u32,
    pub max_temperature_c: f64,
    pub min_voltage_v: f64,
    pub max_voltage_v: f64,
    pub voltage_scale_v_per_raw: f64,
    pub fault_action: FaultAction,
    pub fault_backoff_rad: f64,
    pub fingers: Vec<SoftGraspFingerConfig>,
}

impl Default for SoftGraspConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            feedback_hz: 10,
            command_hz: 100,
            goal_speed_rad_s: 0.6,
            load_filter_alpha: 0.2,
            contact_confirm_samples: 5,
            release_confirm_samples: 3,
            max_position_step_rad: 0.01,
            hold_deadband: 20.0,
            communication_timeout_ms: 200,
            baseline_samples: 25,
            max_temperature_c: 60.0,
            min_voltage_v: 4.5,
            max_voltage_v: 7.2,
            voltage_scale_v_per_raw: 0.1,
            fault_action: FaultAction::Hold,
            fault_backoff_rad: 0.02,
            fingers: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct SoftGraspFingerConfig {
    pub name: String,
    pub contact_load_threshold: f64,
    pub target_hold_load: f64,
    pub max_load: f64,
    #[serde(default = "default_closing_sign_motor1")]
    pub closing_sign_motor1: f64,
    #[serde(default = "default_closing_sign_motor2")]
    pub closing_sign_motor2: f64,
}
fn default_closing_sign_motor1() -> f64 {
    1.0
}
fn default_closing_sign_motor2() -> f64 {
    -1.0
}

impl SoftGraspConfig {
    pub fn finger(&self, name: &str) -> Option<&SoftGraspFingerConfig> {
        self.fingers.iter().find(|finger| finger.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_config_without_soft_grasp_uses_position_tracking() {
        let config: HandConfig = toml::from_str("[Fingers]\n[[Fingers.motors]]\nfinger_name='finger1'\nmotor1={ id=1, offset=0.1, model='SCS0009' }\nmotor2={ id=2, offset=0.0, model='SCS0009' }\n").unwrap();
        assert!(!config.soft_grasp.enabled);
        assert!((config.fingers.motors[0].motor1.command_position(0.2) - 0.3).abs() < 1e-12);
    }

    #[test]
    fn inversion_keeps_existing_offset_semantics() {
        let motor = MotorConfig {
            id: 1,
            offset: 0.1,
            invert: true,
            model: "SCS0009".into(),
            min_position_rad: -1.0,
            max_position_rad: 1.0,
        };
        assert!((motor.command_position(0.2) + 0.3).abs() < 1e-12);
    }

    #[test]
    fn bundled_right_hand_config_parses() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("config/r_hand.toml");
        let config = HandConfig::from_path(path).unwrap();
        assert_eq!(config.fingers.motors.len(), 4);
        assert_eq!(config.soft_grasp.fingers.len(), 4);
    }
}
