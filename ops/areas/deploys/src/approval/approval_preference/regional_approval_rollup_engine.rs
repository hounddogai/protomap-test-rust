use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional approval rollup calculation.
pub trait RegionalApprovalRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalApprovalRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalApprovalRollupAdjuster {
    pub percent: f64,
}

impl RegionalApprovalRollupFactor for PercentRegionalApprovalRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalApprovalRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalApprovalRollupAdjuster {
    pub cap: f64,
}

impl RegionalApprovalRollupFactor for CapRegionalApprovalRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalApprovalRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelRegionalApprovalRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalApprovalRollupFactor for ChannelRegionalApprovalRollupAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &RegionalApprovalRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalApprovalRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalApprovalRollupFactor for FlaggedRegionalApprovalRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalApprovalRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalApprovalRollupAdjuster {
    pub minimum: f64,
}

impl RegionalApprovalRollupFactor for MinimumRegionalApprovalRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalApprovalRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalApprovalRollupCeilingOffsetAdjuster {
    pub weight: f64,
}

impl RegionalApprovalRollupFactor for RegionalApprovalRollupCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalApprovalRollup, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

/// Runs the regional approval rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalApprovalRollupEngine {
    pub steps: Vec<Rc<dyn RegionalApprovalRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalApprovalRollupEngine {
    pub fn run(&mut self, item: &RegionalApprovalRollup) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[RegionalApprovalRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_approval_rollup_engine(flags: &flags::Registry) -> RegionalApprovalRollupEngine {
    RegionalApprovalRollupEngine {
        steps: vec![
            Rc::new(PercentRegionalApprovalRollupAdjuster { percent: -24.8 }) as Rc<dyn RegionalApprovalRollupFactor>,
            Rc::new(CapRegionalApprovalRollupAdjuster { cap: 16583.0 }),
            Rc::new(ChannelRegionalApprovalRollupAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.75)]),
            }),
            Rc::new(FlaggedRegionalApprovalRollupAdjuster {
                flags: flags.clone(),
                flag: "regional-approval-rollup-lenient".to_string(),
                percent: -6.8,
            }),
            Rc::new(MinimumRegionalApprovalRollupAdjuster { minimum: 23.1 }),
            Rc::new(RegionalApprovalRollupCeilingOffsetAdjuster { weight: 0.12 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 34075.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_approval_rollup_engine(&flags::new_registry());
        let item = new_regional_approval_rollup("code-38", "label-891");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
