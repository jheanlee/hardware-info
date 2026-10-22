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

// Take a look at the license at the top of the repository in the LICENSE file.

use crate::Error;
use std::cmp::Ordering;

/// Interacting with PCI (Peripheral Component Interconnect) interfaces.
///
/// ```no_run
/// use hardware_info::PciDevices;
///
/// if let Ok(pci_devices) = PciDevices::new_with_refreshed_list() {
///     for pci_device in pci_devices.list() {
///         println!("{}", pci_device.address());
///     }
/// }
/// ```
pub struct PciDevices {
    pub(crate) inner: crate::PciDevicesInner,
}

impl PciDevices {
    /// Creates a new empty [`PciDevices`][`crate::PciDevices] type.
    ///
    /// If you want it to be filled directly, take a look at [`PciDevices::new_with_refreshed_list`].
    ///
    /// ```no_run
    /// use hardware_info::PciDevices;
    ///
    /// if let Ok(mut pci_devices) = PciDevices::new() {
    ///     pci_devices.refresh(false);
    ///     for pci_device in pci_devices.list() {
    ///         println!("{}", pci_device.address());
    ///     }
    /// }
    /// ```
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            inner: crate::PciDevicesInner::new()?,
        })
    }

    /// Creates a new [`PciDevices`][crate::PciDevices`] type with the PCI device list loaded.
    ///
    /// ```no_run
    /// use hardware_info::PciDevices;
    ///
    /// if let Ok(pci_devices) = PciDevices::new_with_refreshed_list() {
    ///     for pci_device in pci_devices.list() {
    ///         println!("{}", pci_device.address());
    ///     }
    /// }
    /// ```
    pub fn new_with_refreshed_list() -> Result<Self, Error> {
        let mut pci_devices = Self::new()?;
        pci_devices.refresh(false);
        Ok(pci_devices)
    }

    /// Returns the disks list.
    ///
    /// ```no_run
    /// use hardware_info::PciDevices;
    ///
    /// if let Ok(pci_devices) = PciDevices::new_with_refreshed_list() {
    ///     for pci_device in pci_devices.list() {
    ///         println!("{}", pci_device.address());
    ///     }
    /// }
    /// ```
    pub fn list(&self) -> &[PciDevice] {
        self.inner.list()
    }

    /// Refreshes the PCI devices' information.
    pub fn refresh(&mut self, remove_not_listed_interfaces: bool) {
        self.inner.refresh(remove_not_listed_interfaces);
    }
}

impl From<PciDevices> for Vec<PciDevice> {
    fn from(pci_devices: PciDevices) -> Self {
        pci_devices.inner.into_vec()
    }
}

impl From<Vec<PciDevice>> for PciDevices {
    fn from(pci_devices: Vec<PciDevice>) -> Self {
        Self {
            inner: crate::PciDevicesInner::from_vec(pci_devices),
        }
    }
}

impl<'a> IntoIterator for &'a PciDevices {
    type Item = &'a PciDevice;
    type IntoIter = std::slice::Iter<'a, PciDevice>;

    fn into_iter(self) -> Self::IntoIter {
        self.list().iter()
    }
}

impl std::ops::Deref for PciDevices {
    type Target = [PciDevice];

    fn deref(&self) -> &Self::Target {
        self.list()
    }
}

/// Type containing a PCI device's information.
///
/// ```no_run
/// use hardware_info::PciDevices;
///
/// if let Ok(pci_devices) = PciDevices::new_with_refreshed_list() {
///     for pci_device in pci_devices.list() {
///         println!("{}", pci_device.address());
///     }
/// }
/// ```
pub struct PciDevice {
    pub(crate) inner: crate::PciDeviceInner,
}

impl PciDevice {
    /// Returns the address of the PCI device.
    ///
    /// ```no_run
    /// use hardware_info::PciDevices;
    ///
    /// if let Ok(pci_devices) = PciDevices::new_with_refreshed_list() {
    ///     for pci_device in pci_devices.list() {
    ///         println!("{}", pci_device.address());
    ///     }
    /// }
    /// ```
    pub fn address(&self) -> &PciAddress {
        self.inner.address()
    }

