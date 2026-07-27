// Modified from Pollen Robotics AmazingHand (Apache-2.0).
// Local changes implement load-baseline compensation and the soft-grasp state machine.

use crate::{
    config::{SoftGraspConfig, SoftGraspFingerConfig},
    feedback::ServoFeedback,
    safety::FaultReason,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraspState {
    Tracking,
    Closing,
    Contact,
    Holding,
    Releasing,
    Fault,
}

#[derive(Clone, Debug)]
pub struct FingerDecision {
    pub targets: [f64; 2],
    pub state: GraspState,
    pub contact_detected: bool,
    pub overload: bool,
    pub fault_reason: Option<FaultReason>,
}

#[derive(Clone, Debug)]
pub struct SoftGraspController {
    config: SoftGraspConfig,
    fingers: Vec<FingerRuntime>,
}

#[derive(Clone, Debug)]
struct FingerRuntime {
    state: GraspState,
    previous_target: Option<[f64; 2]>,
    hold_target: Option<[f64; 2]>,
    filtered_loads: [f64; 2],
    filtered_excess_loads: [f64; 2],
    baseline_raw_loads: [f64; 2],
    baseline_samples: u32,
    last_filtered_feedback_at: Option<std::time::Instant>,
    contact_samples: u32,
    release_samples: u32,
    fault_reason: Option<FaultReason>,
}

impl SoftGraspController {
    pub fn new(config: SoftGraspConfig, names: impl IntoIterator<Item = String>) -> Self {
        Self {
            config,
            fingers: names
                .into_iter()
                .map(|_| FingerRuntime {
                    state: GraspState::Tracking,
                    previous_target: None,
                    hold_target: None,
                    filtered_loads: [0.0; 2],
                    filtered_excess_loads: [0.0; 2],
                    baseline_raw_loads: [0.0; 2],
                    baseline_samples: 0,
                    last_filtered_feedback_at: None,
                    contact_samples: 0,
                    release_samples: 0,
                    fault_reason: None,
                })
                .collect(),
        }
    }

    pub fn set_initial_targets(&mut self, targets: impl IntoIterator<Item = [f64; 2]>) {
        for (runtime, target) in self.fingers.iter_mut().zip(targets) {
            runtime.previous_target = Some(target);
        }
    }

    pub fn update(
        &mut self,
        index: usize,
        finger: &SoftGraspFingerConfig,
        requested: [f64; 2],
        feedback: [&ServoFeedback; 2],
        external_fault: Option<FaultReason>,
    ) -> FingerDecision {
        let runtime = &mut self.fingers[index];
        // motor2 is refreshed after motor1 by the round-robin reader. Filter each
        // physical feedback pair once, not repeatedly at the faster command rate.
        if runtime.last_filtered_feedback_at != Some(feedback[1].timestamp) {
            for (slot, source) in runtime.filtered_loads.iter_mut().zip(feedback) {
                *slot = self.config.load_filter_alpha * source.raw_load.abs()
                    + (1.0 - self.config.load_filter_alpha) * *slot;
            }
            for ((slot, baseline), source) in runtime
                .filtered_excess_loads
                .iter_mut()
                .zip(runtime.baseline_raw_loads)
                .zip(feedback)
            {
                let excess = (source.raw_load.abs() - baseline).max(0.0);
                *slot = self.config.load_filter_alpha * excess
                    + (1.0 - self.config.load_filter_alpha) * *slot;
            }
            runtime.last_filtered_feedback_at = Some(feedback[1].timestamp);
        }
        let estimated_load = runtime.filtered_excess_loads[0].max(runtime.filtered_excess_loads[1]);
        let overload = estimated_load >= finger.max_load;
        if let Some(reason) = external_fault.or(overload.then_some(FaultReason::Overload)) {
            runtime.state = GraspState::Fault;
            runtime.fault_reason = Some(reason);
        }
        let previous = runtime.previous_target.unwrap_or(requested);
        let delta = [requested[0] - previous[0], requested[1] - previous[1]];
        let closing_progress =
            (delta[0] * finger.closing_sign_motor1 + delta[1] * finger.closing_sign_motor2) / 2.0;
        let opening = closing_progress < -self.config.max_position_step_rad / 2.0;
        if opening && runtime.state != GraspState::Fault {
            runtime.release_samples += 1;
            // Releasing is deliberately immediate: contact locking must never trap an operator's opening command.
            runtime.state = GraspState::Releasing;
            runtime.hold_target = None;
        } else {
            runtime.release_samples = 0;
        }

        let mut targets = requested;
        if closing_progress > 0.0 {
            targets = [
                requested[0].clamp(
                    previous[0] - self.config.max_position_step_rad,
                    previous[0] + self.config.max_position_step_rad,
                ),
                requested[1].clamp(
                    previous[1] - self.config.max_position_step_rad,
                    previous[1] + self.config.max_position_step_rad,
                ),
            ];
        }
        if runtime.state != GraspState::Fault
            && !opening
            && estimated_load >= finger.contact_load_threshold
        {
            runtime.contact_samples += 1;
            if runtime.contact_samples >= self.config.contact_confirm_samples {
                runtime.state = GraspState::Contact;
                runtime.hold_target = Some(previous);
            }
        } else if !opening {
            runtime.contact_samples = 0;
        }

        match runtime.state {
            GraspState::Contact => {
                runtime.state = GraspState::Holding;
                targets = runtime.hold_target.unwrap_or(previous);
            }
            GraspState::Holding if !opening => {
                targets = runtime.hold_target.unwrap_or(previous);
                if estimated_load < finger.target_hold_load - self.config.hold_deadband {
                    targets[0] +=
                        finger.closing_sign_motor1 * self.config.max_position_step_rad.min(0.002);
                    targets[1] +=
                        finger.closing_sign_motor2 * self.config.max_position_step_rad.min(0.002);
                    runtime.hold_target = Some(targets);
                }
            }
            GraspState::Fault => {
                targets = runtime.hold_target.unwrap_or(previous);
            }
            GraspState::Tracking | GraspState::Releasing if closing_progress > 0.0 => {
                runtime.state = GraspState::Closing
            }
            GraspState::Releasing if !opening => runtime.state = GraspState::Tracking,
            _ => {}
        }
        runtime.previous_target = Some(targets);
        FingerDecision {
            targets,
            state: runtime.state,
            contact_detected: matches!(runtime.state, GraspState::Contact | GraspState::Holding),
            overload,
            fault_reason: runtime.fault_reason.clone(),
        }
    }

    pub fn filtered_loads(&self, index: usize) -> [f64; 2] {
        self.fingers[index].filtered_loads
    }

    pub fn observe_baseline(&mut self, index: usize, feedback: [&ServoFeedback; 2]) {
        let runtime = &mut self.fingers[index];
        if runtime.baseline_samples >= self.config.baseline_samples {
            return;
        }
        let count = runtime.baseline_samples as f64;
        for (slot, source) in runtime.baseline_raw_loads.iter_mut().zip(feedback) {
            *slot = (*slot * count + source.raw_load.abs()) / (count + 1.0);
        }
        runtime.baseline_samples += 1;
    }

    pub fn baseline_ready(&self) -> bool {
        self.fingers
            .iter()
            .all(|finger| finger.baseline_samples >= self.config.baseline_samples)
    }

    pub fn baseline_loads(&self, index: usize) -> [f64; 2] {
        self.fingers[index].baseline_raw_loads
    }

    pub fn estimated_loads(&self, index: usize) -> [f64; 2] {
        self.fingers[index].filtered_excess_loads
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FaultAction, SoftGraspConfig};
    use std::time::Instant;
    fn config() -> SoftGraspConfig {
        SoftGraspConfig {
            contact_confirm_samples: 3,
            max_position_step_rad: 0.02,
            hold_deadband: 20.0,
            fault_action: FaultAction::Hold,
            ..Default::default()
        }
    }
    fn finger() -> SoftGraspFingerConfig {
        SoftGraspFingerConfig {
            name: "finger1".into(),
            contact_load_threshold: 100.0,
            target_hold_load: 140.0,
            max_load: 300.0,
            closing_sign_motor1: 1.0,
            closing_sign_motor2: -1.0,
        }
    }
    fn fb(load: f64) -> ServoFeedback {
        ServoFeedback {
            id: 1,
            position_rad: 0.0,
            speed: 0.0,
            raw_load: load,
            filtered_load: 0.0,
            temperature_c: 25.0,
            voltage_v: 6.0,
            timestamp: Instant::now(),
            valid: true,
        }
    }
    #[test]
    fn low_pass_filter_is_exponential() {
        let mut c = SoftGraspController::new(config(), ["finger1".into()]);
        let f = finger();
        let a = fb(100.0);
        c.update(0, &f, [0.0, 0.0], [&a, &a], None);
        assert_eq!(c.filtered_loads(0), [20.0, 20.0]);
    }
    #[test]
    fn spike_does_not_trigger_contact() {
        let mut c = SoftGraspController::new(config(), ["finger1".into()]);
        let f = finger();
        let hi = fb(800.0);
        let lo = fb(0.0);
        c.update(0, &f, [0.0, 0.0], [&lo, &lo], None);
        let d = c.update(0, &f, [0.02, -0.02], [&hi, &hi], None);
        assert!(!d.contact_detected);
    }
    #[test]
    fn contact_freezes_closing_and_opening_releases() {
        let mut c = SoftGraspController::new(config(), ["finger1".into()]);
        let f = finger();
        let hi = fb(500.0);
        let lo = fb(0.0);
        c.update(0, &f, [0.0, 0.0], [&lo, &lo], None);
        for _ in 0..3 {
            let hi = fb(500.0);
            c.update(0, &f, [0.02, -0.02], [&hi, &hi], None);
        }
        let held = c.update(0, &f, [0.5, -0.5], [&hi, &hi], None);
        assert_eq!(held.state, GraspState::Holding);
        assert!(held.targets[0] < 0.1);
        let released = c.update(0, &f, [-0.2, 0.2], [&lo, &lo], None);
        assert_eq!(released.state, GraspState::Releasing);
        assert_eq!(released.targets, [-0.2, 0.2]);
    }
    #[test]
    fn closing_step_is_limited() {
        let mut c = SoftGraspController::new(config(), ["finger1".into()]);
        let f = finger();
        let lo = fb(0.0);
        c.update(0, &f, [0.0, 0.0], [&lo, &lo], None);
        let d = c.update(0, &f, [1.0, -1.0], [&lo, &lo], None);
        assert_eq!(d.targets, [0.02, -0.02]);
    }
    #[test]
    fn first_command_is_limited_from_the_initialized_target() {
        let mut c = SoftGraspController::new(config(), ["finger1".into()]);
        c.set_initial_targets([[0.0, 0.0]]);
        let f = finger();
        let lo = fb(0.0);
        let d = c.update(0, &f, [1.0, -1.0], [&lo, &lo], None);
        assert_eq!(d.targets, [0.02, -0.02]);
    }
    #[test]
    fn overload_faults() {
        let mut c = SoftGraspController::new(config(), ["finger1".into()]);
        let f = finger();
        let hi = fb(2_000.0);
        let d = c.update(0, &f, [0.0, 0.0], [&hi, &hi], None);
        assert_eq!(d.state, GraspState::Fault);
        assert_eq!(d.fault_reason, Some(FaultReason::Overload));
    }
}
