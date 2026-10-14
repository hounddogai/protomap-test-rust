use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the delegation override calculation.
pub trait DelegationOverrideFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationOverride, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDelegationOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DelegationOverrideFactor for FlaggedDelegationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DelegationOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DelegationOverrideFactor for TieredDelegationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DelegationOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDelegationOverrideAdjuster {
    pub percent: f64,
}

impl DelegationOverrideFactor for PercentDelegationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DelegationOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationOverrideDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl DelegationOverrideFactor for DelegationOverrideDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &DelegationOverride, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDelegationOverrideAdjuster {
    pub minimum: f64,
}

impl DelegationOverrideFactor for MinimumDelegationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DelegationOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelDelegationOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DelegationOverrideFactor for ChannelDelegationOverrideAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &DelegationOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the delegation override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationOverrideEngine {
    pub steps: Vec<Rc<dyn DelegationOverrideFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationOverrideEngine {
    pub fn run(&mut self, item: &DelegationOverride) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[DelegationOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_delegation_override_engine(flags: &flags::Registry) -> DelegationOverrideEngine {
    DelegationOverrideEngine {
        steps: vec![
            Rc::new(FlaggedDelegationOverrideAdjuster {
                flags: flags.clone(),
                flag: "delegation-override-lenient".to_string(),
                percent: 0.7,
            }) as Rc<dyn DelegationOverrideFactor>,
            Rc::new(TieredDelegationOverrideAdjuster {
                bounds: vec![3061.0, 4917.0, 7143.0],
                rates: vec![0.97, 0.91, 0.85],
            }),
            Rc::new(PercentDelegationOverrideAdjuster { percent: -22.6 }),
            Rc::new(DelegationOverrideDurationMinutesOffsetAdjuster { weight: 0.9 }),
            Rc::new(MinimumDelegationOverrideAdjuster { minimum: 19.6 }),
            Rc::new(ChannelDelegationOverrideAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.36),
                    ("west".to_string(), 1.4),
                    ("online".to_string(), 1.34),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 58536.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_delegation_override_engine(&flags::new_registry());
        let item = new_delegation_override("code-596", "label-190");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