    /// Returns the vendor ID of the PCI device.
    ///
    /// ```no_run
    /// use hardware_info::PciDevices;
    ///
    /// if let Ok(pci_devices) = PciDevices::new_with_refreshed_list() {
    ///     for pci_device in pci_devices.list() {
    ///         println!("{:04x}", pci_device.vendor_id());
    ///     }
    /// }
    /// ```
    pub fn vendor_id(&self) -> u16 {
        self.inner.vendor_id()
    }

    /// Returns the vendor name of the PCI device.
    ///
    /// Returns `None` if the information cannot be retrieved or cannot be not found in the database.
    ///
    /// ```no_run
    /// use hardware_info::PciDevices;
    ///
    /// if let Ok(pci_devices) = PciDevices::new_with_refreshed_list() {
    ///     for pci_device in pci_devices.list() {
    ///         println!("{:?}", pci_device.vendor_name());
    ///     }
    /// }
    /// ```
    pub fn vendor_name(&self) -> Option<&str> {
        self.inner.vendor_name()
    }

    /// Returns the device ID of the PCI device.
    ///
    /// It is often used together with the vendor ID to identify the PCI device.
    ///
    /// ```no_run
    /// use hardware_info::PciDevices;
    ///
    /// if let Ok(pci_devices) = PciDevices::new_with_refreshed_list() {
    ///     for pci_device in pci_devices.list() {
    ///         println!("{:04x}", pci_device.device_id());
    ///     }
    /// }
    /// ```
    pub fn device_id(&self) -> u16 {
        self.inner.device_id()
    }

