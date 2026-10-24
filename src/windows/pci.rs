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
use crate::sys::utils::{parse_multi_string, parse_string};
use crate::utils::pci_vendor_name;
use crate::{Error, PciAddress, PciBusSpeed, PciDevice, PciDeviceClass};
use windows::Win32::Devices::DeviceAndDriverInstallation::{
    DIGCF_ALLCLASSES, DIGCF_PRESENT, HDEVINFO, SETUP_DI_REGISTRY_PROPERTY, SP_DEVINFO_DATA,
    SPDRP_ADDRESS, SPDRP_BUSNUMBER, SPDRP_HARDWAREID, SPDRP_SERVICE, SetupDiDestroyDeviceInfoList,
    SetupDiEnumDeviceInfo, SetupDiGetClassDevsW, SetupDiGetDevicePropertyW,
    SetupDiGetDeviceRegistryPropertyW,
};
use windows::Win32::Devices::Properties::{DEVPROP_TYPE_UINT32, DEVPROPTYPE};
use windows::Win32::Foundation::{DEVPROPKEY, ERROR_NO_MORE_ITEMS, GetLastError};
use windows::core::{GUID, PCWSTR};

fn refresh_pci_devices(devices: &mut Vec<PciDevice>, remove_not_listed_devices: bool) {
    unsafe {
        const PCI_ENUMERATOR: &[u16] = &[b'P' as u16, b'C' as u16, b'I' as u16, 0];

        let Ok(dev_info_set) = SetupDiGetClassDevsW(
            None,
            PCWSTR(PCI_ENUMERATOR.as_ptr()),
            None,
            DIGCF_ALLCLASSES | DIGCF_PRESENT,
        ) else {
            debug!("Failed to get device info set");
            return;
        };

        let mut index: u32 = 0;

        loop {
            let mut dev_info_data = SP_DEVINFO_DATA {
                cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
                ClassGuid: Default::default(),
                DevInst: 0,
                Reserved: 0,
            };

            if SetupDiEnumDeviceInfo(dev_info_set, index, &mut dev_info_data).is_err() {
                let error = GetLastError();
                if error == ERROR_NO_MORE_ITEMS {
                    break;
                }

                index += 1;
                continue;
            }

            index += 1;

            let Some(hardware_ids) =
                read_registry_property_vec(dev_info_set, &dev_info_data, SPDRP_HARDWAREID)
            else {
                continue;
            };
            let hardware_ids = parse_multi_string(&hardware_ids);

            let Some((vendor, device, subsystem_vendor, subsystem_device, revision)) = hardware_ids
                .first() //  Windows returns the ids from most to least specific
                .and_then(|v| parse_hardware_id(v.as_str()))
            else {
                continue;
            };

            let mut class = None;
            for id in hardware_ids {
                let id = id.to_ascii_uppercase();

                if id.starts_with("PCI\\")
                    && let Some(index) = id.find("CC_")
                    && let Some((_, class_str)) = id.split_at_checked(index + 3)
                    && let class_str = format!("{class_str:0<6}")   //  either 4 digit or 6 digit hex
                    && let Ok(class_val) = u32::from_str_radix(class_str.as_str(), 16)
                {
                    class = Some(PciDeviceClass::from(class_val))
                }
            }

            let Some(bus) =
                read_registry_property_u32(dev_info_set, &dev_info_data, SPDRP_BUSNUMBER)
                    .map(|v| v as u8)
            else {
                continue;
            };
            let Some(address) =
                read_registry_property_u32(dev_info_set, &dev_info_data, SPDRP_ADDRESS)
            else {
                continue;
            };

            let address = PciAddress {
                domain: 0,
                bus,
                device: (address >> 16) as u8,
                function: address as u8,
            };

            let driver = read_registry_property_vec(dev_info_set, &dev_info_data, SPDRP_SERVICE)
                .and_then(|v| parse_string(&v));

            let vendor_name = pci_vendor_name(vendor).map(str::to_string);
            let subsystem_vendor_name = subsystem_vendor
                .and_then(pci_vendor_name)
                .map(str::to_string);

            let device_type = match read_device_property_u32(dev_info_set, &dev_info_data, 1) {
                Some(0 | 6) => Some(DeviceType::Pci),
                Some(1 | 7) => Some(DeviceType::PciX),
                Some(2..=4 | 8..=12 | 14) => Some(DeviceType::PciE),
                _ => None, //  can't get meaningful values for PCI-X as PCI (5) and PCIe as PCI (13)
            };

            let current_bus_speed = match device_type {
                Some(DeviceType::Pci) => {
                    match read_device_property_u32(dev_info_set, &dev_info_data, 2) {
                        Some(0) => Some(PciBusSpeed::Pci33Mhz),
                        Some(1) => Some(PciBusSpeed::Pci66Mhz),
                        _ => None,
                    }
                }
                Some(DeviceType::PciX) => {
                    match read_device_property_u32(dev_info_set, &dev_info_data, 2) {
                        Some(0x0) => None, // PCI-X as PCI
                        Some(0x1 | 0x5) => Some(PciBusSpeed::PciX66Mhz),
                        Some(0x2 | 0x6) => Some(PciBusSpeed::PciX100Mhz),
                        Some(0x3 | 0x7) => Some(PciBusSpeed::PciX133Mhz),
                        Some(0x9) => Some(PciBusSpeed::PciX266_66Mhz),
                        Some(0xa) => Some(PciBusSpeed::PciX266_100Mhz),
                        Some(0xb) => Some(PciBusSpeed::PciX266_133Mhz),
                        Some(0xd) => Some(PciBusSpeed::PciX533_66Mhz),
                        Some(0xe) => Some(PciBusSpeed::PciX533_100Mhz),
                        Some(0xf) => Some(PciBusSpeed::PciX533_133Mhz),
                        _ => None,
                    }
                }
                Some(DeviceType::PciE) => {
                    match read_device_property_u32(dev_info_set, &dev_info_data, 9) {
                        Some(1) => Some(PciBusSpeed::PcieGen1),
                        Some(2) => Some(PciBusSpeed::PcieGen2),
                        Some(3) => Some(PciBusSpeed::PcieGen3),
                        Some(4) => Some(PciBusSpeed::PcieGen4),
                        Some(5) => Some(PciBusSpeed::PcieGen5),
                        Some(6) => Some(PciBusSpeed::PcieGen6),
                        _ => None, //  add future versions when available
                    }
                }
                None => None,
            };
            let max_bus_speed = match device_type {
                Some(DeviceType::Pci) => None,  //  not available on Windows
                Some(DeviceType::PciX) => None, //  not available on Windows
                Some(DeviceType::PciE) => {
                    match read_device_property_u32(dev_info_set, &dev_info_data, 11) {
                        Some(1) => Some(PciBusSpeed::PcieGen1),
                        Some(2) => Some(PciBusSpeed::PcieGen2),
                        Some(3) => Some(PciBusSpeed::PcieGen3),
                        Some(4) => Some(PciBusSpeed::PcieGen4),
                        Some(5) => Some(PciBusSpeed::PcieGen5),
                        Some(6) => Some(PciBusSpeed::PcieGen6),
                        _ => None, //  add future versions when available
                    }
                }
                None => None,
            };

            let current_link_width = match device_type {
                Some(DeviceType::PciE) => {
                    read_device_property_u32(dev_info_set, &dev_info_data, 10)
                }
                _ => None,
            };
            let max_link_width = match device_type {
                Some(DeviceType::PciE) => {
                    read_device_property_u32(dev_info_set, &dev_info_data, 12)
                }
                _ => None,
            };

            if let Some(existing_device) =
                devices.iter_mut().find(|dev| dev.inner.address == address)
            {
                let inner = &mut existing_device.inner;
                inner.vendor_id = vendor;
                inner.vendor_name = vendor_name;
                inner.device = device;
                inner.subsystem_vendor_id = subsystem_vendor;
                inner.subsystem_vendor_name = subsystem_vendor_name;
                inner.subsystem_device = subsystem_device;
                inner.class = class;
                inner.revision = revision;
                inner.driver = driver;
                inner.current_bus_speed = current_bus_speed;
                inner.max_bus_speed = max_bus_speed;
                inner.current_link_width = current_link_width;
                inner.max_link_width = max_link_width;
                inner.updated = true;
            } else {
                devices.push(new_pci_device(
                    address.clone(),
                    vendor,
                    vendor_name,
                    device,
                    subsystem_vendor,
                    subsystem_vendor_name,
                    subsystem_device,
                    class,
                    revision,
                    driver,
                    current_bus_speed,
                    max_bus_speed,
                    current_link_width,
                    max_link_width,
                ));
            }
        }

        let _ = SetupDiDestroyDeviceInfoList(dev_info_set);
    }

    if remove_not_listed_devices {
        devices.retain_mut(|device| {
            if !device.inner.updated {
                return false;
            }

            device.inner.updated = false;
            true
        });
    } else {
        for device in devices {
            device.inner.updated = false;
        }
    }
}

