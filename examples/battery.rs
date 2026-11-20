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

//! An example of listing all batteries and their information.

use hardware_info::Batteries;

fn format_optional<T: std::fmt::Display>(value: Option<T>) -> String {
    value.map_or_else(|| "(not available)".to_string(), |value| value.to_string())
}

fn main() {
    let Ok(batteries) = Batteries::new_with_refreshed_list() else {
        println!("Battery is not supported on this OS.");
        return;
    };

    for battery in batteries.list() {
        println!("========== {} ==========", battery.name());

        println!(
            "Manufacturer: {}",
            battery.manufacturer().unwrap_or("(not available)")
        );
        println!("Model: {}", battery.model().unwrap_or("(not available)"));
        println!("Chemistry: {}", format_optional(battery.chemistry()));

        println!(
            "Charge status: {}",
            format_optional(battery.charge_status())
        );

        println!("Voltage: {} µV", format_optional(battery.voltage_now()));
        println!("Current: {} µA", format_optional(battery.current_now()));

        let charge_now = battery.charge_now();
        let charge_full = battery.charge_full();
        let charge_full_design = battery.charge_full_design();

        println!(
            "Charge: {} / {} µAh",
            format_optional(charge_now),
            format_optional(charge_full)
        );

        let battery_health = charge_full
            .zip(charge_full_design)
            .filter(|(_, design)| *design > 0)
            .map(|(full, design)| full as f64 / design as f64 * 100.0);

        println!(
            "Designed full charge capacity: {} µAh (Health: {})",
            format_optional(charge_full_design),
            battery_health.map_or_else(
                || "(not available)".to_string(),
                |health| format!("{health:.2}%")
            )
        );

        if let (Some(charge_now), Some(charge_full)) = (charge_now, charge_full)
            && charge_full > 0
        {
            println!(
                "Charge level: {:.2}%",
                charge_now as f64 / charge_full as f64 * 100.0
            );
        } else {
            println!("Charge level: (not available)");
        }

        println!(
            "Charge start threshold: {} µAh",
            format_optional(battery.charge_control_start_threshold())
        );
        println!(
            "Charge end threshold: {} µAh",
            format_optional(battery.charge_control_end_threshold())
        );

        println!("Cycle count: {}", format_optional(battery.cycle_count()));
    }
}
