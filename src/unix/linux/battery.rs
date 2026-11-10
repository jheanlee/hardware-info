/*
 * Copyright 2026 Jhe-An Lee
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::sys::utils::{read_str, read_unsigned};
use crate::{BatteriesInner, Battery, BatteryChargeStatus, Error};

fn refresh_battery_list_from_sysfs(batteries: &mut Vec<Battery>, remove_not_listed_devices: bool) {
    let Ok(sys_dir) = std::fs::read_dir("/sys/class/power_supply") else {
        debug!("Unable to open sysfs");
        return;
    };

    for entry in sys_dir.flatten() {
        let parent = &entry.path();

        let mut str_buf = Vec::with_capacity(128);
        if read_str(parent, "type", &mut str_buf) != b"Battery\n" {
            continue;
        }

        let name = &entry.file_name().to_string_lossy().to_string();

        if let Some(battery) = batteries.iter_mut().find(|v| v.inner.name == *name) {
            battery.inner.updated = false;
            battery.inner.refresh();
        } else {
            let battery = batteries.push_mut(Battery {
                inner: Default::default(),
            });
            battery.inner.name = name.clone();
            battery.inner.refresh();
        }
    }

    if remove_not_listed_devices {
        batteries.retain_mut(|battery| {
            if !battery.inner.updated {
                return false;
            }

            battery.inner.updated = true;
            true
        });
    } else {
        for battery in batteries {
            battery.inner.updated = false;
        }
    }
}

impl BatteriesInner {
    pub(crate) fn new() -> Result<Self, Error> {
        Ok(Self {
            batteries: Vec::new(),
        })
    }

    pub(crate) fn list(&self) -> &[Battery] {
        &self.batteries
    }

    // pub(crate) fn list_mut(&mut self) -> &mut [Battery] {
    //     &mut self.batteries
    // }

    pub(crate) fn refresh(&mut self, remove_not_listed_interfaces: bool) {
        refresh_battery_list_from_sysfs(&mut self.batteries, remove_not_listed_interfaces);
    }
}

#[derive(Clone)]
pub(crate) struct BatteryInner {
    pub(crate) name: String,
    pub(crate) manufacturer: Option<String>,
    pub(crate) model_name: Option<String>,
    pub(crate) status: Option<BatteryChargeStatus>,
    pub(crate) charge_now: Option<u64>,
    pub(crate) charge_full: Option<u64>,
    pub(crate) charge_empty: Option<u64>,
    pub(crate) charge_full_design: Option<u64>,
    pub(crate) charge_empty_design: Option<u64>,
    pub(crate) charge_control_start_threshold: Option<u64>,
    pub(crate) charge_control_end_threshold: Option<u64>,
    pub(crate) voltage_now: Option<u64>,
    pub(crate) current_now: Option<u64>,
    pub(crate) updated: bool,
}

impl BatteryInner {
    pub(crate) fn refresh(&mut self) {
        let parent = format!("/sys/class/power_supply/{}", self.name);
        let parent = parent.as_str();

        if !std::fs::exists(parent).unwrap_or(false) {
            return;
        }

        let mut num_buf = [0u8; (u64::MAX.ilog10() + 2) as usize + 2];
        let mut str_buf = Vec::with_capacity(128);

        self.manufacturer = str::from_utf8(read_str(parent, "manufacturer", &mut str_buf))
            .ok()
            .map(str::to_string);
        self.model_name = str::from_utf8(read_str(parent, "model_name", &mut str_buf))
            .ok()
            .map(str::to_string);

        self.status = match read_str(parent, "status", &mut str_buf) {
            b"Charging\n" => Some(BatteryChargeStatus::Charging),
            b"Discharging\n" => Some(BatteryChargeStatus::Discharging),
            b"Not charging\n" => Some(BatteryChargeStatus::NotCharging),
            b"Full\n" => Some(BatteryChargeStatus::Full),
            _ => None,
        };

        self.charge_now = read_unsigned(parent, "charge_now", &mut num_buf);
        self.charge_full = read_unsigned(parent, "charge_full", &mut num_buf);
        self.charge_empty = read_unsigned(parent, "charge_empty", &mut num_buf);
        self.charge_full_design = read_unsigned(parent, "charge_full_design", &mut num_buf);
        self.charge_empty_design = read_unsigned(parent, "charge_empty_design", &mut num_buf);
        self.charge_control_start_threshold =
            read_unsigned(parent, "charge_control_start_threshold", &mut num_buf);
        self.charge_control_end_threshold =
            read_unsigned(parent, "charge_control_end_threshold", &mut num_buf);
        self.voltage_now = read_unsigned(parent, "voltage_now", &mut num_buf);
        self.current_now = read_unsigned(parent, "current_now", &mut num_buf);
        self.updated = true;
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn manufacturer(&self) -> Option<&str> {
        self.manufacturer.as_deref()
    }

    pub fn model_name(&self) -> Option<&str> {
        self.model_name.as_deref()
    }

    pub fn charge_status(&self) -> Option<BatteryChargeStatus> {
        self.status
    }

    pub fn charge_now(&self) -> Option<u64> {
        self.charge_now
    }

    pub fn charge_full(&self) -> Option<u64> {
        self.charge_full
            .map(|v| v.saturating_sub(self.charge_empty.unwrap_or(0)))
    }

    pub fn charge_full_design(&self) -> Option<u64> {
        self.charge_full_design
            .map(|v| v.saturating_sub(self.charge_empty_design.unwrap_or(0)))
    }

    pub fn charge_control_start_threshold(&self) -> Option<u64> {
        self.charge_control_start_threshold
    }

    pub fn charge_control_end_threshold(&self) -> Option<u64> {
        self.charge_control_end_threshold
    }

    pub fn voltage_now(&self) -> Option<u64> {
        self.voltage_now
    }

    pub fn current_now(&self) -> Option<u64> {
        self.current_now
    }
}

impl Default for BatteryInner {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            manufacturer: None,
            model_name: None,
            status: None,
            charge_now: None,
            charge_full: None,
            charge_empty: None,
            charge_full_design: None,
            charge_empty_design: None,
            charge_control_start_threshold: None,
            charge_control_end_threshold: None,
            voltage_now: None,
            current_now: None,
            updated: false,
        }
    }
}