fn read_registry_property_u32(
    dev_info_set: HDEVINFO,
    dev_info_data: &SP_DEVINFO_DATA,
    property: SETUP_DI_REGISTRY_PROPERTY,
) -> Option<u32> {
    let mut required = std::mem::size_of::<u32>() as u32;
    let mut buffer = [0u8; std::mem::size_of::<u32>()];

    if unsafe {
        SetupDiGetDeviceRegistryPropertyW(
            dev_info_set,
            dev_info_data,
            property,
            None,
            Some(&mut buffer),
            Some(&mut required),
        )
        .is_ok()
    } {
        Some(u32::from_le_bytes(buffer))
    } else {
        None
    }
}

fn read_registry_property_vec(
    dev_info_set: HDEVINFO,
    dev_info_data: &SP_DEVINFO_DATA,
    property: SETUP_DI_REGISTRY_PROPERTY,
) -> Option<Vec<u8>> {
    let mut required: u32 = 0;

    let _ = unsafe {
        SetupDiGetDeviceRegistryPropertyW(
            dev_info_set,
            dev_info_data,
            property,
            None,
            None,
            Some(&mut required),
        )
    };

    //  a general buffer size check; increase if needed in the future
    if required == 0 || required > 512 {
        return None;
    }

    let mut buffer = vec![0u8; required as usize];

    let ret = unsafe {
        SetupDiGetDeviceRegistryPropertyW(
            dev_info_set,
            dev_info_data,
            property,
            None,
            Some(&mut buffer),
            Some(&mut required),
        )
    };
    if ret.is_err() || required as usize > buffer.len() {
        return None;
    }

    buffer.truncate(required as usize);
    Some(buffer)
}

