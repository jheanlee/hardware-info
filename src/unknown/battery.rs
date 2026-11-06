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

use crate::{Battery, BatteryStatus, Error};

#[derive(Clone)]
pub(crate) struct BatteriesInner;

impl BatteriesInner {
    pub(crate) fn new() -> Result<Self, Error> {
        Err(Error::Unsupported)
    }

    pub(crate) fn list(&self) -> &[Battery] {
        &[]
    }

    pub(crate) fn list_mut(&mut self) -> &mut [Battery] {
        &mut []
    }

    pub(crate) fn into_vec(self) -> Vec<Battery> {
        Vec::new()
    }

    pub(crate) fn refresh(&mut self, _: bool) {}
}

#[derive(Clone)]
pub(crate) struct BatteryInner {}

impl BatteryInner {
    pub(crate) fn refresh(&mut self) {}

    pub(crate) fn name(&self) -> &str {
        ""
    }

    pub(crate) fn manufacturer(&self) -> Option<&str> {
        None
    }

    pub(crate) fn model_name(&self) -> Option<&str> {
        None
    }

    pub(crate) fn status(&self) -> Option<BatteryStatus> {
        None
    }

    pub(crate) fn charge_now(&self) -> Option<u64> {
        None
    }

    pub(crate) fn charge_full(&self) -> Option<u64> {
        None
    }

    pub(crate) fn charge_full_design(&self) -> Option<u64> {
        None
    }

    pub(crate) fn charge_control_start_threshold(&self) -> Option<u64> {
        None
    }

    pub(crate) fn charge_control_end_threshold(&self) -> Option<u64> {
        None
    }

    pub(crate) fn voltage_now(&self) -> Option<u64> {
        None
    }

    pub(crate) fn current_now(&self) -> Option<u64> {
        None
    }
}
