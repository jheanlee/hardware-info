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

#[cfg(feature = "pci")]
fn should_skip_test() -> bool {
    match hardware_info::PciDevices::new_with_refreshed_list() {
        Ok(_) => false,
        Err(error) => {
            if !matches!(error, hardware_info::Error::Unsupported) {
                panic!("Expected `Error::Unsupported`, found {error:?}");
            }
            true
        }
    }
}

#[cfg(feature = "pci")]
#[test]
fn test_pci() {
    if should_skip_test() {
        return;
    }

    let mut devices = hardware_info::PciDevices::new().unwrap();
    assert!(devices.is_empty());
    devices.refresh(false);
    assert!(!devices.is_empty());

    for device in devices.list() {
        assert_ne!(device.vendor_id(), 0xffff);
        assert_ne!(device.subsystem_vendor_id(), Some(0xffff));
    }
}
