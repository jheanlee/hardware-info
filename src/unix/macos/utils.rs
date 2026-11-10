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

// Use kIOMasterPortDefault on macOS to support older OS versions.
#[allow(deprecated)]
#[cfg(all(target_os = "macos", any(feature = "pci", feature = "battery")))]
pub(crate) static MAIN_PORT: &libc::mach_port_t = unsafe { &objc2_io_kit::kIOMasterPortDefault };

// iOS, watchOS, tvOS and visionOS only have the newer symbol.
#[cfg(all(not(target_os = "macos"), any(feature = "pci", feature = "battery")))]
pub(crate) static MAIN_PORT: &libc::mach_port_t = unsafe { &objc2_io_kit::kIOMainPortDefault };

//  Adapted from `sysinfo`:
//  https://github.com/GuillaumeGomez/sysinfo
//  Original code licensed under the MIT License. See `THIRD-PARTY-LICENSES` at the project root.
cfg_select! {
    any(
        feature = "pci"
    ) => {
        type IoObject = std::num::NonZeroU32;

        pub(crate) struct IOReleaser(IoObject);

        impl IOReleaser {
            pub(crate) fn new(obj: u32) -> Option<Self> {
                IoObject::new(obj).map(Self)
            }

            #[inline]
            pub(crate) fn inner(&self) -> u32 {
                self.0.get()
            }
        }

        impl Drop for IOReleaser {
            fn drop(&mut self) {
                objc2_io_kit::IOObjectRelease(self.0.get() as _);
            }
        }
    },
    _ => {}
}

cfg_select! {
    target_arch = "aarch64" => {
        pub(crate) const IS_ARM: bool = true;
    },
    _ => {
        pub(crate) const IS_ARM: bool = false;
    }
}

#[cfg(any(feature = "pci", feature = "battery"))]
pub(crate) fn read_ioreg_property_data(
    entry: objc2_io_kit::io_registry_entry_t,
    key: &str,
) -> Option<Vec<u8>> {
    unsafe {
        let key = objc2_core_foundation::CFString::from_str(key);
        let property = objc2_io_kit::IORegistryEntryCreateCFProperty(
            entry,
            Some(&key),
            objc2_core_foundation::kCFAllocatorDefault,
            0,
        )?;
        let property = property.downcast::<objc2_core_foundation::CFData>().ok()?;
        let property = property.as_bytes_unchecked();

        if !property.is_empty() {
            Some(property.to_vec())
        } else {
            None
        }
    }
}

#[cfg(feature = "battery")]
pub(crate) fn read_ioreg_property_string(
    entry: objc2_io_kit::io_registry_entry_t,
    key: &str,
) -> Option<String> {
    unsafe {
        let key = objc2_core_foundation::CFString::from_str(key);
        let property = objc2_io_kit::IORegistryEntryCreateCFProperty(
            entry,
            Some(&key),
            objc2_core_foundation::kCFAllocatorDefault,
            0,
        )?;

        Some(
            property
                .downcast::<objc2_core_foundation::CFString>()
                .ok()?
                .to_string(),
        )
    }
}

#[cfg(feature = "battery")]
pub(crate) fn read_ioreg_property_i64(
    entry: objc2_io_kit::io_registry_entry_t,
    key: &str,
) -> Option<i64> {
    unsafe {
        let key = objc2_core_foundation::CFString::from_str(key);
        let property = objc2_io_kit::IORegistryEntryCreateCFProperty(
            entry,
            Some(&key),
            objc2_core_foundation::kCFAllocatorDefault,
            0,
        )?;

        property
            .downcast::<objc2_core_foundation::CFNumber>()
            .ok()?
            .as_i64()
    }
}

#[cfg(feature = "battery")]
pub(crate) fn read_ioreg_property_bool(
    entry: objc2_io_kit::io_registry_entry_t,
    key: &str,
) -> Option<bool> {
    unsafe {
        let key = objc2_core_foundation::CFString::from_str(key);
        let property = objc2_io_kit::IORegistryEntryCreateCFProperty(
            entry,
            Some(&key),
            objc2_core_foundation::kCFAllocatorDefault,
            0,
        )?;

        Some(
            property
                .downcast::<objc2_core_foundation::CFBoolean>()
                .ok()?
                .as_bool(),
        )
    }
}

#[cfg(feature = "battery")]
pub(crate) fn get_cf_dictionary_string(
    dict: &objc2_core_foundation::CFRetained<objc2_core_foundation::CFDictionary>,
    key: &str,
) -> Option<String> {
    unsafe {
        let key = objc2_core_foundation::CFString::from_str(key);
        let value = dict.value(&*key as *const objc2_core_foundation::CFString as _);
        if value.is_null() {
            return None;
        }

        Some(
            (&*(value as *const objc2_core_foundation::CFType))
                .downcast_ref::<objc2_core_foundation::CFString>()?
                .to_string(),
        )
    }
}

#[cfg(feature = "battery")]
pub(crate) fn get_cf_dictionary_i64(
    dict: &objc2_core_foundation::CFRetained<objc2_core_foundation::CFDictionary>,
    key: &str,
) -> Option<i64> {
    unsafe {
        let key = objc2_core_foundation::CFString::from_str(key);
        let value = dict.value(&*key as *const objc2_core_foundation::CFString as _);
        if value.is_null() {
            return None;
        }

        (&*(value as *const objc2_core_foundation::CFType))
            .downcast_ref::<objc2_core_foundation::CFNumber>()?
            .as_i64()
    }
}
