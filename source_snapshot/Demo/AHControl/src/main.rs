// Modified from Pollen Robotics AmazingHand (Apache-2.0).
// Local changes integrate servo feedback, soft-grasp control, and Dora status outputs.

use clap::Parser;
use dora_node_api::{
    arrow::array::{Array, Float64Array},
    DoraNode, Event, MetadataParameters, Parameter,
};
use std::{
    error::Error,
    fs::File,
    io::{BufWriter, Write},
    time::{Duration, Instant},
};
use tracing::{error, info, warn};
use AHControl::{
    config::{FaultAction, HandConfig, ShutdownAction},
    feedback::{FeedbackSource, Scs0009Bus, ServoBus, ServoFeedback},
    safety::{feedback_fault, FaultReason},
    soft_grasp::{GraspState, SoftGraspController},
};

#[derive(Parser, Debug)]
#[command(author, version, about = "AmazingHand motor controller")]
struct Args {
    #[arg(short, long, default_value = "/dev/ttyACM0")]
    serialport: String,
    #[arg(short, long, default_value_t = 1_000_000)]
    baudrate: u32,
    #[arg(short, long, default_value = "AHControl/config/r_hand.toml")]
    config: String,
    /// Explicitly enable the configured soft-grasp controller for this invocation.
    #[arg(long)]
    soft_grasp: bool,
    /// Optional CSV feedback log, written only when soft-grasp mode is active.
    #[arg(long)]
    csv_log: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut config = HandConfig::from_path(&args.config)?;
    if args.soft_grasp {
        config.soft_grasp.enabled = true;
    }
    if config.fingers.motors.len() != 4
        || config
            .fingers
            .motors
            .iter()
            .any(|finger| finger.motor1.model != "SCS0009" || finger.motor2.model != "SCS0009")
    {
        return Err("AHControl requires four SCS0009 finger pairs".into());
    }
    let ids: Vec<u8> = config
        .fingers
        .motors
        .iter()
        .flat_map(|finger| [finger.motor1.id, finger.motor2.id])
        .collect();
    let zero_positions: Vec<f64> = config
        .fingers
        .motors
        .iter()
        .flat_map(|finger| {
            [
                finger.motor1.command_position(0.0),
                finger.motor2.command_position(0.0),
            ]
        })
        .collect();
    let mut bus = Scs0009Bus::open(
        &args.serialport,
        args.baudrate,
        config.soft_grasp.voltage_scale_v_per_raw,
    )?;
    bus.write_torque_enable(&ids, true)?;
    if let Err(error) = bus.write_goal_positions(&ids, &zero_positions) {
        let _ = bus.write_torque_enable(&ids, false);
        return Err(error.into());
    }
    if config.soft_grasp.enabled {
        let speeds = vec![config.soft_grasp.goal_speed_rad_s; ids.len()];
        if let Err(error) = bus.write_goal_speeds(&ids, &speeds) {
            cleanup(&mut bus, &ids, &zero_positions, config.shutdown_action);
            return Err(error.into());
        }
    }
    info!(soft_grasp = config.soft_grasp.enabled, ids = ?ids, "hand controller started");

    let names = config
        .fingers
        .motors
        .iter()
        .map(|finger| finger.finger_name.clone())
        .collect::<Vec<_>>();
    let mut soft_controller = SoftGraspController::new(config.soft_grasp.clone(), names);
    soft_controller.set_initial_targets(
        zero_positions
            .chunks_exact(2)
            .map(|pair| [pair[0], pair[1]]),
    );
    let (mut node, mut events) = match DoraNode::init_from_env() {
        Ok(node) => node,
        Err(error) => {
            cleanup(&mut bus, &ids, &zero_positions, config.shutdown_action);
            return Err(error.into());
        }
    };
    let feedback_period = Duration::from_secs_f64(
        1.0 / (config.soft_grasp.feedback_hz.max(1) as f64 * ids.len() as f64),
    );
    let feedback_publish_period =
        Duration::from_secs_f64(1.0 / config.soft_grasp.feedback_hz.max(1) as f64);
    let command_period = Duration::from_secs_f64(1.0 / config.soft_grasp.command_hz.max(1) as f64);
    let mut last_feedback_at = Instant::now() - feedback_period;
    let mut last_feedback_publish_at = Instant::now();
    let mut feedback_cursor = 0usize;
    let mut last_command_at = Instant::now();
    let mut feedback = invalid_feedback(&ids);
    let mut feedback_ready = false;
    let mut baseline_ready_reported = false;
    let mut last_feedback_error_log = Instant::now() - Duration::from_secs(1);
    let mut reported_safety_fault: Option<FaultReason> = None;
    let mut requested = zero_positions.clone();
    let mut commanded = zero_positions;
    let mut grasp_states = vec![GraspState::Tracking; config.fingers.motors.len()];
    let mut csv_writer = match args.csv_log {
        Some(path) => {
            let mut writer = BufWriter::new(File::create(path)?);
            writeln!(writer, "timestamp,finger,motor_id,command_position,actual_position,raw_load,filtered_load,temperature,voltage,grasp_state")?;
            Some(writer)
        }
        None => None,
    };
    let mut running = true;