    /// Returns the subsystem vendor ID of the PCI device.
    ///
    /// The subsystem vendor is typically the manufacturer of the subsystem (i.e. the board or the
    /// system integrating it), while the vendor is typically responsible for the device or chip.
    ///
    /// Returns `None` if the information cannot be retrieved. It is rather common for manufacturers
    /// to not provide `subsystem_*` fields.
    ///
    /// ```no_run
    /// use hardware_info::PciDevices;
    ///
    /// if let Ok(pci_devices) = PciDevices::new_with_refreshed_list() {
    ///     for pci_device in pci_devices.list() {
    ///         match pci_device.subsystem_vendor_id() {
    ///             Some(subsystem_vendor_id) => { println!("{:04x}", subsystem_vendor_id); },
    ///             None => { println!("Not available"); }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn subsystem_vendor_id(&self) -> Option<u16> {
        self.inner.subsystem_vendor_id()
    }

    /// Returns the subsystem vendor name of the PCI device.
    ///
    /// The subsystem vendor is typically the manufacturer of the subsystem (i.e. the board or the
    /// system integrating it), while the vendor is typically responsible for the device or chip.
    ///
    /// Returns `None` if the information cannot be retrieved or cannot be found in the database. It
    /// is rather common for manufacturers to not provide `subsystem_*` fields.
    ///
    /// ```no_run
    /// use hardware_info::PciDevices;
    ///
    /// if let Ok(pci_devices) = PciDevices::new_with_refreshed_list() {
    ///     for pci_device in pci_devices.list() {
    ///         println!("{:?}", pci_device.subsystem_vendor_name());
    ///     }
    /// }
    /// ```
    pub fn subsystem_vendor_name(&self) -> Option<&str> {
        self.inner.subsystem_vendor_name()
    }

    /// Returns the subsystem device ID of the PCI device.
    ///
    /// It is often used together with the subsystem vendor id to identify the system or board
    /// implementing the PCI device.
    ///
    /// Returns `None` if the information cannot be retrieved. It is rather common for manufacturers
    /// to not provide `subsystem_*` fields.
    ///
    /// ```no_run
    /// use hardware_info::PciDevices;
    ///
    /// if let Ok(pci_devices) = PciDevices::new_with_refreshed_list() {
    ///     for pci_device in pci_devices.list() {
    ///         match pci_device.subsystem_device_id() {
    ///             Some(subsystem_device_id) => { println!("{:04x}", subsystem_device_id); },
    ///             None => { println!("Not available"); }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn subsystem_device_id(&self) -> Option<u16> {
        self.inner.subsystem_device_id()
    }

    /// Returns the device class (i.e. type) of the PCI device.
    ///
    /// Returns `None` if the information cannot be retrieved.
    ///
    /// ```no_run
    /// use hardware_info::PciDevices;
    ///
    /// if let Ok(pci_devices) = PciDevices::new_with_refreshed_list() {
    ///     for pci_device in pci_devices.list() {
    ///         match pci_device.class() {
    ///             Some(class) => { println!("{class}"); },
    ///             None => { println!("Not available"); }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn class(&self) -> Option<&PciDeviceClass> {
        self.inner.class()
    }

    /// Returns the device revision of the PCI device.
    ///
    /// Returns `None` if the information cannot be retrieved.
    ///
    /// ```no_run
    /// use hardware_info::PciDevices;
    ///
    /// if let Ok(pci_devices) = PciDevices::new_with_refreshed_list() {
    ///     for pci_device in pci_devices.list() {
    ///         println!("{:?}", pci_device.revision());
    ///     }
    /// }
    /// ```
    pub fn revision(&self) -> Option<u8> {
        self.inner.revision()
    }

    /// Returns the driver used by the PCI device.
    ///
    /// On Linux, the value is the absolute path to the driver.
    ///
    /// On NetBSD, the value is the name of the driver.
    ///
    /// On Windows, the value is the kernel driver service.
    ///
    /// On supported systems, `None` means that this PCI device does not have a driver (for example,
    /// host bridges).
    ///
    /// ⚠️ Only implemented on Linux, NetBSD and Windows. This always returns `None` on other systems.
    ///
    /// ```no_run
    /// use hardware_info::PciDevices;
    ///
    /// if let Ok(pci_devices) = PciDevices::new_with_refreshed_list() {
    ///     for pci_device in pci_devices.list() {
    ///         println!("{:?}", pci_device.driver());
    ///     }
    /// }
    /// ```
    pub fn driver(&self) -> Option<&str> {
        self.inner.driver()
    }

    /// Returns the negotiated bus speed of the device.
    ///
    /// Returns `None` if the information cannot be retrieved.
    ///
    /// ⚠️ Only implemented on Linux. This always returns `None` on other systems.
    ///
    /// ```no_run
    /// use hardware_info::PciDevices;
    ///
    /// if let Ok(pci_devices) = PciDevices::new_with_refreshed_list() {
    ///     for pci_device in pci_devices.list() {
    ///         println!("{:?}", pci_device.current_bus_speed());
    ///     }
    /// }
    /// ```
    pub fn current_bus_speed(&self) -> Option<PciBusSpeed> {
        self.inner.current_bus_speed()
    }

    /// Returns the maximum bus speed of the device.
    ///
    /// Returns `None` if the information cannot be retrieved.
    ///
    /// ⚠️ Only implemented on Linux. This always returns `None` on other systems.
    ///
    /// ```no_run
    /// use hardware_info::PciDevices;
    ///
    /// if let Ok(pci_devices) = PciDevices::new_with_refreshed_list() {
    ///     for pci_device in pci_devices.list() {
    ///         println!("{:?}", pci_device.max_bus_speed());
    ///     }
    /// }
    /// ```
    pub fn max_bus_speed(&self) -> Option<PciBusSpeed> {
        self.inner.max_bus_speed()
    }

    /// Returns the negotiated PCIe link width (lanes).
    ///
    /// Returns `None` if this device is not a PCIe device or if the information cannot be retrieved.
    ///
    /// ⚠️ Only implemented on Linux. This always returns `None` on other systems.
    ///
    /// ```no_run
    /// use hardware_info::PciDevices;
    ///
    /// if let Ok(pci_devices) = PciDevices::new_with_refreshed_list() {
    ///     for pci_device in pci_devices.list() {
    ///         println!("{:?}", pci_device.current_link_width());
    ///     }
    /// }
    /// ```
    pub fn current_link_width(&self) -> Option<u32> {
        self.inner.current_link_width()
    }

    /// Returns the maximum PCIe link width (lanes) supported by the device.
    ///
    /// Returns `None` if this device is not a PCIe device or if the information cannot be retrieved.
    ///
    /// ⚠️ Only implemented on Linux. This always returns `None` on other systems.
    ///
    /// ```no_run
    /// use hardware_info::PciDevices;
    ///
    /// if let Ok(pci_devices) = PciDevices::new_with_refreshed_list() {
    ///     for pci_device in pci_devices.list() {
    ///         println!("{:?}", pci_device.max_link_width());
    ///     }
    /// }
    /// ```
    pub fn max_link_width(&self) -> Option<u32> {
        self.inner.max_link_width()
    }
}

