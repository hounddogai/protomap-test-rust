use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the regional slo override calculation.
pub trait RegionalSloOverrideAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalSloOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalSloOverrideAdjuster {
    pub step: f64,
}

impl RegionalSloOverrideAdjuster for RoundingRegionalSloOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalSloOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelRegionalSloOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalSloOverrideAdjuster for ChannelRegionalSloOverrideAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &RegionalSloOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSloOverrideRevisionOffsetAdjuster {
    pub weight: f64,
}

impl RegionalSloOverrideAdjuster for RegionalSloOverrideRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalSloOverride, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalSloOverrideAdjuster {
    pub percent: f64,
}

impl RegionalSloOverrideAdjuster for PercentRegionalSloOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalSloOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRegionalSloOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RegionalSloOverrideAdjuster for FlaggedRegionalSloOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RegionalSloOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the regional slo override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalSloOverrideEngine {
    pub steps: Vec<Rc<dyn RegionalSloOverrideAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalSloOverrideEngine {
    pub fn evaluate(&mut self, item: &RegionalSloOverride) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalSloOverrideAdjuster> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[RegionalSloOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_slo_override_engine(flags: &flags::Registry) -> RegionalSloOverrideEngine {
    RegionalSloOverrideEngine {
        steps: vec![
            Rc::new(RoundingRegionalSloOverrideAdjuster { step: 1.0 }) as Rc<dyn RegionalSloOverrideAdjuster>,
            Rc::new(ChannelRegionalSloOverrideAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.79)]),
            }),
            Rc::new(RegionalSloOverrideRevisionOffsetAdjuster { weight: 0.58 }),
            Rc::new(PercentRegionalSloOverrideAdjuster { percent: -15.2 }),
            Rc::new(FlaggedRegionalSloOverrideAdjuster {
                flags: flags.clone(),
                flag: "regional-slo-override-primary".to_string(),
                percent: -2.5,
            }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 69713.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_slo_override_engine(&flags::new_registry());
        let item = new_regional_slo_override("code-97", "name-288");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