    while running {
        if let Some(event) = events.recv_timeout(Duration::from_millis(2)) {
            match event {
                Event::Input { id, metadata, data } if id.as_str() == "mj_joints_pos" => {
                    let Some(values) = data.as_any().downcast_ref::<Float64Array>() else {
                        warn!("mj_joints_pos was not Float64Array");
                        continue;
                    };
                    if let Some(next) = map_targets(values.values(), &metadata.parameters, &config)
                    {
                        requested = next;
                        if !config.soft_grasp.enabled {
                            if let Err(error) = bus.write_goal_positions(&ids, &requested) {
                                warn!(%error, "position command failed");
                            } else {
                                commanded.clone_from(&requested);
                            }
                        }
                    }
                }
                Event::Stop(cause) => {
                    info!(?cause, "received stop");
                    running = false;
                }
                Event::Error(message) if !message.contains("Timeout") => {
                    warn!(%message, "Dora event error")
                }
                _ => {}
            }
        } else {
            running = false;
        }

        let now = Instant::now();
        if config.soft_grasp.enabled && now.duration_since(last_feedback_at) >= feedback_period {
            last_feedback_at = now;
            let id = ids[feedback_cursor];
            match bus.read_feedback(&[id]) {
                Ok(next) => {
                    feedback[feedback_cursor] = next[0].clone();
                    // A pair is sampled once its second motor has been refreshed.
                    if feedback_cursor % 2 == 1
                        && feedback[feedback_cursor - 1].valid
                        && feedback[feedback_cursor].valid
                    {
                        soft_controller.observe_baseline(
                            feedback_cursor / 2,
                            [&feedback[feedback_cursor - 1], &feedback[feedback_cursor]],
                        );
                    }
                    feedback_ready = feedback.iter().all(|sample| sample.valid);
                    if !baseline_ready_reported && soft_controller.baseline_ready() {
                        eprintln!(
                            "AHControl load baseline ready after {} feedback samples; soft-grasp tracking enabled",
                            config.soft_grasp.baseline_samples
                        );
                        baseline_ready_reported = true;
                    }
                    if now.duration_since(last_feedback_publish_at) >= feedback_publish_period {
                        last_feedback_publish_at = now;
                        publish_feedback(&mut node, &feedback, &commanded);
                        if let Err(error) = write_feedback_csv(
                            &mut csv_writer,
                            &config,
                            &feedback,
                            &commanded,
                            &grasp_states,
                        ) {
                            warn!(%error, "feedback CSV write failed");
                        }
                    }
                    feedback_cursor = (feedback_cursor + 1) % ids.len();
                }
                Err(error) => {
                    warn!(%error, "feedback read failed; retaining last valid sample");
                    if now.duration_since(last_feedback_error_log) >= Duration::from_secs(1) {
                        eprintln!("AHControl feedback read failed: {error}");
                        last_feedback_error_log = now;
                    }
                }
            }
        }
        if config.soft_grasp.enabled
            && feedback_ready
            && soft_controller.baseline_ready()
            && now.duration_since(last_command_at) >= command_period
        {
            last_command_at = now;
            let fault = feedback_fault(&feedback, &config.soft_grasp, now);
            if fault != reported_safety_fault {
                if let Some(reason) = &fault {
                    eprintln!("AHControl safety fault: {reason:?}; holding soft-grasp commands");
                }
                reported_safety_fault = fault.clone();
            }
            let mut next = requested.clone();
            let mut state_values = Vec::with_capacity(12);
            let mut torque_off = false;
            for (index, finger) in config.fingers.motors.iter().enumerate() {
                let Some(soft_config) = config.soft_grasp.finger(&finger.finger_name) else {
                    continue;
                };
                let previous_state = grasp_states[index];
                let decision = soft_controller.update(
                    index,
                    soft_config,
                    [requested[index * 2], requested[index * 2 + 1]],
                    [&feedback[index * 2], &feedback[index * 2 + 1]],
                    fault.clone(),
                );
                next[index * 2] = decision.targets[0].clamp(
                    finger.motor1.min_position_rad,
                    finger.motor1.max_position_rad,
                );
                next[index * 2 + 1] = decision.targets[1].clamp(
                    finger.motor2.min_position_rad,
                    finger.motor2.max_position_rad,
                );
                let filtered = soft_controller.filtered_loads(index);
                let baseline = soft_controller.baseline_loads(index);
                let estimated = soft_controller.estimated_loads(index);
                feedback[index * 2].filtered_load = filtered[0];
                feedback[index * 2 + 1].filtered_load = filtered[1];
                let fault_value = decision.fault_reason.clone().map(fault_code).unwrap_or(0.0);
                grasp_states[index] = decision.state;
                state_values.extend([
                    state_code(decision.state),
                    f64::from(decision.contact_detected),
                    f64::from(decision.overload),
                    fault_value,
                ]);
                if decision.state == GraspState::Fault {
                    error!(finger = %finger.finger_name, reason = ?decision.fault_reason, "soft grasp fault");
                    if previous_state != GraspState::Fault && decision.fault_reason != fault {
                        eprintln!(
                            "AHControl finger {} fault: {:?}; raw_loads=({:.0}, {:.0}), baseline_loads=({:.0}, {:.0}), filtered_raw_loads=({:.0}, {:.0}), estimated_loads=({:.0}, {:.0}), max_load={:.0}",
                            finger.finger_name,
                            decision.fault_reason,
                            feedback[index * 2].raw_load,
                            feedback[index * 2 + 1].raw_load,
                            baseline[0],
                            baseline[1],
                            filtered[0],
                            filtered[1],
                            estimated[0],
                            estimated[1],
                            soft_config.max_load,
                        );
                    }
                    match config.soft_grasp.fault_action {
                        FaultAction::Hold => {}
                        FaultAction::Backoff => {
                            next[index * 2] = (next[index * 2]
                                - soft_config.closing_sign_motor1
                                    * config.soft_grasp.fault_backoff_rad)
                                .clamp(
                                    finger.motor1.min_position_rad,
                                    finger.motor1.max_position_rad,
                                );
                            next[index * 2 + 1] = (next[index * 2 + 1]
                                - soft_config.closing_sign_motor2
                                    * config.soft_grasp.fault_backoff_rad)
                                .clamp(
                                    finger.motor2.min_position_rad,
                                    finger.motor2.max_position_rad,
                                );
                        }
                        FaultAction::TorqueOff => torque_off = true,
                    }
                }
            }
            if torque_off {
                if let Err(error) = bus.write_torque_enable(&ids, false) {
                    warn!(%error, "fault torque disable failed");
                }
            } else if let Err(error) = bus.write_goal_positions(&ids, &next) {
                warn!(%error, "soft grasp command failed");
            } else {
                commanded = next;
            }
            let _ = node.send_output(
                "grasp_state".to_owned().into(),
                MetadataParameters::default(),
                Float64Array::from(state_values),
            );
            let status = fault.map(fault_code).unwrap_or(0.0);
            let _ = node.send_output(
                "safety_status".to_owned().into(),
                MetadataParameters::default(),
                Float64Array::from(vec![status]),
            );
        }
    }