impl PartialEq for PciDevice {
    fn eq(&self, other: &Self) -> bool {
        self.address() == other.address()
    }
}

impl Eq for PciDevice {}

impl PartialOrd for PciDevice {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PciDevice {
    fn cmp(&self, other: &Self) -> Ordering {
        self.address().cmp(other.address())
    }
}

/// Unique id for PCI devices on a system.
#[cfg(feature = "pci")]
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PciAddress {
    pub(crate) domain: u32,
    pub(crate) bus: u8,      //  8 bits
    pub(crate) device: u8,   //  5 bits
    pub(crate) function: u8, //  3 bits
}

impl PciAddress {
    /// The OS-defined PCI domain. (Sometimes known as the segment).
    ///
    /// Returns `0` on macOS, NetBSD and Windows.
    pub fn domain(&self) -> u32 {
        self.domain
    }

    /// The PCI bus.
    pub fn bus(&self) -> u8 {
        self.bus
    }

    /// The PCI device.
    pub fn device(&self) -> u8 {
        self.device
    }

    /// The PCI function
    pub fn function(&self) -> u8 {
        self.function
    }
}

#[cfg(feature = "pci")]
impl std::fmt::Display for PciAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let Self {
            domain,
            bus,
            device,
            function,
        } = self;
        write!(f, "{domain:08x}:{bus:02x}:{device:02x}.{function}")
    }
}

//  Adapted from `sysinfo`:
//  https://github.com/GuillaumeGomez/sysinfo
//  Original code licensed under the MIT License. See `THIRD-PARTY-LICENSES` at the project root.
#[cfg(feature = "pci")]
impl core::str::FromStr for PciAddress {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn get_next_u32<'a>(
            iter: &mut impl Iterator<Item = &'a str>,
            missing_msg: &'static str,
            invalid_msg: &'static str,
        ) -> Result<u32, &'static str> {
            let Some(value) = iter.next() else {
                return Err(missing_msg);
            };
            u32::from_str_radix(value, 16).map_err(|_| invalid_msg)
        }

        let mut iter = s.split(':');
        let domain = get_next_u32(&mut iter, "missing domain", "invalid domain")?;
        let bus = get_next_u32(&mut iter, "missing bus", "invalid bus")?;
        let Some(last) = iter.next() else {
            return Err("missing device");
        };
        if iter.next().is_some() {
            return Err("unexpected `:` after bus");
        };
        let mut iter = last.split('.');
        let device = get_next_u32(&mut iter, "missing device", "invalid device")?;
        let function = get_next_u32(&mut iter, "missing function", "invalid function")?;
        if iter.next().is_some() {
            return Err("unexpected `:` after function");
        };

        if bus > 0xff {
            return Err("invalid bus");
        }
        if device > 0x1f {
            return Err("invalid device");
        }
        if function > 0x07 {
            return Err("invalid function");
        }

        Ok(Self {
            domain,
            bus: bus as u8,
            device: device as u8,
            function: function as u8,
        })
    }
}

