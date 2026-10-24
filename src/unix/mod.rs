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

cfg_select! {
    any(target_os = "macos") => {
        pub(crate) mod macos;
        pub(crate) use macos as sys;
    }
    any(target_os = "linux", target_os = "android") => {
        pub(crate) mod linux;
        pub(crate) use linux as sys;
    }
    any(target_os = "freebsd", target_os = "netbsd") => {
        pub(crate) mod bsd;
        pub(crate) use bsd as sys;
    }
    _ => {
        compile_error!("Invalid cfg!");
    }
}

cfg_select! {
    feature = "pci" => {
        #[derive(Clone)]
        pub(crate) struct PciDevicesInner {
            pub(crate) pci_devices: Vec<crate::PciDevice>,
        }

        impl PciDevicesInner {
            pub(crate) fn into_vec(self) -> Vec<crate::PciDevice> {
                self.pci_devices
            }
        }
    }
    _ => {}
}

// Make formattable by rustfmt.
#[cfg(any())]
mod bsd;
#[cfg(any())]
mod linux;
#[cfg(any())]
mod macos;
