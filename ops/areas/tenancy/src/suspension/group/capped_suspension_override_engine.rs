use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped suspension override calculation.
pub trait CappedSuspensionOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedSuspensionOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedSuspensionOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedSuspensionOverrideAdjuster for TieredCappedSuspensionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedSuspensionOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedSuspensionOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedSuspensionOverrideAdjuster for FlaggedCappedSuspensionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedSuspensionOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedSuspensionOverrideAdjuster {
    pub percent: f64,
}

impl CappedSuspensionOverrideAdjuster for PercentCappedSuspensionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedSuspensionOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedSuspensionOverrideAdjuster {
    pub minimum: f64,
}

impl CappedSuspensionOverrideAdjuster for MinimumCappedSuspensionOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedSuspensionOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the capped suspension override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedSuspensionOverrideEngine {
    pub steps: Vec<Rc<dyn CappedSuspensionOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedSuspensionOverrideEngine {
    pub fn compute(&mut self, item: &CappedSuspensionOverride) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[CappedSuspensionOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_suspension_override_engine(flags: &flags::Registry) -> CappedSuspensionOverrideEngine {
    CappedSuspensionOverrideEngine {
        steps: vec![
            Rc::new(TieredCappedSuspensionOverrideAdjuster {
                bounds: vec![6171.0, 7674.0, 8014.0, 8412.0],
                rates: vec![0.9, 1.16, 1.2, 1.18],
            }) as Rc<dyn CappedSuspensionOverrideAdjuster>,
            Rc::new(FlaggedCappedSuspensionOverrideAdjuster {
                flags: flags.clone(),
                flag: "capped-suspension-override-manual".to_string(),
                percent: 9.6,
            }),
            Rc::new(PercentCappedSuspensionOverrideAdjuster { percent: 12.8 }),
            Rc::new(MinimumCappedSuspensionOverrideAdjuster { minimum: 29.4 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 45735.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_suspension_override_engine(&flags::new_registry());
        let item = new_capped_suspension_override("code-548", "name-545");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
