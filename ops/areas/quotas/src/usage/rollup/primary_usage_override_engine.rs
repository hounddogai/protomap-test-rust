use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary usage override calculation.
pub trait PrimaryUsageOverrideStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryUsageOverride, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryUsageOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryUsageOverrideStep for FlaggedPrimaryUsageOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryUsageOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryUsageOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryUsageOverrideStep for TieredPrimaryUsageOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryUsageOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryUsageOverrideAdjuster {
    pub percent: f64,
}

impl PrimaryUsageOverrideStep for PercentPrimaryUsageOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryUsageOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary usage override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryUsageOverrideEngine {
    pub steps: Vec<Rc<dyn PrimaryUsageOverrideStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryUsageOverrideEngine {
    pub fn run(&mut self, item: &PrimaryUsageOverride) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[PrimaryUsageOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_usage_override_engine(flags: &flags::Registry) -> PrimaryUsageOverrideEngine {
    PrimaryUsageOverrideEngine {
        steps: vec![
            Rc::new(FlaggedPrimaryUsageOverrideAdjuster {
                flags: flags.clone(),
                flag: "primary-usage-override-tiered".to_string(),
                percent: -5.7,
            }) as Rc<dyn PrimaryUsageOverrideStep>,
            Rc::new(TieredPrimaryUsageOverrideAdjuster {
                bounds: vec![7482.0, 8467.0],
                rates: vec![1.11, 0.95],
            }),
            Rc::new(PercentPrimaryUsageOverrideAdjuster { percent: 24.5 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 68539.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_usage_override_engine(&flags::new_registry());
        let item = new_primary_usage_override("code-797", "title-201");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
