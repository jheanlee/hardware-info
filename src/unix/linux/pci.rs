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

use crate::sys::utils::{read_hex, read_str, read_unsigned};
use crate::utils::pci_vendor_name;
use crate::{Error, PciAddress, PciBusSpeed, PciDevice, PciDeviceClass, PciDevicesInner};
use std::path::Path;

fn refresh_pci_devices_list_from_sysfs(
    devices: &mut Vec<PciDevice>,
    remove_not_listed_devices: bool,
    sysfs_bus_pci_devices: &Path,
) {
    let Ok(sys_dir) = std::fs::read_dir(sysfs_bus_pci_devices) else {
        return;
    };

    let mut num_buf = [0u8; (u64::MAX.ilog10() + 2) as usize + 2];
    let mut str_buf = Vec::with_capacity(32);

    for entry in sys_dir.flatten() {
        let parent = &entry.path();
        let Some(address) = &entry
            .file_name()
            .to_string_lossy()
            .parse::<PciAddress>()
            .ok()
        else {
            continue;
        };

        let Some(vendor) = read_hex(parent, "vendor", &mut num_buf).map(|v| v as u16) else {
            continue;
        };
        //  illegal vendor id
        if vendor == 0xffff {
            continue;
        }
        let Some(device) = read_hex(parent, "device", &mut num_buf).map(|v| v as u16) else {
            continue;
        };
        let mut subsystem_vendor =
            read_hex(parent, "subsystem_vendor", &mut num_buf).map(|v| v as u16);
        let mut subsystem_device =
            read_hex(parent, "subsystem_device", &mut num_buf).map(|v| v as u16);
        let class = read_hex(parent, "class", &mut num_buf).map(|v| PciDeviceClass::from(v as u32));
        let revision = read_hex(parent, "revision", &mut num_buf).map(|v| v as u8);

        let driver = std::fs::canonicalize(entry.path().join("driver"))
            .map(|v| v.to_string_lossy().to_string())
            .ok();

        let current_link_speed =
            str::from_utf8(read_str(parent, "current_link_speed", &mut str_buf))
                .ok()
                .and_then(parse_pci_link_speed);
        let max_link_speed = str::from_utf8(read_str(parent, "max_link_speed", &mut str_buf))
            .ok()
            .and_then(parse_pci_link_speed);
        let mut current_link_width =
            read_unsigned(parent, "current_link_width", &mut num_buf).map(|v| v as u32);
        let mut max_link_width =
            read_unsigned(parent, "max_link_width", &mut num_buf).map(|v| v as u32);

        if current_link_width == Some(0) || current_link_width == Some(255) {
            current_link_width = None;
        }
        if max_link_width == Some(0) || max_link_width == Some(255) {
            max_link_width = None;
        }

        let vendor_name = pci_vendor_name(vendor).map(str::to_string);
        let mut subsystem_vendor_name = subsystem_vendor
            .and_then(pci_vendor_name)
            .map(str::to_string);

        //  illegal vendor id
        if subsystem_vendor == Some(0xffff) {
            subsystem_vendor = None;
            subsystem_vendor_name = None;
            subsystem_device = None;
        }

        if let Some(existing_device) = devices.iter_mut().find(|d| d.inner.address == *address) {
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
            inner.current_bus_speed = current_link_speed;
            inner.max_bus_speed = max_link_speed;
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
                current_link_speed,
                max_link_speed,
                current_link_width,
                max_link_width,
            ));
        }
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

fn parse_pci_link_speed(s: &str) -> Option<PciBusSpeed> {
    match s.trim() {
        "33 MHz PCI" => Some(PciBusSpeed::Pci33Mhz),
        "66 MHz PCI" => Some(PciBusSpeed::Pci66Mhz),
        "66 MHz PCI-X" => Some(PciBusSpeed::PciX66Mhz),
        "100 MHz PCI-X" => Some(PciBusSpeed::PciX100Mhz),
        "133 MHz PCI-X" => Some(PciBusSpeed::PciX133Mhz),
        "66 MHz PCI-X 266" => Some(PciBusSpeed::PciX266_66Mhz),
        "100 MHz PCI-X 266" => Some(PciBusSpeed::PciX266_100Mhz),
        "133 MHz PCI-X 266" => Some(PciBusSpeed::PciX266_133Mhz),
        "Unknown AGP" => Some(PciBusSpeed::AgpUnknown),
        "1x AGP" => Some(PciBusSpeed::Agp1x),
        "2x AGP" => Some(PciBusSpeed::Agp2x),
        "4x AGP" => Some(PciBusSpeed::Agp4x),
        "8x AGP" => Some(PciBusSpeed::Agp8x),
        "66 MHz PCI-X 533" => Some(PciBusSpeed::PciX533_66Mhz),
        "100 MHz PCI-X 533" => Some(PciBusSpeed::PciX533_100Mhz),
        "133 MHz PCI-X 533" => Some(PciBusSpeed::PciX533_133Mhz),
        "2.5 GT/s PCIe" => Some(PciBusSpeed::PcieGen1),
        "5.0 GT/s PCIe" => Some(PciBusSpeed::PcieGen2),
        "8.0 GT/s PCIe" => Some(PciBusSpeed::PcieGen3),
        "16.0 GT/s PCIe" => Some(PciBusSpeed::PcieGen4),
        "32.0 GT/s PCIe" => Some(PciBusSpeed::PcieGen5),
        "64.0 GT/s PCIe" => Some(PciBusSpeed::PcieGen6),
        s => {
            //  try parse PCIe speed for future versions
            let (whole, fraction) = s.split_once(" GT/s PCIe")?.0.split_once('.')?;
            let mtps = whole
                .parse::<u32>()
                .ok()?
                .saturating_mul(1000)
                .saturating_add(fraction.parse::<u32>().ok()?.saturating_mul(100));
            Some(PciBusSpeed::PcieOther(mtps))
        }
    }
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

impl PciDevicesInner {
    pub(crate) fn new() -> Result<Self, Error> {
        Ok(Self {
            pci_devices: Vec::new(),
        })
    }

    pub(crate) fn list(&self) -> &[PciDevice] {
        &self.pci_devices
    }

    pub(crate) fn refresh(&mut self, remove_not_listed_interfaces: bool) {
        refresh_pci_devices_list_from_sysfs(
            &mut self.pci_devices,
            remove_not_listed_interfaces,
            "/sys/bus/pci/devices".as_ref(),
        );
    }
}

#[derive(Clone)]
pub(crate) struct PciDeviceInner {
    address: PciAddress,
    vendor_id: u16,
    vendor_name: Option<String>,
    subsystem_vendor_id: Option<u16>,
    subsystem_vendor_name: Option<String>,
    device: u16,
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