//  Option<(vendor, device, subsystem_vendor, subsystem_device, revision)>
#[allow(clippy::type_complexity)]
fn parse_hardware_id(id: &str) -> Option<(u16, u16, Option<u16>, Option<u16>, Option<u8>)> {
    let id = id.to_ascii_uppercase();

    let id = id.strip_prefix("PCI\\")?;

    let vendor = parse_hex(id, "VEN_")? as u16;
    let device = parse_hex(id, "DEV_")? as u16;
    let subsystem = parse_hex(id, "SUBSYS_");
    let revision = parse_hex(id, "REV_").map(|v| v as u8);

    let mut subsystem_vendor = subsystem.map(|v| v as u16);
    let mut subsystem_device = subsystem.map(|v| (v >> 16) as u16);

    //  illegal vendor id
    if subsystem_vendor == Some(0x0000) || subsystem_vendor == Some(0xffff) {
        subsystem_vendor = None;
        subsystem_device = None;
    }

    if vendor != 0xffff {
        Some((vendor, device, subsystem_vendor, subsystem_device, revision))
    } else {
        None
    }
}

fn parse_hex(str: &str, prefix: &str) -> Option<u64> {
    str.split('&')
        .find_map(|v| v.strip_prefix(prefix))
        .and_then(|v| u64::from_str_radix(v, 16).ok())
}

fn read_device_property_u32(
    dev_info_set: HDEVINFO,
    dev_info_data: &SP_DEVINFO_DATA,
    property_id: u32,
) -> Option<u32> {
    //  Windows SDK pciprop.h
    //  DEFINE_PCI_DEVICE_DEVPKEY(_DevPkeyName, _Pid) macro
    let property = DEVPROPKEY {
        fmtid: GUID::from_u128(0x3ab22e31_8264_4b4e_9af5_a8d2d8e33e62),
        pid: property_id,
    };

    let mut property_type = DEVPROPTYPE::default();
    let mut required = std::mem::size_of::<u32>() as u32;
    let mut buffer = [0u8; std::mem::size_of::<u32>()];

    if unsafe {
        SetupDiGetDevicePropertyW(
            dev_info_set,
            dev_info_data,
            &property,
            &mut property_type,
            Some(&mut buffer),
            Some(&mut required),
            0,
        )
        .is_ok()
    } && property_type == DEVPROP_TYPE_UINT32
    {
        Some(u32::from_le_bytes(buffer))
    } else {
        None
    }
}

