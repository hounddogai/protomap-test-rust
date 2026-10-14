use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional delegation milestone calculation.
pub trait RegionalDelegationMilestoneAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalDelegationMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRegionalDelegationMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalDelegationMilestoneAdjuster for NameRegionalDelegationMilestoneAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RegionalDelegationMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalDelegationMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalDelegationMilestoneAdjuster for TieredRegionalDelegationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalDelegationMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationMilestoneAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl RegionalDelegationMilestoneAdjuster for RegionalDelegationMilestoneAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalDelegationMilestone, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalDelegationMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalDelegationMilestoneAdjuster for FlaggedRegionalDelegationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalDelegationMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalDelegationMilestoneAdjuster {
    pub percent: f64,
}

impl RegionalDelegationMilestoneAdjuster for PercentRegionalDelegationMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalDelegationMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional delegation milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalDelegationMilestoneEngine {
    pub steps: Vec<Rc<dyn RegionalDelegationMilestoneAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalDelegationMilestoneEngine {
    pub fn price(&mut self, item: &RegionalDelegationMilestone) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[RegionalDelegationMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_delegation_milestone_engine(flags: &flags::Registry) -> RegionalDelegationMilestoneEngine {
    RegionalDelegationMilestoneEngine {
        steps: vec![
            Rc::new(NameRegionalDelegationMilestoneAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.33), ("central".to_string(), 1.1)]),
            }) as Rc<dyn RegionalDelegationMilestoneAdjuster>,
            Rc::new(TieredRegionalDelegationMilestoneAdjuster {
                bounds: vec![1077.0, 1981.0, 7183.0],
                rates: vec![0.94, 1.04, 0.92],
            }),
            Rc::new(RegionalDelegationMilestoneAttemptsOffsetAdjuster { weight: 0.28 }),
            Rc::new(FlaggedRegionalDelegationMilestoneAdjuster {
                flags: flags.clone(),
                flag: "regional-delegation-milestone-manual".to_string(),
                percent: 9.3,
            }),
            Rc::new(PercentRegionalDelegationMilestoneAdjuster { percent: 13.7 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 73977.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_delegation_milestone_engine(&flags::new_registry());
        let item = new_regional_delegation_milestone("code-634", "name-606");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
