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
use crate::unix::macos::utils::{IOReleaser, MAIN_PORT};
use crate::unix::sys::utils::read_ioreg_property_data;
use crate::utils::pci_vendor_name;
use crate::{Error, PciAddress, PciBusSpeed, PciDevice, PciDeviceClass, PciDevicesInner};
use objc2_core_foundation::{CFDictionary, CFRetained, CFString, kCFAllocatorDefault};
use objc2_io_kit::{
    IOIteratorNext, IORegistryEntryCreateCFProperty, IOServiceGetMatchingServices,
    IOServiceMatching, io_iterator_t, io_registry_entry_t, kIOReturnSuccess,
};

fn refresh_pci_devices_from_ioreg(devices: &mut Vec<PciDevice>, remove_not_listed_devices: bool) {
    unsafe {
        let Some(matching) = IOServiceMatching(c"IOPCIDevice".as_ptr()) else {
            debug!("Failed to create IOPCIDevice matching dictionary");
            return;
        };
        let matching = CFRetained::<CFDictionary>::from(&matching);

        let mut iterator: io_iterator_t = 0;
        let result = IOServiceGetMatchingServices(*MAIN_PORT, Some(matching), &mut iterator);
        if result != kIOReturnSuccess {
            debug!("Error: IOServiceGetMatchingServices() = {result}");
            return;
        }

        let Some(iterator) = IOReleaser::new(iterator) else {
            debug!("Error: IOServiceGetMatchingServices succeeded but returned invalid descriptor");
            return;
        };

        while let Some(device) = IOReleaser::new(IOIteratorNext(iterator.inner())) {
            let entry: io_registry_entry_t = device.inner();

            let Some(pci_address) = read_pci_address(entry) else {
                continue;
            };

            let Some(vendor_id) = read_property_u32(entry, "vendor-id").map(|v| v as u16) else {
                continue;
            };
            //  illegal vendor id
            if vendor_id == 0xffff {
                continue;
            }

            let Some(device_id) = read_property_u32(entry, "device-id").map(|v| v as u16) else {
                continue;
            };
            let mut subsystem_vendor_id =
                read_property_u32(entry, "subsystem-vendor-id").map(|v| v as u16);
            let mut subsystem_device_id =
                read_property_u32(entry, "subsystem-id").map(|v| v as u16);
            let class = read_property_u32(entry, "class-code").map(PciDeviceClass::from);
            let revision = read_property_u32(entry, "revision-id").map(|v| v as u8);

            //  illegal vendor id
            if subsystem_vendor_id == Some(0xffff) {
                subsystem_vendor_id = None;
                subsystem_device_id = None;
            }

            let vendor_name = pci_vendor_name(vendor_id).map(str::to_string);
            let subsystem_vendor_name = subsystem_vendor_id
                .and_then(pci_vendor_name)
                .map(str::to_string);

            if let Some(existing_device) =
                devices.iter_mut().find(|d| d.inner.address == pci_address)
            {
                let inner = &mut existing_device.inner;
                inner.vendor_id = vendor_id;
                inner.vendor_name = vendor_name;
                inner.device = device_id;
                inner.subsystem_vendor_id = subsystem_vendor_id;
                inner.subsystem_vendor_name = subsystem_vendor_name;
                inner.subsystem_device = subsystem_device_id;
                inner.class = class;
                inner.revision = revision;
                inner.updated = true;
            } else {
                devices.push(new_pci_device(
                    pci_address,
                    vendor_id,
                    vendor_name,
                    device_id,
                    subsystem_vendor_id,
                    subsystem_vendor_name,
                    subsystem_device_id,
                    class,
                    revision,
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
}

unsafe fn read_pci_address(entry: io_registry_entry_t) -> Option<PciAddress> {
    let address_key = CFString::from_str("pcidebug");
    if let Some(prop) = unsafe {
        IORegistryEntryCreateCFProperty(entry, Some(&address_key), kCFAllocatorDefault, 0)
    } && let Ok(pcidebug) = prop.downcast::<CFString>()
        && let pcidebug = pcidebug.to_string()
    {
        parse_pcidebug(pcidebug.as_str())
    } else {
        None
    }
}

unsafe fn read_property_u32(entry: io_registry_entry_t, key: &str) -> Option<u32> {
    let data = read_ioreg_property_data(entry, key)?;

    Some(u32::from_le_bytes([
        *data.first()?,
        *data.get(1)?,
        *data.get(2)?,
        *data.get(3)?,
    ]))
}

//  `pcidebug` format: either `busNum:deviceNum:functionNum` or `busNum:deviceNum:functionNum(secBusNum:subBusNum)` (decimal)
//  https://github.com/apple-oss-distributions/IOPCIFamily/blob/main/IOPCIConfigurator.cpp
fn parse_pcidebug(pcidebug: &str) -> Option<PciAddress> {
    let (address, _) = pcidebug.split_once('(').unwrap_or((pcidebug, ""));
    let mut parts = address.split(':');
    let bus: u8 = parts.next()?.parse().ok()?;
    let device: u8 = parts.next()?.parse().ok()?;
    let function: u8 = parts.next()?.parse().ok()?;

    if device > 0x1f {
        return None;
    }
    if function > 0x07 {
        return None;
    }

    Some(PciAddress {
        domain: 0,
        bus,
        device,
        function,
    })
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
        refresh_pci_devices_from_ioreg(&mut self.pci_devices, remove_not_listed_interfaces);
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
        None
    }

    pub(crate) fn current_bus_speed(&self) -> Option<PciBusSpeed> {
        None
    }

    pub(crate) fn max_bus_speed(&self) -> Option<PciBusSpeed> {
        None
    }

    pub(crate) fn current_link_width(&self) -> Option<u32> {
        None
    }

    pub(crate) fn max_link_width(&self) -> Option<u32> {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::PciAddress;
    use crate::unix::sys::pci::parse_pcidebug;

    #[test]
    fn test_parse_pcidebug() {
        assert_eq!(
            parse_pcidebug("1:2:3"),
            Some(PciAddress {
                domain: 0,
                bus: 1,
                device: 2,
                function: 3
            })
        );

        assert_eq!(
            parse_pcidebug("128:31:7"),
            Some(PciAddress {
                domain: 0,
                bus: 128,
                device: 31,
                function: 7
            })
        );

        assert_eq!(
            parse_pcidebug("1:2:3(4:5)"),
            Some(PciAddress {
                domain: 0,
                bus: 1,
                device: 2,
                function: 3
            })
        );

        assert_eq!(
            parse_pcidebug("255:31:7(1:2)"),
            Some(PciAddress {
                domain: 0,
                bus: 255,
                device: 31,
                function: 7
            })
        );

        assert_eq!(parse_pcidebug("256:31:7(1:2)"), None);
        assert_eq!(parse_pcidebug("128:32:7(1:2)"), None);
        assert_eq!(parse_pcidebug("128:31:8(1:2)"), None);
        assert_eq!(parse_pcidebug("ff:1f:07"), None);
        assert_eq!(parse_pcidebug("::"), None);
    }
}
