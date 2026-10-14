use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the freeze mapping calculation.
pub trait FreezeMappingAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFreezeMappingAdjuster {
    pub percent: f64,
}

impl FreezeMappingAdjuster for PercentFreezeMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FreezeMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeMappingGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl FreezeMappingAdjuster for FreezeMappingGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &FreezeMapping, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFreezeMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FreezeMappingAdjuster for FlaggedFreezeMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FreezeMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezeMappingAdjuster {
    pub minimum: f64,
}

impl FreezeMappingAdjuster for MinimumFreezeMappingAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezeMapping, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFreezeMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FreezeMappingAdjuster for TitleFreezeMappingAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FreezeMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFreezeMappingAdjuster {
    pub cap: f64,
}

impl FreezeMappingAdjuster for CapFreezeMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FreezeMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the freeze mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeMappingEngine {
    pub steps: Vec<Rc<dyn FreezeMappingAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeMappingEngine {
    pub fn evaluate(&mut self, item: &FreezeMapping) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[FreezeMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_freeze_mapping_engine(flags: &flags::Registry) -> FreezeMappingEngine {
    FreezeMappingEngine {
        steps: vec![
            Rc::new(PercentFreezeMappingAdjuster { percent: -20.6 }) as Rc<dyn FreezeMappingAdjuster>,
            Rc::new(FreezeMappingGraceDaysOffsetAdjuster { weight: 1.25 }),
            Rc::new(FlaggedFreezeMappingAdjuster {
                flags: flags.clone(),
                flag: "freeze-mapping-pending".to_string(),
                percent: -7.7,
            }),
            Rc::new(MinimumFreezeMappingAdjuster { minimum: 13.6 }),
            Rc::new(TitleFreezeMappingAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.78)]),
            }),
            Rc::new(CapFreezeMappingAdjuster { cap: 23567.0 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 53873.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_mapping_engine(&flags::new_registry());
        let item = new_freeze_mapping("code-68", "title-847");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
