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

#[cfg(feature = "pci")]
pub(crate) const PCI_IOC_BUSINFO: libc::c_ulong =
    libc::_IOR::<pciio_businfo>(b'P' as libc::c_ulong, 4);

#[cfg(feature = "pci")]
pub(crate) const PCI_BHLC_REG: libc::c_uint = 0x0c;
#[cfg(feature = "pci")]
pub(crate) const PCI_ID_REG: libc::c_uint = 0x00;
#[cfg(feature = "pci")]
pub(crate) const PCI_SUBSYS_ID_REG: libc::c_uint = 0x2c;
#[cfg(feature = "pci")]
pub(crate) const PCI_CLASS_REG: libc::c_uint = 0x08;

#[cfg(feature = "pci")]
pub(crate) const PCI_HDRTYPE_SHIFT: u8 = 16;
#[cfg(feature = "pci")]
pub(crate) const PCI_HDRTYPE_MASK: u32 = 0xff;

#[cfg(feature = "pci")]
pub(crate) const PCI_IO_DRVNAME_LEN: usize = 16;

#[cfg(feature = "pci")]
#[link(name = "pci")]
unsafe extern "C" {
    pub(crate) fn pcibus_conf_read(
        fd: libc::c_int,
        bus: libc::c_uint,
        device: libc::c_uint,
        func: libc::c_uint,
        reg: libc::c_uint,
        valp: *mut u32,
    ) -> libc::c_int;

    pub(crate) fn pci_drvname(
        fd: libc::c_int,
        device: libc::c_uint,
        func: libc::c_uint,
        name: *mut libc::c_char,
        len: libc::size_t,
    ) -> libc::c_int;
}

#[cfg(feature = "pci")]
#[repr(C)]
pub(crate) struct pciio_businfo {
    pub busno: libc::c_uint,
    pub maxdev: libc::c_uint,
}
