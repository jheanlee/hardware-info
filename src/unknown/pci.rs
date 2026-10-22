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

use crate::{Error, PciAddress, PciBusSpeed, PciDevice, PciDeviceClass};

pub(crate) struct PciDevicesInner;

impl PciDevicesInner {
    pub(crate) fn new() -> Result<Self, Error> {
        Err(Error::Unsupported)
    }

    pub(crate) fn from_vec(_: Vec<PciDevice>) -> Self {
        Self
    }

    pub(crate) fn into_vec(self) -> Vec<PciDevice> {
        Vec::new()
    }

    pub(crate) fn list(&self) -> &[PciDevice] {
        &[]
    }

    pub(crate) fn refresh(&mut self, _: bool) {}
}

pub(crate) struct PciDeviceInner {}

impl PciDeviceInner {
    pub(crate) fn address(&self) -> &PciAddress {
        unreachable!()
    }

    pub(crate) fn vendor_id(&self) -> u16 {
        unreachable!()
    }

    pub(crate) fn vendor_name(&self) -> Option<&str> {
        unreachable!()
    }

    pub(crate) fn device_id(&self) -> u16 {
        unreachable!()
    }

    pub(crate) fn subsystem_vendor_id(&self) -> Option<u16> {
        unreachable!()
    }

    pub(crate) fn subsystem_vendor_name(&self) -> Option<&str> {
        unreachable!()
    }

    pub(crate) fn subsystem_device_id(&self) -> Option<u16> {
        unreachable!()
    }

    pub(crate) fn class(&self) -> Option<&PciDeviceClass> {
        unreachable!()
    }

    pub(crate) fn revision(&self) -> Option<u8> {
        unreachable!()
    }

    pub(crate) fn driver(&self) -> Option<&str> {
        unreachable!()
    }

    pub(crate) fn current_bus_speed(&self) -> Option<PciBusSpeed> {
        unreachable!()
    }

    pub(crate) fn max_bus_speed(&self) -> Option<PciBusSpeed> {
        unreachable!()
    }

    pub(crate) fn current_link_width(&self) -> Option<u32> {
        unreachable!()
    }

    pub(crate) fn max_link_width(&self) -> Option<u32> {
        unreachable!()
    }
}
