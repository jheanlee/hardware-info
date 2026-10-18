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
use std::io::Read;

#[cfg(feature = "pci")]
pub(crate) fn read_hex<P: AsRef<std::path::Path>>(
    parent: P,
    path: &str,
    data: &mut [u8],
) -> Option<u64> {
    if let Ok(mut f) = std::fs::File::open(parent.as_ref().join(path))
        && let Ok(size) = f.read(data)
        && size >= 3
        && data.len() >= 3
        && let Ok(str) = str::from_utf8(&data[..size])
        && let Some(str) = str.strip_prefix("0x")
    {
        return u64::from_str_radix(str.trim(), 16).ok();
    }
    None
}
