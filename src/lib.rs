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

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../README.md")]

#[macro_use]
mod macros;

cfg_select! {
    feature = "unknown-ci" => {
        // This is used in CI to check that the build for unknown targets is compiling fine.
        mod unknown;
        use crate::unknown as sys;

        #[cfg(test)]
        pub(crate) const MIN_USERS: usize = 0;
    }
    any(
        target_os = "macos", target_os = "linux",
        target_os = "freebsd", target_os = "netbsd") =>
    {
        mod unix;
        use crate::unix::sys as sys;
    }
    windows => {
        mod windows;
        use crate::windows as sys;
    }
    _ => {
        mod unknown;
        use crate::unknown as sys;
    }
}

pub use crate::sys::IS_SUPPORTED_SYSTEM;

#[cfg(feature = "pci")]
pub use crate::common::pci::{PciAddress, PciDevice, PciDeviceClass, PciDevices};
pub use crate::error::Error;

#[cfg(feature = "pci")]
pub(crate) use crate::sys::{PciDeviceInner, PciDevicesInner};

mod common;
mod error;
pub(crate) mod utils;

// Make formattable by rustfmt.
#[cfg(any())]
mod unix;
#[cfg(any())]
mod unknown;
#[cfg(any())]
mod windows;

mod tests {}
