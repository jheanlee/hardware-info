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

use crate::{Error, PciAddress, PciDevice, PciDeviceClass};

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
        todo!()
    }
}

pub(crate) struct PciDeviceInner {}

impl PciDeviceInner {
    pub(crate) fn address(&self) -> &PciAddress {
        todo!()
    }

    pub(crate) fn vendor_id(&self) -> Option<u16> {
        todo!()
    }

    pub(crate) fn vendor_name(&self) -> Option<&str> {
        todo!()
    }

    pub(crate) fn device_id(&self) -> Option<u16> {
        todo!()
    }

    pub(crate) fn subsystem_vendor_id(&self) -> Option<u16> {
        todo!()
    }

    pub(crate) fn subsystem_vendor_name(&self) -> Option<&str> {
        todo!()
    }

    pub(crate) fn subsystem_device_id(&self) -> Option<u16> {
        todo!()
    }

    pub(crate) fn class(&self) -> Option<&PciDeviceClass> {
        todo!()
    }

    pub(crate) fn revision(&self) -> Option<u8> {
        todo!()
    }

    pub(crate) fn driver(&self) -> Option<&str> {
        todo!()
    }
}
