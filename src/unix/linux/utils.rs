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

//  Adapted from `sysinfo`:
//  https://github.com/GuillaumeGomez/sysinfo
//  Original code licensed under the MIT License. See `THIRD-PARTY-LICENSES` at the project root.
#[cfg(feature = "pci")]
pub(crate) fn read_unsigned<P: AsRef<std::path::Path>>(
    parent: P,
    path: &str,
    data: &mut [u8],
) -> Option<u64> {
    if let Ok(mut f) = std::fs::File::open(parent.as_ref().join(path))
        && let Ok(size) = f.read(data)
    {
        let mut i = 0;
        let mut ret = 0;

        while i < size && i < data.len() && data[i] >= b'0' && data[i] <= b'9' {
            ret *= 10;
            ret += (data[i] - b'0') as u64;
            i += 1;
        }
        return Some(ret);
    }
    None
}

//  Adapted from `sysinfo`:
//  https://github.com/GuillaumeGomez/sysinfo
//  Original code licensed under the MIT License. See `THIRD-PARTY-LICENSES` at the project root.
#[cfg(feature = "pci")]
#[allow(clippy::ptr_arg)]
pub(crate) fn read_str<'data, P: AsRef<std::path::Path>>(
    parent: P,
    path: &str,
    data: &'data mut Vec<u8>,
) -> &'data [u8] {
    data.clear();
    if let Ok(mut f) = std::fs::File::open(parent.as_ref().join(path))
        && let Ok(size) = f.read_to_end(data)
    {
        &mut data[..size]
    } else {
        b""
    }
}
