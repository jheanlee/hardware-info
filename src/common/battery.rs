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

    /// Creates a new empty [`Batteries`][`crate::Batteries`] type.
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

    // /// Returns the mutable batteries list.
    // ///
    // /// ```no_run
    // /// use hardware_info::Batteries;
    // ///
    // /// if let Ok(mut batteries) = Batteries::new_with_refreshed_list() {
    // ///     for battery in batteries.list_mut() {
    // ///         battery.refresh();
    // ///     }
    // /// }
    // /// ```
    // pub fn list_mut(&mut self) -> &mut [Battery] {
    //     self.inner.list_mut()
    // }

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

// impl<'a> IntoIterator for &'a mut Batteries {
//     type Item = &'a mut Battery;
//     type IntoIter = std::slice::IterMut<'a, Battery>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         self.list_mut().iter_mut()
//     }
// }

impl std::ops::Deref for Batteries {
    type Target = [Battery];

    fn deref(&self) -> &Self::Target {
        self.list()
    }
}

// impl std::ops::DerefMut for Batteries {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         self.list_mut()
//     }
// }

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
    // /// Refreshes the battery's information.
    // ///
    // /// ```no_run
    // /// use hardware_info::Batteries;
    // ///
    // /// if let Ok(mut batteries) = Batteries::new_with_refreshed_list() {
    // ///     for battery in batteries.list_mut() {
    // ///         battery.refresh();
    // ///     }
    // /// }
    // /// ```
    // pub fn refresh(&mut self) {
    //     self.inner.refresh()
    // }

    /// Returns the name of the battery.
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    /// Returns the manufacturer of the battery.
    pub fn manufacturer(&self) -> Option<&str> {
        self.inner.manufacturer()
    }

    /// Returns the model name of the battery.
    pub fn model_name(&self) -> Option<&str> {
        self.inner.model_name()
    }

    /// Returns the output voltage, in µV, of the battery at this moment.
    pub fn voltage_now(&self) -> Option<u64> {
        self.inner.voltage_now()
    }

    /// Returns the output current, in µA, of the battery at this moment.
    ///
    /// On M series MacBooks, this value is only available when the battery is discharging.
    pub fn current_now(&self) -> Option<u64> {
        self.inner.current_now()
    }

    /// Returns the charge status of the battery.
    pub fn charge_status(&self) -> Option<BatteryChargeStatus> {
        self.inner.charge_status()
    }

    /// Returns the charge, in µAh, that the battery holds at this moment.
    pub fn charge_now(&self) -> Option<u64> {
        self.inner.charge_now()
    }

    /// Returns the charge, in µAh, that the battery holds when full.
    pub fn charge_full(&self) -> Option<u64> {
        self.inner.charge_full()
    }

    /// Returns the charge, in µAh, that the battery is designed to hold when full.
    pub fn charge_full_design(&self) -> Option<u64> {
        self.inner.charge_full_design()
    }

    /// Returns the charge threshold, in µAh, at which the battery starts charging.
    pub fn charge_control_start_threshold(&self) -> Option<u64> {
        self.inner.charge_control_start_threshold()
    }

    /// Returns the charge threshold, in µAh, at which the battery stops charging.
    pub fn charge_control_end_threshold(&self) -> Option<u64> {
        self.inner.charge_control_end_threshold()
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
