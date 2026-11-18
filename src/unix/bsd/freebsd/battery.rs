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
use crate::common::battery::BatteryChemistry;
use crate::unix::sys::ffi::{acpi_battery_ioctl_arg, acpi_bix, acpi_bst};
use crate::{BatteriesInner, Battery, BatteryChargeStatus, Error};
use libc::{_IOR, _IOWR, c_int, c_ulong, ioctl};
use std::fs::File;
use std::os::fd::AsRawFd;

fn refresh_battery_list(batteries: &mut Vec<Battery>, remove_not_listed_devices: bool) {
    const ACPIIO_BATT_GET_UNITS: c_ulong = _IOR::<c_int>(b'B' as c_ulong, 0x01);
    const ACPIIO_BATT_GET_BIX: c_ulong = _IOWR::<acpi_bix>(b'B' as c_ulong, 0x10);
    const ACPIIO_BATT_GET_BST: c_ulong = _IOWR::<acpi_bst>(b'B' as c_ulong, 0x11);

    unsafe {
        let Ok(dev) = File::open("/dev/acpi") else {
            debug!("Failed to open /dev/acpi");
            return;
        };

        let fd = dev.as_raw_fd();

        let mut unit_count: c_int = 0;
        if ioctl(fd, ACPIIO_BATT_GET_UNITS, &mut unit_count) < 0 {
            debug!("Failed to get battery unit count");
            return;
        }

        for index in 0..unit_count {
            let mut args: acpi_battery_ioctl_arg = std::mem::zeroed();
            if ioctl(fd, ACPIIO_BATT_GET_BIX, &mut args) < 0 {
                debug!("Failed to get battery information");
                continue;
            }

            let unit = match args.bix.units {
                0 => PowerUnit::Milliwatt,
                1 => PowerUnit::Milliampere,
                _ => {
                    // invalid
                    continue;
                }
            };

            let name = format!("bat{index}");
            let model_name = String::from_utf8_lossy(
                &args.bix.model.iter().map(|&c| c as u8).collect::<Vec<u8>>(),
            )
            .trim_end_matches('\0')
            .trim()
            .to_string();

            // mAh or mWh, depending on `unit`
            let design_capacity = args.bix.dcap;
            let full_capacity = args.bix.lfcap;

            let cycles = args.bix.cycles;

            if ioctl(fd, ACPIIO_BATT_GET_BST, &mut args) < 0 {
                debug!("Failed to get battery information");
                continue;
            }

            let capacity = args.bst.cap; // mAh or mWh, depending on `unit`
            let voltage = args.bst.volt;
            let rate = args.bst.rate; // mA or mW, depending on `unit`
            let power_state = if args.bst.state & 0x01 != 0 {
                BatteryChargeStatus::Discharging
            } else if args.bst.state & 0x02 != 0 {
                BatteryChargeStatus::Charging
            } else {
                if (full_capacity - capacity) <= full_capacity / 100 {
                    BatteryChargeStatus::Full //  1% tolerance
                } else {
                    BatteryChargeStatus::NotCharging
                }
            };

            if let Some(battery) = batteries.iter_mut().find(|b| b.name() == name) {
                let inner = &mut battery.inner;
                inner.model = model_name;
                inner.unit = unit;
                inner.charge_status = power_state;
                inner.voltage = voltage;
                inner.rate = rate;
                inner.capacity = capacity;
                inner.capacity_full = full_capacity;
                inner.capacity_design = design_capacity;
                inner.cycles = cycles;
                inner.updated = true;
            } else {
                batteries.push(new_battery(
                    unit,
                    name,
                    model_name,
                    power_state,
                    voltage,
                    rate,
                    capacity,
                    full_capacity,
                    design_capacity,
                    cycles,
                ));
            }
        }

        if remove_not_listed_devices {
            batteries.retain_mut(|battery| {
                if !battery.inner.updated {
                    return false;
                }

                battery.inner.updated = false;
                true
            });
        } else {
            for battery in batteries {
                battery.inner.updated = false;
            }
        }
    }
}

fn mw_to_ua(mw: u32, voltage_mv: u32) -> u64 {
    ((mw as u64).saturating_mul(1_000_000)) / voltage_mv as u64
}

#[allow(clippy::too_many_arguments)]
fn new_battery(
    unit: PowerUnit,
    name: String,
    model: String,
    charge_status: BatteryChargeStatus,
    voltage: u32,
    rate: u32,
    capacity: u32,
    capacity_full: u32,
    capacity_design: u32,
    cycles: u32,
) -> Battery {
    Battery {
        inner: BatteryInner {
            unit,
            name,
            model,
            charge_status,
            voltage,
            rate,
            capacity,
            capacity_full,
            capacity_design,
            cycles,
            updated: true,
        },
    }
}

fn convert_charge(unit: PowerUnit, capacity: u32, voltage: u32) -> u64 {
    match unit {
        PowerUnit::Milliampere => (capacity as u64).saturating_mul(1000),
        PowerUnit::Milliwatt => mw_to_ua(capacity, voltage),
    }
}

#[derive(Copy, Clone)]
pub(crate) enum PowerUnit {
    Milliwatt,
    Milliampere,
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

    pub(crate) fn refresh(&mut self, remove_not_listed_interfaces: bool) {
        refresh_battery_list(&mut self.batteries, remove_not_listed_interfaces);
    }
}

#[derive(Clone)]
pub(crate) struct BatteryInner {
    pub(crate) unit: PowerUnit,
    pub(crate) name: String,
    pub(crate) model: String,
    pub(crate) charge_status: BatteryChargeStatus,
    pub(crate) voltage: u32,
    pub(crate) rate: u32,
    pub(crate) capacity: u32,
    pub(crate) capacity_full: u32,
    pub(crate) capacity_design: u32,
    pub(crate) cycles: u32,
    pub(crate) updated: bool,
}

impl BatteryInner {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn manufacturer(&self) -> Option<&str> {
        None
    }

    pub(crate) fn model(&self) -> Option<&str> {
        Some(&self.model)
    }

    pub(crate) fn charge_status(&self) -> Option<BatteryChargeStatus> {
        Some(self.charge_status)
    }

    pub(crate) fn charge_now(&self) -> Option<u64> {
        Some(convert_charge(self.unit, self.capacity, self.voltage))
    }

    pub(crate) fn charge_full(&self) -> Option<u64> {
        Some(convert_charge(self.unit, self.capacity_full, self.voltage))
    }

    pub(crate) fn charge_full_design(&self) -> Option<u64> {
        Some(convert_charge(
            self.unit,
            self.capacity_design,
            self.voltage,
        ))
    }

    pub(crate) fn charge_control_start_threshold(&self) -> Option<u64> {
        None
    }

    pub(crate) fn charge_control_end_threshold(&self) -> Option<u64> {
        None
    }

    pub(crate) fn voltage_now(&self) -> Option<u64> {
        Some((self.voltage as u64).saturating_mul(1000))
    }

    pub(crate) fn current_now(&self) -> Option<u64> {
        match self.unit {
            PowerUnit::Milliwatt => Some(mw_to_ua(self.rate, self.voltage)),
            PowerUnit::Milliampere => Some((self.rate as u64).saturating_mul(1000)),
        }
    }

    pub(crate) fn cycle_count(&self) -> Option<u32> {
        Some(self.cycles)
    }

    pub(crate) fn chemistry(&self) -> Option<BatteryChemistry> {
        None
    }
}