    cleanup(&mut bus, &ids, &commanded, config.shutdown_action);
    Ok(())
}

fn map_targets(
    values: &[f64],
    metadata: &MetadataParameters,
    config: &HandConfig,
) -> Option<Vec<f64>> {
    let mut targets = Vec::with_capacity(8);
    for finger in &config.fingers.motors {
        let Parameter::ListInt(indexes) = metadata.get(&finger.finger_name)? else {
            return None;
        };
        let [first, second] = indexes.as_slice() else {
            return None;
        };
        let (first, second) = (*first as usize, *second as usize);
        if first >= values.len() || second >= values.len() {
            return None;
        }
        targets.push(finger.motor1.command_position(values[first]));
        targets.push(finger.motor2.command_position(values[second]));
    }
    Some(targets)
}

fn invalid_feedback(ids: &[u8]) -> Vec<ServoFeedback> {
    let timestamp = Instant::now() - Duration::from_secs(1);
    ids.iter()
        .map(|id| ServoFeedback {
            id: *id,
            position_rad: 0.0,
            speed: 0.0,
            raw_load: 0.0,
            filtered_load: 0.0,
            temperature_c: 0.0,
            voltage_v: 0.0,
            timestamp,
            valid: false,
        })
        .collect()
}

fn publish_feedback(node: &mut DoraNode, feedback: &[ServoFeedback], commanded: &[f64]) {
    let mut values = Vec::with_capacity(1 + feedback.len() * 8);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    values.push(timestamp);
    for (sample, target) in feedback.iter().zip(commanded) {
        values.extend([
            sample.id as f64,
            *target,
            sample.position_rad,
            sample.raw_load,
            sample.filtered_load,
            sample.temperature_c,
            sample.voltage_v,
            f64::from(sample.valid),
        ]);
    }
    if let Err(error) = node.send_output(
        "motor_feedback".to_owned().into(),
        MetadataParameters::default(),
        Float64Array::from(values),
    ) {
        warn!(%error, "motor_feedback output failed");
    }
}

