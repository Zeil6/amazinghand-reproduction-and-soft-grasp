// Modified from Pollen Robotics AmazingHand (Apache-2.0).
// Local changes add SCS0009 feedback reads and bus abstractions for soft grasp.

use rustypot::{
    servo,
    servo::{
        conversion::Conversion,
        feetech::scs0009::{AnglePosition, BigEndian_i16, Velocity},
    },
};
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct ServoFeedback {
    pub id: u8,
    pub position_rad: f64,
    pub speed: f64,
    pub raw_load: f64,
    pub filtered_load: f64,
    pub temperature_c: f64,
    pub voltage_v: f64,
    pub timestamp: Instant,
    pub valid: bool,
}

#[derive(Clone, Debug)]
pub struct FingerFeedback {
    pub finger_name: String,
    pub motor1: ServoFeedback,
    pub motor2: ServoFeedback,
    pub contact_detected: bool,
    pub overload: bool,
    pub position_error: f64,
}

pub trait FeedbackSource {
    fn read_feedback(&mut self, ids: &[u8]) -> eyre::Result<Vec<ServoFeedback>>;
}

pub trait ServoBus: FeedbackSource {
    fn write_goal_positions(&mut self, ids: &[u8], positions: &[f64]) -> eyre::Result<()>;
    fn write_goal_speeds(&mut self, ids: &[u8], speeds: &[f64]) -> eyre::Result<()>;
    fn write_torque_enable(&mut self, ids: &[u8], enabled: bool) -> eyre::Result<()>;
}

pub struct Scs0009Bus {
    controller: servo::feetech::scs0009::Scs0009Controller,
    voltage_scale_v_per_raw: f64,
}

impl Scs0009Bus {
    pub fn open(
        serialport_name: &str,
        baudrate: u32,
        voltage_scale_v_per_raw: f64,
    ) -> eyre::Result<Self> {
        let port = serialport::new(serialport_name, baudrate)
            .timeout(std::time::Duration::from_millis(10))
            .open()?;
        Ok(Self {
            controller: servo::feetech::scs0009::Scs0009Controller::new()
                .with_protocol_v1()
                .with_serial_port(port),
            voltage_scale_v_per_raw,
        })
    }
}

impl FeedbackSource for Scs0009Bus {
    fn read_feedback(&mut self, ids: &[u8]) -> eyre::Result<Vec<ServoFeedback>> {
        let timestamp = Instant::now();
        // SCS0009 registers 56..63 are contiguous. Although rustypot exposes sync_read,
        // deployed SCS0009 buses commonly do not answer that broadcast instruction. Use
        // protocol-v1 unicast reads, which match the existing get_zeros tool's behavior.
        ids.iter()
            .map(|id| {
                let row = self.controller.read_raw_data(*id, 56, 8).map_err(|error| {
                    eyre::eyre!("SCS0009 feedback read failed for id {id}: {error}")
                })?;
                if row.len() != 8 {
                    return Err(eyre::eyre!(
                        "servo {id} returned {} feedback bytes, expected 8",
                        row.len()
                    ));
                }
                let position_raw = i16::from_le_bytes([row[0], row[1]]);
                let speed_raw = u16::from_le_bytes([row[2], row[3]]);
                let load_raw = u16::from_le_bytes([row[4], row[5]]);
                Ok(ServoFeedback {
                    id: *id,
                    position_rad: AnglePosition::from_raw(position_raw),
                    speed: Velocity::from_raw(speed_raw),
                    raw_load: BigEndian_i16::from_raw(load_raw) as f64,
                    filtered_load: 0.0,
                    temperature_c: row[7] as f64,
                    voltage_v: row[6] as f64 * self.voltage_scale_v_per_raw,
                    timestamp,
                    valid: true,
                })
            })
            .collect()
    }
}

impl ServoBus for Scs0009Bus {
    fn write_goal_positions(&mut self, ids: &[u8], positions: &[f64]) -> eyre::Result<()> {
        self.controller
            .sync_write_goal_position(ids, positions)
            .map_err(|error| eyre::eyre!("SCS0009 goal-position write failed: {error}"))?;
        Ok(())
    }

    fn write_goal_speeds(&mut self, ids: &[u8], speeds: &[f64]) -> eyre::Result<()> {
        self.controller
            .sync_write_goal_speed(ids, speeds)
            .map_err(|error| eyre::eyre!("SCS0009 goal-speed write failed: {error}"))?;
        Ok(())
    }

    fn write_torque_enable(&mut self, ids: &[u8], enabled: bool) -> eyre::Result<()> {
        self.controller
            .sync_write_torque_enable(ids, &vec![u8::from(enabled); ids.len()])
            .map_err(|error| eyre::eyre!("SCS0009 torque write failed: {error}"))?;
        Ok(())
    }
}
