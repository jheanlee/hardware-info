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
use crate::sys::utils::{DeviceInfoReleaser, FileHandleReleaser};
use crate::{Battery, BatteryChargeStatus, Error};
use windows::Win32::Devices::DeviceAndDriverInstallation::{
    DIGCF_DEVICEINTERFACE, DIGCF_PRESENT, GUID_DEVCLASS_BATTERY, SP_DEVICE_INTERFACE_DATA,
    SP_DEVICE_INTERFACE_DETAIL_DATA_W, SetupDiEnumDeviceInterfaces, SetupDiGetClassDevsW,
    SetupDiGetDeviceInterfaceDetailW,
};
use windows::Win32::Foundation::{
    ERROR_INSUFFICIENT_BUFFER, ERROR_NO_MORE_ITEMS, GENERIC_READ, GetLastError,
};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::IO::DeviceIoControl;
use windows::Win32::System::Power::{
    BATTERY_CAPACITY_RELATIVE, BATTERY_INFORMATION, BATTERY_IS_SHORT_TERM,
    BATTERY_QUERY_INFORMATION, BATTERY_STATUS, BATTERY_SYSTEM_BATTERY, BATTERY_WAIT_STATUS,
    BatteryDeviceName, BatteryInformation, BatteryManufactureName, IOCTL_BATTERY_QUERY_INFORMATION,
    IOCTL_BATTERY_QUERY_STATUS, IOCTL_BATTERY_QUERY_TAG,
};
use windows::core::PCWSTR;