fn write_feedback_csv(
    writer: &mut Option<BufWriter<File>>,
    config: &HandConfig,
    feedback: &[ServoFeedback],
    commanded: &[f64],
    states: &[GraspState],
) -> std::io::Result<()> {
    let Some(writer) = writer else {
        return Ok(());
    };
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    for (index, sample) in feedback.iter().enumerate() {
        let finger_index = index / 2;
        writeln!(
            writer,
            "{timestamp:.6},{},{},{:.6},{:.6},{:.0},{:.3},{:.1},{:.2},{}",
            config.fingers.motors[finger_index].finger_name,
            sample.id,
            commanded[index],
            sample.position_rad,
            sample.raw_load,
            sample.filtered_load,
            sample.temperature_c,
            sample.voltage_v,
            state_name(states[finger_index]),
        )?;
    }
    Ok(())
}

fn cleanup(bus: &mut Scs0009Bus, ids: &[u8], commanded: &[f64], action: ShutdownAction) {
    match action {
        ShutdownAction::Hold => info!("shutdown: retaining current target and torque"),
        ShutdownAction::Open => {
            let open = vec![0.0; ids.len()];
            if let Err(error) = bus.write_goal_positions(ids, &open) {
                warn!(%error, "shutdown open command failed");
            }
        }
        ShutdownAction::TorqueOff => {
            if let Err(error) = bus.write_torque_enable(ids, false) {
                warn!(%error, "shutdown torque disable failed");
            }
        }
    }
    let _ = commanded;
}

fn state_code(state: GraspState) -> f64 {
    match state {
        GraspState::Tracking => 0.0,
        GraspState::Closing => 1.0,
        GraspState::Contact => 2.0,
        GraspState::Holding => 3.0,
        GraspState::Releasing => 4.0,
        GraspState::Fault => 5.0,
    }
}
fn fault_code(reason: FaultReason) -> f64 {
    match reason {
        FaultReason::Overload => 1.0,
        FaultReason::OverTemperature => 2.0,
        FaultReason::VoltageOutOfRange => 3.0,
        FaultReason::FeedbackTimeout => 4.0,
        FaultReason::Communication => 5.0,
    }
}

fn state_name(state: GraspState) -> &'static str {
    match state {
        GraspState::Tracking => "Tracking",
        GraspState::Closing => "Closing",
        GraspState::Contact => "Contact",
        GraspState::Holding => "Holding",
        GraspState::Releasing => "Releasing",
        GraspState::Fault => "Fault",
    }
}
