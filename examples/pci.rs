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

//! An example of listing all PCI devices and their information.

use hardware_info::PciDevices;

fn format_optional<T: std::fmt::Display>(value: Option<T>) -> String {
    value.map_or_else(|| "None".to_string(), |value| value.to_string())
}

fn format_optional_hex(value: Option<impl std::fmt::LowerHex>) -> String {
    value.map_or_else(|| "None".to_string(), |value| format!("{value:04x}"))
}

fn main() {
    let Ok(pci_devices) = PciDevices::new_with_refreshed_list() else {
        println!("PCI is not supported on this OS.");
        return;
    };

    for pci_device in pci_devices.list() {
        println!("========== Device {} ==========", pci_device.address());

        println!(
            "Vendor: {:04x} ({})",
            pci_device.vendor_id(),
            pci_device.vendor_name().unwrap_or("name not available")
        );
        println!("Device ID: {:04x}", pci_device.device_id());

        if let Some(substem_vendor_id) = pci_device.subsystem_vendor_id() {
            println!(
                "Subsystem vendor: {substem_vendor_id} ({})",
                pci_device
                    .subsystem_vendor_name()
                    .unwrap_or("name not available")
            );
            println!(
                "Subsystem device ID: {}",
                format_optional_hex(pci_device.subsystem_device_id())
            );
        } else {
            println!("Subsystem vendor: None");
            println!("Subsystem device ID: None");
        }

        println!("Revision: {}", format_optional(pci_device.revision()));
        println!("Device class: {}", format_optional(pci_device.class()));
        println!("Driver: {}", format_optional(pci_device.driver()));

        let current_rate_per_lane = pci_device
            .current_bus_speed()
            .and_then(|speed| speed.pcie_transfer_rate());
        let current_link_width = pci_device.current_link_width();
        let max_rate_per_lane = pci_device
            .max_bus_speed()
            .and_then(|speed| speed.pcie_transfer_rate());
        let max_link_width = pci_device.max_link_width();

        if let Some(current_rate_per_lane) = current_rate_per_lane
            && let Some(current_link_width) = current_link_width
            && let Some(max_rate_per_lane) = max_rate_per_lane
            && let Some(max_link_width) = max_link_width
        {
            println!(
                "PCIe bus total speed: {}/{} MT/s",
                current_rate_per_lane * current_link_width,
                max_rate_per_lane * max_link_width
            );
        } else {
            println!("PCIe bus total speed: not available");
        }
    }
}
