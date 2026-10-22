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
    }
}
