// Modified from Pollen Robotics AmazingHand (Apache-2.0).
// Local changes add feedback timeout, temperature, and voltage safety checks.

use crate::config::SoftGraspConfig;
use crate::feedback::ServoFeedback;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FaultReason {
    Overload,
    OverTemperature,
    VoltageOutOfRange,
    FeedbackTimeout,
    Communication,
}

pub fn feedback_fault(
    feedback: &[ServoFeedback],
    config: &SoftGraspConfig,
    now: Instant,
) -> Option<FaultReason> {
    if feedback.iter().any(|f| {
        !f.valid
            || now.duration_since(f.timestamp)
                > Duration::from_millis(config.communication_timeout_ms)
    }) {
        return Some(FaultReason::FeedbackTimeout);
    }
    if feedback
        .iter()
        .any(|f| f.temperature_c > config.max_temperature_c)
    {
        return Some(FaultReason::OverTemperature);
    }
    if feedback
        .iter()
        .any(|f| f.voltage_v < config.min_voltage_v || f.voltage_v > config.max_voltage_v)
    {
        return Some(FaultReason::VoltageOutOfRange);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn feedback() -> ServoFeedback {
        ServoFeedback {
            id: 1,
            position_rad: 0.0,
            speed: 0.0,
            raw_load: 0.0,
            filtered_load: 0.0,
            temperature_c: 25.0,
            voltage_v: 6.0,
            timestamp: Instant::now(),
            valid: true,
        }
    }

    #[test]
    fn temperature_faults() {
        let mut feedback = feedback();
        feedback.temperature_c = 61.0;
        assert_eq!(
            feedback_fault(&[feedback], &SoftGraspConfig::default(), Instant::now()),
            Some(FaultReason::OverTemperature)
        );
    }

    #[test]
    fn stale_feedback_faults() {
        let mut feedback = feedback();
        feedback.timestamp -= Duration::from_millis(201);
        assert_eq!(
            feedback_fault(&[feedback], &SoftGraspConfig::default(), Instant::now()),
            Some(FaultReason::FeedbackTimeout)
        );
    }

    #[test]
    fn voltage_faults() {
        let mut feedback = feedback();
        feedback.voltage_v = 4.0;
        assert_eq!(
            feedback_fault(&[feedback], &SoftGraspConfig::default(), Instant::now()),
            Some(FaultReason::VoltageOutOfRange)
        );
    }
}