enum DeviceType {
    Pci,
    PciX,
    PciE,
}

#[allow(clippy::too_many_arguments)]
fn new_pci_device(
    address: PciAddress,
    vendor_id: u16,
    vendor_name: Option<String>,
    device: u16,
    subsystem_vendor_id: Option<u16>,
    subsystem_vendor_name: Option<String>,
    subsystem_device: Option<u16>,
    class: Option<PciDeviceClass>,
    revision: Option<u8>,
    driver: Option<String>,
    current_bus_speed: Option<PciBusSpeed>,
    max_bus_speed: Option<PciBusSpeed>,
    current_link_width: Option<u32>,
    max_link_width: Option<u32>,
) -> PciDevice {
    PciDevice {
        inner: PciDeviceInner {
            address,
            vendor_id,
            vendor_name,
            device,
            subsystem_vendor_id,
            subsystem_vendor_name,
            subsystem_device,
            class,
            revision,
            driver,
            current_bus_speed,
            max_bus_speed,
            current_link_width,
            max_link_width,
            updated: true,
        },
    }
}

#[derive(Clone)]
pub(crate) struct PciDevicesInner {
    pub(crate) pci_devices: Vec<PciDevice>,
}

impl PciDevicesInner {
    pub(crate) fn from_vec(pci_devices: Vec<PciDevice>) -> Self {
        Self { pci_devices }
    }

    pub(crate) fn into_vec(self) -> Vec<PciDevice> {
        self.pci_devices
    }

    pub(crate) fn new() -> Result<Self, Error> {
        Ok(Self {
            pci_devices: Vec::new(),
        })
    }

    pub(crate) fn list(&self) -> &[PciDevice] {
        &self.pci_devices
    }

    pub(crate) fn refresh(&mut self, remove_not_listed_interfaces: bool) {
        refresh_pci_devices(&mut self.pci_devices, remove_not_listed_interfaces);
    }
}

#[derive(Clone)]
pub(crate) struct PciDeviceInner {
    address: PciAddress,
    vendor_id: u16,
    vendor_name: Option<String>,
    device: u16,
    subsystem_vendor_id: Option<u16>,
    subsystem_vendor_name: Option<String>,
    subsystem_device: Option<u16>,
    class: Option<PciDeviceClass>,
    revision: Option<u8>,
    driver: Option<String>,
    current_bus_speed: Option<PciBusSpeed>,
    max_bus_speed: Option<PciBusSpeed>,
    current_link_width: Option<u32>,
    max_link_width: Option<u32>,
    updated: bool,
}

impl PciDeviceInner {
    pub(crate) fn address(&self) -> &PciAddress {
        &self.address
    }

    pub(crate) fn vendor_id(&self) -> u16 {
        self.vendor_id
    }

    pub(crate) fn vendor_name(&self) -> Option<&str> {
        self.vendor_name.as_deref()
    }

    pub(crate) fn device_id(&self) -> u16 {
        self.device
    }

    pub(crate) fn subsystem_vendor_id(&self) -> Option<u16> {
        self.subsystem_vendor_id
    }

    pub(crate) fn subsystem_vendor_name(&self) -> Option<&str> {
        self.subsystem_vendor_name.as_deref()
    }

    pub(crate) fn subsystem_device_id(&self) -> Option<u16> {
        self.subsystem_device
    }

    pub(crate) fn class(&self) -> Option<&PciDeviceClass> {
        self.class.as_ref()
    }

    pub(crate) fn revision(&self) -> Option<u8> {
        self.revision
    }

    pub(crate) fn driver(&self) -> Option<&str> {
        self.driver.as_deref()
    }

    pub(crate) fn current_bus_speed(&self) -> Option<PciBusSpeed> {
        self.current_bus_speed
    }

    pub(crate) fn max_bus_speed(&self) -> Option<PciBusSpeed> {
        self.max_bus_speed
    }

    pub(crate) fn current_link_width(&self) -> Option<u32> {
        self.current_link_width
    }

    pub(crate) fn max_link_width(&self) -> Option<u32> {
        self.max_link_width
    }
}
