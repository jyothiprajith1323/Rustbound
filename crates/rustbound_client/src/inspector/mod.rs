//! Machine inspector panel reading live telemetry from the simulation snapshot.

use rustbound_protocol::packets::TelemetryPacket;

/// Formatted telemetry and status snapshot for inspector rendering.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTelemetryView {
    pub machine_id: u32,
    pub machine_type_name: &'static str,
    pub status: &'static str,
    pub temperature_celsius: i32,
    pub power_draw_watts: u32,
    pub item_count: u32,
    pub next_item_id: u64,
    pub is_overheated: bool,
    pub is_jammed: bool,
}

impl MachineTelemetryView {
    pub fn from_packet(packet: &TelemetryPacket) -> Self {
        let machine_type_name = match packet.machine_type {
            1 => "Subterranean Quarry Drill",
            2 => "Industrial Conveyor Belt",
            3 => "Planetary Storage Silo",
            4 => "Gantry Transfer Crane",
            5 => "Auxiliary Base Generator",
            _ => "Unknown Automation Device",
        };

        let is_overheated = (packet.status_flags & 0x04) != 0;
        let is_jammed = (packet.status_flags & 0x02) != 0;
        let is_running = (packet.status_flags & 0x01) != 0;

        let status = if is_overheated {
            "FAULT: THERMAL OVERHEAT"
        } else if is_jammed {
            "FAULT: INTAKE JAMMED"
        } else if is_running {
            "OPERATIONAL: ONLINE"
        } else {
            "STANDBY: IDLE"
        };

        Self {
            machine_id: packet.machine_id,
            machine_type_name,
            status,
            temperature_celsius: packet.temperature_celsius,
            power_draw_watts: packet.power_draw_watts,
            item_count: packet.item_count,
            next_item_id: packet.next_item_id,
            is_overheated,
            is_jammed,
        }
    }

    /// Returns a warning color code (hex) for UI gauge rendering.
    pub fn thermal_status_color(&self) -> (u8, u8, u8) {
        if self.temperature_celsius >= 80 {
            (239, 68, 68) // Danger Red
        } else if self.temperature_celsius >= 60 {
            (245, 158, 11) // Warning Amber
        } else {
            (34, 197, 94) // Nominal Green
        }
    }
}

/// State for the machine inspector panel.
#[derive(Default, Clone, Debug)]
pub struct InspectorState {
    pub selected_machine_id: Option<u32>,
}

impl InspectorState {
    pub fn new() -> Self {
        Self {
            selected_machine_id: None,
        }
    }

    pub fn select_machine(&mut self, machine_id: u32) {
        self.selected_machine_id = Some(machine_id);
    }
}
