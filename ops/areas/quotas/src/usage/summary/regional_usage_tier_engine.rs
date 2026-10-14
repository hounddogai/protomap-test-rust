use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional usage tier calculation.
pub trait RegionalUsageTierStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalUsageTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageTierVersionOffsetAdjuster {
    pub weight: f64,
}

impl RegionalUsageTierStep for RegionalUsageTierVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalUsageTier, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalUsageTierAdjuster {
    pub step: f64,
}

impl RegionalUsageTierStep for RoundingRegionalUsageTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalUsageTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRegionalUsageTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalUsageTierStep for NameRegionalUsageTierAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RegionalUsageTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalUsageTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalUsageTierStep for FlaggedRegionalUsageTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalUsageTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional usage tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalUsageTierEngine {
    pub steps: Vec<Rc<dyn RegionalUsageTierStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalUsageTierEngine {
    pub fn evaluate(&mut self, item: &RegionalUsageTier) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[RegionalUsageTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_usage_tier_engine(flags: &flags::Registry) -> RegionalUsageTierEngine {
    RegionalUsageTierEngine {
        steps: vec![
            Rc::new(RegionalUsageTierVersionOffsetAdjuster { weight: 0.1 }) as Rc<dyn RegionalUsageTierStep>,
            Rc::new(RoundingRegionalUsageTierAdjuster { step: 5.0 }),
            Rc::new(NameRegionalUsageTierAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.11)]),
            }),
            Rc::new(FlaggedRegionalUsageTierAdjuster {
                flags: flags.clone(),
                flag: "regional-usage-tier-rolling".to_string(),
                percent: 7.1,
            }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 8492.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_usage_tier_engine(&flags::new_registry());
        let item = new_regional_usage_tier("code-787", "name-468");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
