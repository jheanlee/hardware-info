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

        // Use kIOMasterPortDefault on macOS to support older OS versions.
        #[allow(deprecated)]
        #[cfg(target_os = "macos")]
        pub(crate) static MAIN_PORT: &libc::mach_port_t = unsafe { &objc2_io_kit::kIOMasterPortDefault };

        // iOS, watchOS, tvOS and visionOS only have the newer symbol.
        #[cfg(not(target_os = "macos"))]
        pub(crate) static MAIN_PORT: &libc::mach_port_t = unsafe { &objc2_io_kit::kIOMainPortDefault };
    }
    _ => {}
}
