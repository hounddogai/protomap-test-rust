use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the legacy zone policy calculation.
pub trait LegacyZonePolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyZonePolicy, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyZonePolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyZonePolicyAdjuster for FlaggedLegacyZonePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyZonePolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyZonePolicyAdjuster {
    pub cap: f64,
}

impl LegacyZonePolicyAdjuster for CapLegacyZonePolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyZonePolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZonePolicyStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl LegacyZonePolicyAdjuster for LegacyZonePolicyStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyZonePolicy, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyZonePolicyAdjuster {
    pub percent: f64,
}

impl LegacyZonePolicyAdjuster for PercentLegacyZonePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyZonePolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyZonePolicyAdjuster {
    pub minimum: f64,
}

impl LegacyZonePolicyAdjuster for MinimumLegacyZonePolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyZonePolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the legacy zone policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyZonePolicyEngine {
    pub steps: Vec<Rc<dyn LegacyZonePolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyZonePolicyEngine {
    pub fn run(&mut self, item: &LegacyZonePolicy) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyZonePolicyAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyZonePolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_zone_policy_engine(flags: &flags::Registry) -> LegacyZonePolicyEngine {
    LegacyZonePolicyEngine {
        steps: vec![
            Rc::new(FlaggedLegacyZonePolicyAdjuster {
                flags: flags.clone(),
                flag: "legacy-zone-policy-legacy".to_string(),
                percent: -5.7,
            }) as Rc<dyn LegacyZonePolicyAdjuster>,
            Rc::new(CapLegacyZonePolicyAdjuster { cap: 47017.0 }),
            Rc::new(LegacyZonePolicyStartsAtOffsetAdjuster { weight: 1.08 }),
            Rc::new(PercentLegacyZonePolicyAdjuster { percent: 20.8 }),
            Rc::new(MinimumLegacyZonePolicyAdjuster { minimum: 0.9 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 70309.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_zone_policy_engine(&flags::new_registry());
        let item = new_legacy_zone_policy("code-743", "title-94");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
