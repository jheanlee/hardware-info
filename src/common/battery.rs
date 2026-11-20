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
use crate::Error;

#[derive(Clone)]
/// Interacting with batteries.
///
/// ⚠️Only supported on Linux, macOS, Windows, and FreeBSD.
///
/// ```no_run
/// use hardware_info::Batteries;
///
/// if let Ok(batteries) = Batteries::new_with_refreshed_list() {
///     for battery in batteries.list() {
///         println!("{}", battery.name());
///     }
/// }
/// ```
pub struct Batteries {
    pub(crate) inner: crate::BatteriesInner,
}

impl Batteries {
    /// Creates a new empty [`Batteries`][`crate::Batteries`] type.
    ///
    /// If you want it to be filled directly, take a look at [`Batteries::new_with_refreshed_list`].
    ///
    /// ```no_run
    /// use hardware_info::Batteries;
    ///
    /// if let Ok(mut batteries) = Batteries::new() {
    ///     batteries.refresh(false);
    ///     for battery in batteries.list() {
    ///         println!("{}", battery.name());
    ///     }
    /// }
    /// ```
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            inner: crate::BatteriesInner::new()?,
        })
    }

    /// Creates a new [`Batteries`][`crate::Batteries`] type with the battery list loaded.
    ///
    /// ```no_run
    /// use hardware_info::Batteries;
    ///
    /// if let Ok(batteries) = Batteries::new_with_refreshed_list() {
    ///     for battery in batteries.list() {
    ///         println!("{}", battery.name());
    ///     }
    /// }
    /// ```
    pub fn new_with_refreshed_list() -> Result<Self, Error> {
        let mut batteries = Self::new()?;
        batteries.refresh(false);
        Ok(batteries)
    }

    /// Returns the batteries list.
    ///
    /// ```no_run
    /// use hardware_info::Batteries;
    ///
    /// if let Ok(batteries) = Batteries::new_with_refreshed_list() {
    ///     for battery in batteries.list() {
    ///         println!("{}", battery.name());
    ///     }
    /// }
    /// ```
    pub fn list(&self) -> &[Battery] {
        self.inner.list()
    }

    /// Refreshes the batteries' information.
    ///
    /// ```no_run
    /// use hardware_info::Batteries;
    ///
    /// if let Ok(mut batteries) = Batteries::new() {
    ///     batteries.refresh(false);
    ///     for battery in batteries.list() {
    ///         println!("{}", battery.name());
    ///     }
    /// }
    /// ```
    pub fn refresh(&mut self, remove_not_listed_interfaces: bool) {
        self.inner.refresh(remove_not_listed_interfaces)
    }
}

impl From<Batteries> for Vec<Battery> {
    fn from(batteries: Batteries) -> Self {
        batteries.inner.into_vec()
    }
}

impl<'a> IntoIterator for &'a Batteries {
    type Item = &'a Battery;
    type IntoIter = std::slice::Iter<'a, Battery>;

    fn into_iter(self) -> Self::IntoIter {
        self.list().iter()
    }
}

impl std::ops::Deref for Batteries {
    type Target = [Battery];

    fn deref(&self) -> &Self::Target {
        self.list()
    }
}

#[derive(Clone)]
/// Type containing a battery's information.
///
/// ```no_run
/// use hardware_info::Batteries;
///
/// if let Ok(batteries) = Batteries::new_with_refreshed_list() {
///     for battery in batteries.list() {
///         println!("{}", battery.name());
///     }
/// }
/// ```
pub struct Battery {
    pub(crate) inner: crate::BatteryInner,
}

impl Battery {
    /// Returns the name of the battery.
    ///
    /// ```no_run
    /// use hardware_info::Batteries;
    ///
    /// if let Ok(batteries) = Batteries::new_with_refreshed_list() {
    ///     for battery in batteries.list() {
    ///         println!("{}", battery.name());
    ///     }
    /// }
    /// ```
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    /// Returns the manufacturer of the battery.
    ///
    /// ⚠️ Only implemented on Linux and Windows.
    ///
    /// ```no_run
    /// use hardware_info::Batteries;
    ///
    /// if let Ok(batteries) = Batteries::new_with_refreshed_list() {
    ///     for battery in batteries.list() {
    ///         if let Some(manufacturer) = battery.manufacturer() {
    ///             println!("Manufacturer: {manufacturer}");
    ///         }
    ///     }
    /// }
    /// ```
    pub fn manufacturer(&self) -> Option<&str> {
        self.inner.manufacturer()
    }

