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

#![allow(non_camel_case_types)]

//  https://github.com/freebsd/freebsd-src/blob/4a12e28f330c2fd002f31c6a70f88994fe902f8f/sys/sys/pciio.h
//  Note: FreeBSD 15.0.0 changed the definition of struct `pci_conf` and command number of `PCIOCGETCONF`.
//  Therefore, the definition on the `main` branch is new and does not work on earlier versions.
//  Backwards compatibility is preserved according to docs. (Tested on 15.1)
#[cfg(feature = "pci")]
const PCI_MAXNAMELEN: usize = 16;

#[repr(C)]
#[cfg(feature = "pci")]
pub(crate) struct pci_conf_io {
    pub pat_buf_len: u32,
    pub num_patterns: u32,
    pub patterns: *mut libc::c_void, //  struct pci_match_conf; not used
    pub match_buf_len: u32,
    pub num_matches: u32,
    pub matches: *mut pci_conf,
    pub offset: u32,
    pub generation: u32,
    pub status: libc::c_int,
}

#[repr(C)]
#[cfg(feature = "pci")]
pub(crate) struct pci_conf {
    pub(crate) pc_sel: pcisel,
    pub(crate) pc_hdr: u8,
    pub(crate) pc_subvendor: u16,
    pub(crate) pc_subdevice: u16,
    pub(crate) pc_vendor: u16,
    pub(crate) pc_device: u16,
    pub(crate) pc_class: u8,
    pub(crate) pc_subclass: u8,
    pub(crate) pc_progif: u8,
    pub(crate) pc_revid: u8,
    pub(crate) pd_name: [libc::c_char; PCI_MAXNAMELEN + 1],
    pub(crate) pd_unit: libc::c_ulong,
}

#[repr(C)]
#[cfg(feature = "pci")]
pub struct pcisel {
    pub(crate) pc_domain: u32,
    pub(crate) pc_bus: u8,
    pub(crate) pc_dev: u8,
    pub(crate) pc_func: u8,
}
