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
pub(crate) fn parse_string(bytes: &[u8]) -> Option<String> {
    let words: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .take_while(|&word| word != 0)
        .collect();

    String::from_utf16(&words).ok()
}

#[cfg(feature = "pci")]
pub(crate) fn parse_multi_string(bytes: &[u8]) -> Vec<String> {
    let words: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    words
        .split(|&word| word == 0)
        .filter(|v| !v.is_empty())
        .filter_map(|v| String::from_utf16(v).ok())
        .collect()
}

cfg_select! {
    any (
        feature = "pci",
        feature = "battery"
    ) => {
        use windows::Win32::Devices::DeviceAndDriverInstallation::{SetupDiDestroyDeviceInfoList, HDEVINFO};
        pub(crate) struct DeviceInfoReleaser(HDEVINFO);

        impl DeviceInfoReleaser {
            pub(crate) fn new(hdevinfo: HDEVINFO) -> Self {
                Self(hdevinfo)
            }

            #[inline]
            pub(crate) fn inner(&self) -> HDEVINFO {
                self.0
            }
        }

        impl Drop for DeviceInfoReleaser {
            fn drop(&mut self) {
                unsafe {
                    let _ = SetupDiDestroyDeviceInfoList(self.0);
                }
            }
        }
    },
    _ => {}
}

cfg_select! {
    any (
        feature = "pci",
        feature = "battery"
    ) => {
        use windows::Win32::Foundation::{CloseHandle, HANDLE};
        pub(crate) struct FileHandleReleaser(HANDLE);

        impl FileHandleReleaser {
            pub(crate) fn new(handle: HANDLE) -> Self {
                Self(handle)
            }

            #[inline]
            pub(crate) fn inner(&self) -> HANDLE {
                self.0
            }
        }

        impl Drop for FileHandleReleaser {
            fn drop(&mut self) {
                unsafe {
                    let _ = CloseHandle(self.0);
                }
            }
        }
    },
    _ => {}
}
