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
use crate::sys::utils::{
    IS_ARM, MAIN_PORT, get_cf_dictionary_i64, get_cf_dictionary_string, read_ioreg_property_bool,
    read_ioreg_property_i64, read_ioreg_property_string,
};
use crate::{BatteriesInner, Battery, BatteryChargeStatus, Error};
use objc2_core_foundation::{
    CFArray, CFDictionary, CFRetained, CFString, CFType, kCFAllocatorDefault,
};
use objc2_io_kit::{
    IOPSCopyPowerSourcesInfo, IOPSCopyPowerSourcesList, IOPSGetPowerSourceDescription,
    IORegistryEntryCreateCFProperty, IOServiceGetMatchingService, IOServiceMatching,
    kIOPSInternalBatteryType,
};

fn refresh_battery_list(batteries: &mut Vec<Battery>, remove_not_listed_devices: bool) {
    unsafe {
        let Some(matching) = IOServiceMatching(c"AppleSmartBattery".as_ptr()) else {
            debug!("Failed to create AppleSmartBattery matching dictionary");
            return;
        };
        let matching = CFRetained::<CFDictionary>::from(&matching);
        let service = IOServiceGetMatchingService(*MAIN_PORT, Some(matching)); //  Macs only have one battery

        let Some(ps_info) = IOPSCopyPowerSourcesInfo() else {
            debug!("IOPSCopyPowerSourcesInfo failed");
            return;
        };
        let Some(ps_list) = IOPSCopyPowerSourcesList(Some(&ps_info)) else {
            debug!("IOPSCopyPowerSourcesList failed");
            return;
        };
        if ps_list.is_empty() {
            debug!("IOPSCopyPowerSourcesList failed");
            return;
        }

        let ps_list: &CFArray<CFType> = ps_list.cast_unchecked();
        let descriptions: Vec<CFRetained<CFDictionary>> = ps_list
            .iter()
            .filter_map(|ps| IOPSGetPowerSourceDescription(Some(&ps_info), Some(&ps)))
            .collect();

        let list: Vec<CFRetained<CFDictionary>> = descriptions
            .into_iter()
            .filter(|desc| {
                get_cf_dictionary_string(desc, "Type")
                    == Some(kIOPSInternalBatteryType.to_string_lossy().into_owned())
            })
            .collect();

        //  Macs only have one battery
        let Some(dict) = list.first() else {
            return;
        };

        let battery_data_identifier = CFString::from_str("BatteryData");
        let Some(battery_data) = IORegistryEntryCreateCFProperty(
            service,
            battery_data_identifier.downcast_ref(),
            kCFAllocatorDefault,
            0,
        )
        .and_then(|d| d.downcast::<CFDictionary>().ok()) else {
            return;
        };

        let Some(name) = get_cf_dictionary_string(dict, "Name") else {
            return;
        };

        //  ManufacturerData available, but we don't know its corresponding manufacturers.
        let model_name = read_ioreg_property_string(service, "DeviceName");

        let charge_status = match (
            get_cf_dictionary_string(dict, "Power Source State").as_deref(),
            read_ioreg_property_bool(service, "FullyCharged"),
            read_ioreg_property_bool(service, "IsCharging"),
        ) {
            (Some("Battery Power"), _, _) => Some(BatteryChargeStatus::Discharging),
            (_, Some(true), _) => Some(BatteryChargeStatus::Full),
            (_, _, Some(true)) => Some(BatteryChargeStatus::Charging),
            (_, Some(false), Some(false)) => Some(BatteryChargeStatus::NotCharging),
            _ => None,
        };

        //  Apple provide values in mV, mA and mAh
        let voltage_now = read_ioreg_property_i64(service, "Voltage").map(|v| v.unsigned_abs());
        let current_now = if !IS_ARM || charge_status == Some(BatteryChargeStatus::Discharging) {
            read_ioreg_property_i64(service, "Amperage").map(|v| v.unsigned_abs())
        } else {
            //  On M series, when charging, Amperage is the input current charging the battery.
            None
        };
        let capacity_now = read_ioreg_property_i64(service, "AppleRawCurrentCapacity")
            .or_else(|| get_cf_dictionary_i64(&battery_data, "RemainingCapacity"))
            .map(|v| v.unsigned_abs());
        let capacity_max = read_ioreg_property_i64(service, "AppleRawMaxCapacity")
            .or_else(|| read_ioreg_property_i64(service, "MaxCapacity"))
            .or_else(|| read_ioreg_property_i64(service, "NominalChargeCapacity"))
            .or_else(|| get_cf_dictionary_i64(&battery_data, "FullChargeCapacity"))
            .or_else(|| get_cf_dictionary_i64(&battery_data, "NominalChargeCapacity"))
            .map(|v| v.unsigned_abs());
        let capacity_designed = read_ioreg_property_i64(service, "DesignCapacity")
            .or_else(|| get_cf_dictionary_i64(&battery_data, "DesignCapacity"))
            .map(|v| v.unsigned_abs());

        if let Some(existing_battery) = batteries.iter_mut().find(|b| b.inner.name == name) {
            let inner = &mut existing_battery.inner;
            inner.model_name = model_name;
            inner.charge_status = charge_status;
            inner.voltage_now = voltage_now;
            inner.current_now = current_now;
            inner.capacity_now = capacity_now;
            inner.capacity_max = capacity_max;
            inner.capacity_designed = capacity_designed;
            inner.updated = true;
        } else {
            batteries.push(new_battery(
                name,
                model_name,
                charge_status,
                voltage_now,
                current_now,
                capacity_now,
                capacity_max,
                capacity_designed,
            ));
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

#[allow(clippy::too_many_arguments)]
fn new_battery(
    name: String,
    model_name: Option<String>,
    charge_status: Option<BatteryChargeStatus>,
    voltage_now: Option<u64>,
    current_now: Option<u64>,
    capacity_now: Option<u64>,
    capacity_max: Option<u64>,
    capacity_designed: Option<u64>,
) -> Battery {
    Battery {
        inner: BatteryInner {
            name,
            model_name,
            charge_status,
            voltage_now,
            current_now,
            capacity_now,
            capacity_max,
            capacity_designed,
            updated: true,
        },
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
    //     &mut []
    // }

    pub(crate) fn refresh(&mut self, remove_not_listed_interfaces: bool) {
        refresh_battery_list(&mut self.batteries, remove_not_listed_interfaces);
    }
}

#[derive(Clone)]
pub(crate) struct BatteryInner {
    pub(crate) name: String,
    pub(crate) model_name: Option<String>,
    pub(crate) charge_status: Option<BatteryChargeStatus>,
    pub(crate) voltage_now: Option<u64>,
    pub(crate) current_now: Option<u64>,
    pub(crate) capacity_now: Option<u64>,
    pub(crate) capacity_max: Option<u64>,
    pub(crate) capacity_designed: Option<u64>,
    pub(crate) updated: bool,
}

impl BatteryInner {
    // pub(crate) fn refresh(&mut self) {}

    pub(crate) fn name(&self) -> &str {
        self.name.as_str()
    }

    pub(crate) fn manufacturer(&self) -> Option<&str> {
        None
    }

    pub(crate) fn model_name(&self) -> Option<&str> {
        self.model_name.as_deref()
    }

    pub(crate) fn charge_status(&self) -> Option<BatteryChargeStatus> {
        self.charge_status
    }

    pub(crate) fn charge_now(&self) -> Option<u64> {
        Some(self.capacity_now?.saturating_mul(1000))
    }

    pub(crate) fn charge_full(&self) -> Option<u64> {
        Some(self.capacity_max?.saturating_mul(1000))
    }

    pub(crate) fn charge_full_design(&self) -> Option<u64> {
        Some(self.capacity_designed?.saturating_mul(1000))
    }

    pub(crate) fn charge_control_start_threshold(&self) -> Option<u64> {
        None
    }

    pub(crate) fn charge_control_end_threshold(&self) -> Option<u64> {
        None
    }

    pub(crate) fn voltage_now(&self) -> Option<u64> {
        Some(self.voltage_now?.saturating_mul(1000))
    }

    pub(crate) fn current_now(&self) -> Option<u64> {
        Some(self.current_now?.saturating_mul(1000))
    }
}