    /// Returns the model name of the battery.
    ///
    /// ```no_run
    /// use hardware_info::Batteries;
    ///
    /// if let Ok(batteries) = Batteries::new_with_refreshed_list() {
    ///     for battery in batteries.list() {
    ///         if let Some(model) = battery.model() {
    ///             println!("Model: {model}");
    ///         }
    ///     }
    /// }
    /// ```
    pub fn model(&self) -> Option<&str> {
        self.inner.model()
    }

    /// Returns the charge status of the battery.
    ///
    /// ```no_run
    /// use hardware_info::Batteries;
    ///
    /// if let Ok(batteries) = Batteries::new_with_refreshed_list() {
    ///     for battery in batteries.list() {
    ///         if let Some(status) = battery.charge_status() {
    ///             println!("Charge status: {status}");
    ///         }
    ///     }
    /// }
    /// ```
    pub fn charge_status(&self) -> Option<BatteryChargeStatus> {
        self.inner.charge_status()
    }

    /// Returns the output voltage, in µV, of the battery at this moment.
    ///
    /// ```no_run
    /// use hardware_info::Batteries;
    ///
    /// if let Ok(batteries) = Batteries::new_with_refreshed_list() {
    ///     for battery in batteries.list() {
    ///         println!("Voltage: {:?} µV", battery.voltage_now());
    ///     }
    /// }
    /// ```
    pub fn voltage_now(&self) -> Option<u64> {
        self.inner.voltage_now()
    }

    /// Returns the charge/discharge current, in µA, of the battery at this moment.
    ///
    /// On Linux, this value is always the discharge current.
    ///
    /// ```no_run
    /// use hardware_info::Batteries;
    ///
    /// if let Ok(batteries) = Batteries::new_with_refreshed_list() {
    ///     for battery in batteries.list() {
    ///         println!("Current: {:?} µA", battery.current_now());
    ///     }
    /// }
    /// ```
    pub fn current_now(&self) -> Option<u64> {
        self.inner.current_now()
    }

    /// Returns the charge, in µAh, that the battery holds at this moment.
    ///
    /// ```no_run
    /// use hardware_info::Batteries;
    ///
    /// if let Ok(batteries) = Batteries::new_with_refreshed_list() {
    ///     for battery in batteries.list() {
    ///         println!("Charge: {:?} µAh", battery.charge_now());
    ///     }
    /// }
    /// ```
    pub fn charge_now(&self) -> Option<u64> {
        self.inner.charge_now()
    }

    /// Returns the charge, in µAh, that the battery holds when full.
    ///
    /// ```no_run
    /// use hardware_info::Batteries;
    ///
    /// if let Ok(batteries) = Batteries::new_with_refreshed_list() {
    ///     for battery in batteries.list() {
    ///         println!("Full charge: {:?} µAh", battery.charge_full());
    ///     }
    /// }
    /// ```
    pub fn charge_full(&self) -> Option<u64> {
        self.inner.charge_full()
    }

    /// Returns the charge, in µAh, that the battery is designed to hold when full.
    ///
    /// ```no_run
    /// use hardware_info::Batteries;
    ///
    /// if let Ok(batteries) = Batteries::new_with_refreshed_list() {
    ///     for battery in batteries.list() {
    ///         println!("Designed full charge: {:?} µAh", battery.charge_full_design());
    ///     }
    /// }
    /// ```
    pub fn charge_full_design(&self) -> Option<u64> {
        self.inner.charge_full_design()
    }

    /// Returns the charge threshold, in µAh, at which the battery starts charging.
    ///
    /// ⚠️Only implemented on Linux.
    ///
    /// ```no_run
    /// use hardware_info::Batteries;
    ///
    /// if let Ok(batteries) = Batteries::new_with_refreshed_list() {
    ///     for battery in batteries.list() {
    ///         println!(
    ///             "Charge start threshold: {:?} µAh",
    ///             battery.charge_control_start_threshold()
    ///         );
    ///     }
    /// }
    /// ```
    pub fn charge_control_start_threshold(&self) -> Option<u64> {
        self.inner.charge_control_start_threshold()
    }

