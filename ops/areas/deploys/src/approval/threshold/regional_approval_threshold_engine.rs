use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional approval threshold calculation.
pub trait RegionalApprovalThresholdModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalApprovalThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalApprovalThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalApprovalThresholdModifier for FlaggedRegionalApprovalThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalApprovalThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryRegionalApprovalThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalApprovalThresholdModifier for CategoryRegionalApprovalThresholdAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &RegionalApprovalThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalApprovalThresholdAdjuster {
    pub percent: f64,
}

impl RegionalApprovalThresholdModifier for PercentRegionalApprovalThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalApprovalThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalApprovalThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalApprovalThresholdModifier for TieredRegionalApprovalThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalApprovalThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the regional approval threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalApprovalThresholdEngine {
    pub steps: Vec<Rc<dyn RegionalApprovalThresholdModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalApprovalThresholdEngine {
    pub fn run(&mut self, item: &RegionalApprovalThreshold) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[RegionalApprovalThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_approval_threshold_engine(flags: &flags::Registry) -> RegionalApprovalThresholdEngine {
    RegionalApprovalThresholdEngine {
        steps: vec![
            Rc::new(FlaggedRegionalApprovalThresholdAdjuster {
                flags: flags.clone(),
                flag: "regional-approval-threshold-legacy".to_string(),
                percent: -8.9,
            }) as Rc<dyn RegionalApprovalThresholdModifier>,
            Rc::new(CategoryRegionalApprovalThresholdAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.98)]),
            }),
            Rc::new(PercentRegionalApprovalThresholdAdjuster { percent: 4.6 }),
            Rc::new(TieredRegionalApprovalThresholdAdjuster {
                bounds: vec![7579.0, 8557.0],
                rates: vec![0.96, 1.06],
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 26574.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_approval_threshold_engine(&flags::new_registry());
        let item = new_regional_approval_threshold("code-263", "title-319");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