//  https://learn.microsoft.com/en-us/windows/win32/power/enumerating-battery-devices
fn refresh_battery_list(batteries: &mut Vec<Battery>, remove_not_listed_devices: bool) {
    const MAX_BATTERIES: u32 = 100; //  maximum 100 batteries as Microsoft's example, which is already a lot (for now)

    unsafe {
        let Ok(dev_info_set) = SetupDiGetClassDevsW(
            Some(&GUID_DEVCLASS_BATTERY),
            None,
            None,
            DIGCF_DEVICEINTERFACE | DIGCF_PRESENT,
        ) else {
            debug!("Failed to get device info set");
            return;
        };

        let device_info = DeviceInfoReleaser::new(dev_info_set);

        for dev_index in 0..MAX_BATTERIES {
            let mut dev_interface_data = SP_DEVICE_INTERFACE_DATA {
                cbSize: std::mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as u32,
                InterfaceClassGuid: Default::default(),
                Flags: 0,
                Reserved: 0,
            };

            if SetupDiEnumDeviceInterfaces(
                device_info.inner(),
                None,
                &GUID_DEVCLASS_BATTERY,
                dev_index,
                &mut dev_interface_data,
            )
            .is_err()
            {
                if GetLastError() == ERROR_NO_MORE_ITEMS {
                    break;
                }
                debug!("SetupDiEnumDeviceInterfaces failed");
                continue;
            }

            let mut cb_required = 0;
            let _ = SetupDiGetDeviceInterfaceDetailW(
                device_info.inner(),
                &dev_interface_data,
                None,
                0,
                Some(&mut cb_required),
                None,
            );

            if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
                debug!("SetupDiGetDeviceInterfaceDetailW (first pass) failed");
                continue;
            }

            let mut buffer = vec![0u8; cb_required as usize];
            let device_interface_detail_data = buffer
                .as_mut_ptr()
                .cast::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>();

            (*device_interface_detail_data).cbSize =
                std::mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as u32;

            if SetupDiGetDeviceInterfaceDetailW(
                device_info.inner(),
                &dev_interface_data,
                Some(device_interface_detail_data),
                cb_required,
                Some(&mut cb_required),
                None,
            )
            .is_err()
            {
                debug!("SetupDiGetDeviceInterfaceDetailW (second pass) failed");
                continue;
            }

            let Ok(battery) = CreateFileW(
                PCWSTR((*device_interface_detail_data).DevicePath.as_ptr()),
                GENERIC_READ.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                None,
            ) else {
                debug!("Failed to read battery file");
                continue;
            };

            let battery = FileHandleReleaser::new(battery);

            let interface_path = String::from_utf16_lossy(
                PCWSTR((*device_interface_detail_data).DevicePath.as_ptr()).as_wide(),
            );
            let Some(name) = parse_battery_interface_path(interface_path.as_str()) else {
                continue;
            };

            let mut battery_query: BATTERY_QUERY_INFORMATION = std::mem::zeroed();
            let dw_wait: u32 = 0;
            let mut nbytes: u32 = 0;
            if DeviceIoControl(
                battery.inner(),
                IOCTL_BATTERY_QUERY_TAG,
                Some(&dw_wait as *const _ as _),
                std::mem::size_of::<u32>() as u32,
                Some(&mut battery_query.BatteryTag as *mut _ as _),
                std::mem::size_of::<u32>() as u32,
                Some(&mut nbytes),
                None,
            )
            .is_err()
                || battery_query.BatteryTag == 0
            {
                debug!("Failed to get battery tag");
                continue;
            };

            battery_query.InformationLevel = BatteryInformation;
            let mut battery_information: BATTERY_INFORMATION = std::mem::zeroed();

            if DeviceIoControl(
                battery.inner(),
                IOCTL_BATTERY_QUERY_INFORMATION,
                Some(&battery_query as *const _ as _),
                std::mem::size_of::<BATTERY_QUERY_INFORMATION>() as u32,
                Some(&mut battery_information as *mut _ as _),
                std::mem::size_of::<BATTERY_INFORMATION>() as u32,
                Some(&mut nbytes),
                None,
            )
            .is_err()
                || battery_information.Capabilities & BATTERY_SYSTEM_BATTERY == 0
                || battery_information.Capabilities & BATTERY_IS_SHORT_TERM != 0
            {
                //  exclude UPS
                debug!("Failed to get battery information");
                continue;
            }

            //  capacity in mWh
            let (full_capacity, designed_capacity) =
                if battery_information.Capabilities & BATTERY_CAPACITY_RELATIVE == 0 {
                    (
                        Some(battery_information.FullChargedCapacity),
                        Some(battery_information.DesignedCapacity),
                    )
                } else {
                    (None, None)
                };

            let mut battery_status: BATTERY_STATUS = std::mem::zeroed();

            let battery_wait_status = BATTERY_WAIT_STATUS {
                BatteryTag: battery_query.BatteryTag,
                Timeout: 0,
                PowerState: 0,
                LowCapacity: 0,
                HighCapacity: 0,
            };

            if DeviceIoControl(
                battery.inner(),
                IOCTL_BATTERY_QUERY_STATUS,
                Some(&battery_wait_status as *const _ as *const _),
                std::mem::size_of::<BATTERY_WAIT_STATUS>() as u32,
                Some(&mut battery_status as *mut _ as *mut _),
                std::mem::size_of::<BATTERY_STATUS>() as u32,
                Some(&mut nbytes),
                None,
            )
            .is_err()
            {
                debug!("Failed to get battery status");
                continue;
            }

            let capacity = battery_status.Capacity; // mWh
            let voltage = battery_status.Voltage; // mV
            let rate = battery_status.Rate; // mW

            let power_state = if battery_status.PowerState & 0x02 != 0 {
                Some(BatteryChargeStatus::Discharging)
            } else if battery_status.PowerState & 0x04 != 0 {
                Some(BatteryChargeStatus::Charging)
            } else if battery_status.PowerState & 0x01 != 0 {
                if let Some(full_capacity) = full_capacity
                    && (full_capacity - capacity) <= full_capacity / 100
                {
                    Some(BatteryChargeStatus::Full) //  1% tolerance
                } else {
                    Some(BatteryChargeStatus::NotCharging)
                }
            } else {
                None
            };

            let mut str_buffer = vec![0u16; 256];
            battery_query.InformationLevel = BatteryDeviceName;

            if DeviceIoControl(
                battery.inner(),
                IOCTL_BATTERY_QUERY_INFORMATION,
                Some(&battery_query as *const _ as *const _),
                size_of::<BATTERY_QUERY_INFORMATION>() as u32,
                Some(str_buffer.as_mut_ptr() as *mut _),
                (str_buffer.len() * size_of::<u16>()) as u32,
                Some(&mut nbytes),
                None,
            )
            .is_err()
            {
                debug!("Failed to get battery name");
                continue;
            }

            let str_end = str_buffer[..str_buffer.len().min((nbytes / 2) as usize)]
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(0);
            let model_name = if str_end != 0 {
                Some(String::from_utf16_lossy(&str_buffer[..str_end]))
            } else {
                None
            };

            battery_query.InformationLevel = BatteryManufactureName;

            if DeviceIoControl(
                battery.inner(),
                IOCTL_BATTERY_QUERY_INFORMATION,
                Some(&battery_query as *const _ as *const _),
                size_of::<BATTERY_QUERY_INFORMATION>() as u32,
                Some(str_buffer.as_mut_ptr() as *mut _),
                (str_buffer.len() * size_of::<u16>()) as u32,
                Some(&mut nbytes),
                None,
            )
            .is_err()
            {
                debug!("Failed to get battery manufacturer name");
                continue;
            }

            let str_end = str_buffer[..str_buffer.len().min((nbytes / 2) as usize)]
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(0);
            if str_end == 0 {
                debug!("Failed to get battery manufacturer name");
                continue;
            }

            let manufacturer_name = Some(String::from_utf16_lossy(&str_buffer[..str_end]));

            if let Some(battery) = batteries.iter_mut().find(|b| b.inner.name == name) {
                let inner = &mut battery.inner;
                inner.manufacturer = manufacturer_name;
                inner.model_name = model_name;
                inner.voltage_now = voltage;
                inner.wattage_now = rate;
                inner.energy_now = capacity;
                inner.energy_full = full_capacity;
                inner.energy_full_design = designed_capacity;
                inner.charge_status = power_state;
                inner.updated = true;
            } else {
                batteries.push(new_battery(
                    name,
                    manufacturer_name,
                    model_name,
                    power_state,
                    capacity,
                    full_capacity,
                    designed_capacity,
                    voltage,
                    rate,
                ));
            }
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

fn parse_battery_interface_path(path: &str) -> Option<String> {
    let (_, path) = path.split_once(r"\\?\")?;
    let (device, _) = path.split_once("#{")?;
    Some(device.to_string())
}

#[allow(clippy::too_many_arguments)]
fn new_battery(
    name: String,
    manufacturer: Option<String>,
    model_name: Option<String>,
    charge_status: Option<BatteryChargeStatus>,
    energy_now: u32,
    energy_full: Option<u32>,
    energy_full_design: Option<u32>,
    voltage_now: u32,
    wattage_now: i32,
) -> Battery {
    Battery {
        inner: BatteryInner {
            name,
            manufacturer,
            model_name,
            charge_status,
            energy_now,
            energy_full,
            energy_full_design,
            voltage_now,
            wattage_now,
            updated: true,
        },
    }
}

fn mw_to_ua(mw: u32, voltage_mv: u32) -> u64 {
    ((mw as u64).saturating_mul(1_000_000)) / voltage_mv as u64
}

#[derive(Clone)]
pub(crate) struct BatteriesInner {
    pub(crate) batteries: Vec<Battery>,
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

    pub(crate) fn into_vec(self) -> Vec<Battery> {
        self.batteries
    }

    pub(crate) fn refresh(&mut self, remove_not_listed_interfaces: bool) {
        refresh_battery_list(&mut self.batteries, remove_not_listed_interfaces);
    }
}

#[derive(Clone)]
pub(crate) struct BatteryInner {
    name: String,
    manufacturer: Option<String>,
    model_name: Option<String>,
    charge_status: Option<BatteryChargeStatus>,
    energy_now: u32,
    energy_full: Option<u32>,
    energy_full_design: Option<u32>,
    voltage_now: u32,
    wattage_now: i32,
    updated: bool,
}

impl BatteryInner {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn manufacturer(&self) -> Option<&str> {
        self.manufacturer.as_deref()
    }

    pub(crate) fn model_name(&self) -> Option<&str> {
        self.model_name.as_deref()
    }

    pub(crate) fn charge_status(&self) -> Option<BatteryChargeStatus> {
        self.charge_status
    }

    pub(crate) fn charge_now(&self) -> Option<u64> {
        Some(mw_to_ua(self.energy_now, self.voltage_now))
    }

    pub(crate) fn charge_full(&self) -> Option<u64> {
        Some(mw_to_ua(self.energy_full?, self.voltage_now))
    }

    pub(crate) fn charge_full_design(&self) -> Option<u64> {
        Some(mw_to_ua(self.energy_full_design?, self.voltage_now))
    }

    pub(crate) fn charge_control_start_threshold(&self) -> Option<u64> {
        None
    }

    pub(crate) fn charge_control_end_threshold(&self) -> Option<u64> {
        None
    }

    pub(crate) fn voltage_now(&self) -> Option<u64> {
        Some(self.voltage_now.saturating_mul(1000) as u64)
    }

    pub(crate) fn current_now(&self) -> Option<u64> {
        Some(mw_to_ua(self.wattage_now.unsigned_abs(), self.voltage_now))
    }
}