    /// Returns the charge threshold, in µAh, at which the battery stops charging.
    ///
    /// ⚠️Only implemented on Linux.
    ///
    /// ```no_run
    /// use hardware_info::Batteries;
    ///
    /// if let Ok(batteries) = Batteries::new_with_refreshed_list() {
    ///     for battery in batteries.list() {
    ///         println!(
    ///             "Charge end threshold: {:?} µAh",
    ///             battery.charge_control_end_threshold()
    ///         );
    ///     }
    /// }
    /// ```
    pub fn charge_control_end_threshold(&self) -> Option<u64> {
        self.inner.charge_control_end_threshold()
    }

    /// Returns the cycle count.
    ///
    /// ```no_run
    /// use hardware_info::Batteries;
    ///
    /// if let Ok(batteries) = Batteries::new_with_refreshed_list() {
    ///     for battery in batteries.list() {
    ///         println!("Cycle count: {:?}", battery.cycle_count());
    ///     }
    /// }
    /// ```
    pub fn cycle_count(&self) -> Option<u32> {
        self.inner.cycle_count()
    }

    /// Returns the battery's chemistry.
    ///
    /// ⚠️ Only implemented on Linux and Windows.
    ///
    /// ```no_run
    /// use hardware_info::Batteries;
    ///
    /// if let Ok(batteries) = Batteries::new_with_refreshed_list() {
    ///     for battery in batteries.list() {
    ///         println!("Chemistry: {:?}", battery.chemistry());
    ///     }
    /// }
    /// ```
    pub fn chemistry(&self) -> Option<BatteryChemistry> {
        self.inner.chemistry()
    }
}

/// Battery status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatteryChargeStatus {
    /// The battery is charging.
    Charging,
    /// The battery is discharging.
    Discharging,
    /// The battery is not discharging. This can be returned when the battery stopped charging after
    /// reaching a threshold.
    NotCharging,
    /// The battery is full.
    Full,
}

impl std::fmt::Display for BatteryChargeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatteryChargeStatus::Charging => write!(f, "Charging"),
            BatteryChargeStatus::Discharging => write!(f, "Discharging"),
            BatteryChargeStatus::NotCharging => write!(f, "Not charging"),
            BatteryChargeStatus::Full => write!(f, "Full"),
        }
    }
}

/// Battery chemistry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BatteryChemistry {
    /// Nickel–metal hydride battery.
    NiMH,
    /// Lithium-ion battery.
    LiIon,
    /// Lithium polymer battery.
    LiPoly,
    /// Lithium iron phosphate battery.
    LiFe,
    /// Nickel–cadmium battery.
    NiCd,
    /// Lithium-ion manganese oxide battery.
    LiMn,
    /// Lead-acid battery.
    PbAcid,
    /// Nickel–zinc battery.
    NiZn,
    /// Rechargeable alkaline battery.
    Ram,
    /// Zinc–air battery
    ZnAir,
    /// An unknown battery type.
    Unknown(String),
}

impl std::fmt::Display for BatteryChemistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatteryChemistry::NiMH => write!(f, "Nickel–metal hydride"),
            BatteryChemistry::LiIon => write!(f, "Lithium-ion"),
            BatteryChemistry::LiPoly => write!(f, "Lithium polymer"),
            BatteryChemistry::LiFe => write!(f, "Lithium iron phosphate"),
            BatteryChemistry::NiCd => write!(f, "Nickel–cadmium"),
            BatteryChemistry::LiMn => write!(f, "Lithium-ion manganese oxide"),
            BatteryChemistry::PbAcid => write!(f, "Lead-acid"),
            BatteryChemistry::NiZn => write!(f, "Nickel–zinc"),
            BatteryChemistry::Ram => write!(f, "Rechargeable alkaline"),
            BatteryChemistry::ZnAir => write!(f, "Zinc–air"),
            BatteryChemistry::Unknown(name) => write!(f, "{name}"),
        }
    }
}