/// Class information for a PCI device.
///
/// PCI device classes are organized hierarchically: class > subclass > programming_interface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PciDeviceClass {
    /// The PCI device class, identifying the general function of the PCI device.
    pub class: u8,
    /// The PCI device subclass, providing specific classification within the class.
    pub subclass: u8,
    /// The programming_interface, providing specific classification within the subclass.
    pub programming_interface: u8,
}

impl From<u32> for PciDeviceClass {
    fn from(value: u32) -> Self {
        Self {
            class: ((value >> 16) & 0xff) as u8,
            subclass: ((value >> 8) & 0xff) as u8,
            programming_interface: (value & 0xff) as u8,
        }
    }
}

impl From<PciDeviceClass> for u32 {
    fn from(value: PciDeviceClass) -> Self {
        u32::from(&value)
    }
}

impl From<&PciDeviceClass> for u32 {
    fn from(value: &PciDeviceClass) -> Self {
        ((value.class as u32) << 16)
            | ((value.subclass as u32) << 8)
            | (value.programming_interface as u32)
    }
}

impl std::fmt::Display for PciDeviceClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:06x}", u32::from(self))
    }
}

/// PCI bus type and speed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PciBusSpeed {
    /// Legacy PCI, clock speed 33 Mhz
    Pci33Mhz,
    /// Legacy PCI, clock speed 66 Mhz
    Pci66Mhz,
    /// PCI-X, clock speed 66 Mhz
    PciX66Mhz,
    /// PCI-X, clock speed 100 Mhz
    PciX100Mhz,
    /// PCI-X, clock speed 133 Mhz
    PciX133Mhz,
    /// PCI-X 266, clock speed 66 Mhz
    PciX266_66Mhz,
    /// PCI-X 266, clock speed 100 Mhz
    PciX266_100Mhz,
    /// PCI-X 266, clock speed 133 Mhz
    PciX266_133Mhz,
    /// PCI-X 533, clock speed 66 Mhz
    PciX533_66Mhz,
    /// PCI-X 533, clock speed 100 Mhz
    PciX533_100Mhz,
    /// PCI-X 533, clock speed 133 Mhz
    PciX533_133Mhz,
    /// AGP, unknown transfer rate
    AgpUnknown,
    /// AGP, 1x transfer rate (266 MB/s)
    Agp1x,
    /// AGP, 2x transfer rate (533 MB/s)
    Agp2x,
    /// AGP, 4x transfer rate (1066 MB/s)
    Agp4x,
    /// AGP, 8x transfer rate (2133 MB/s)
    Agp8x,
    /// PCIe, generation 1 (2.5 GT/s per lane)
    PcieGen1,
    /// PCIe, generation 2 (5.0 GT/s per lane)
    PcieGen2,
    /// PCIe, generation 3 (8.0 GT/s per lane)
    PcieGen3,
    /// PCIe, generation 4 (16.0 GT/s per lane)
    PcieGen4,
    /// PCIe, generation 5 (32.0 GT/s per lane)
    PcieGen5,
    /// PCIe, generation 6 (64.0 GT/s per lane)
    PcieGen6,
    /// PCIe, future/unknown generation. Contains the transfer rate in megatransfer per second (MT/s) per lane
    PcieOther(u32),
}

impl PciBusSpeed {
    /// Returns the transfer rate per lane corresponding to the PCIe generation.
    ///
    /// Returns `None` for non-PCIe values.
    pub fn pcie_transfer_rate(self) -> Option<u32> {
        match self {
            PciBusSpeed::PcieGen1 => Some(2_500),
            PciBusSpeed::PcieGen2 => Some(5_000),
            PciBusSpeed::PcieGen3 => Some(8_000),
            PciBusSpeed::PcieGen4 => Some(16_000),
            PciBusSpeed::PcieGen5 => Some(32_000),
            PciBusSpeed::PcieGen6 => Some(64_000),
            PciBusSpeed::PcieOther(rate) => Some(rate),
            _ => None,
        }
    }
}
