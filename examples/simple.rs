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

//! An example of listing all PCI devices and their addresses every 5 seconds.

fn main() {
    let Ok(mut pci_devices) = hardware_info::PciDevices::new() else {
        return;
    };

    loop {
        pci_devices.refresh(true); //  Refresh PCI information and remove devices that are no longer present.
        let mut pci_devices = pci_devices.list().to_vec();
        pci_devices.sort();

        println!("PCI devices: ");
        for pci_device in pci_devices {
            println!("{}", pci_device.address())
        }

        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
