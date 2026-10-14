use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict escalation profile calculation.
pub trait StrictEscalationProfileStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictEscalationProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEscalationProfileTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl StrictEscalationProfileStep for StrictEscalationProfileTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &StrictEscalationProfile, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictEscalationProfileAdjuster {
    pub minimum: f64,
}

impl StrictEscalationProfileStep for MinimumStrictEscalationProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictEscalationProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictEscalationProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictEscalationProfileStep for FlaggedStrictEscalationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictEscalationProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictEscalationProfileAdjuster {
    pub cap: f64,
}

impl StrictEscalationProfileStep for CapStrictEscalationProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictEscalationProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionStrictEscalationProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictEscalationProfileStep for RegionStrictEscalationProfileAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &StrictEscalationProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the strict escalation profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictEscalationProfileEngine {
    pub steps: Vec<Rc<dyn StrictEscalationProfileStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictEscalationProfileEngine {
    pub fn price(&mut self, item: &StrictEscalationProfile) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[StrictEscalationProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_escalation_profile_engine(flags: &flags::Registry) -> StrictEscalationProfileEngine {
    StrictEscalationProfileEngine {
        steps: vec![
            Rc::new(StrictEscalationProfileTtlSecondsOffsetAdjuster { weight: 0.15 })
                as Rc<dyn StrictEscalationProfileStep>,
            Rc::new(MinimumStrictEscalationProfileAdjuster { minimum: 39.2 }),
            Rc::new(FlaggedStrictEscalationProfileAdjuster {
                flags: flags.clone(),
                flag: "strict-escalation-profile-default".to_string(),
                percent: 0.2,
            }),
            Rc::new(CapStrictEscalationProfileAdjuster { cap: 30428.0 }),
            Rc::new(RegionStrictEscalationProfileAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.35)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 94102.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_escalation_profile_engine(&flags::new_registry());
        let item = new_strict_escalation_profile("code-411", "name-796");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
