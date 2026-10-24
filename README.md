# hardware-info

A cross-platform library for fetching hardware information.

## Supported OSes

`hardware-info` supports the following OSes (sorted alphabetically):

- FreeBSD
- Linux
- macOS
- NetBSD
- Windows

## Example

Here is an example of listing all PCI devices and their addresses every 5 seconds:

```rust,no_run
let Ok(mut pci_devices) = hardware_info::PciDevices::new() else {
    return;
};

loop {
    pci_devices.refresh(true); //  Refresh PCI information and remove devices that are no longer present.
    let mut pci_devices = pci_devices.list().to_vec();
    pci_devices.sort();

    println!("PCI devices: ");
    for pci_device in pci_devices {
        println!("{}", pci_device.address())
    }

    std::thread::sleep(std::time::Duration::from_secs(5));
}
```

More examples can be found in the [`examples`](https://github.com/jheanlee/hardware-info/tree/master/examples)
directory.

## Related Crates

- [`sysinfo`](https://crates.io/crates/sysinfo) - A cross-platform library for fetching system information, such as
  system resources, processes and networks.

## License

This project is licensed under the [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0).

## Acknowledgement

This project's design is inspired by [`sysinfo`](https://github.com/GuillaumeGomez/sysinfo).