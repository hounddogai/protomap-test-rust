use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the legacy limit group calculation.
pub trait LegacyLimitGroupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyLimitGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitLegacyLimitGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyLimitGroupFactor for UnitLegacyLimitGroupAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &LegacyLimitGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyLimitGroupAdjuster {
    pub minimum: f64,
}

impl LegacyLimitGroupFactor for MinimumLegacyLimitGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyLimitGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyLimitGroupAdjuster {
    pub percent: f64,
}

impl LegacyLimitGroupFactor for PercentLegacyLimitGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyLimitGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyLimitGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyLimitGroupFactor for FlaggedLegacyLimitGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyLimitGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyLimitGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyLimitGroupFactor for TieredLegacyLimitGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyLimitGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the legacy limit group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyLimitGroupEngine {
    pub steps: Vec<Rc<dyn LegacyLimitGroupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyLimitGroupEngine {
    pub fn evaluate(&mut self, item: &LegacyLimitGroup) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyLimitGroupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyLimitGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_legacy_limit_group_engine(flags: &flags::Registry) -> LegacyLimitGroupEngine {
    LegacyLimitGroupEngine {
        steps: vec![
            Rc::new(UnitLegacyLimitGroupAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.95)]),
            }) as Rc<dyn LegacyLimitGroupFactor>,
            Rc::new(MinimumLegacyLimitGroupAdjuster { minimum: 9.0 }),
            Rc::new(PercentLegacyLimitGroupAdjuster { percent: 1.6 }),
            Rc::new(FlaggedLegacyLimitGroupAdjuster {
                flags: flags.clone(),
                flag: "legacy-limit-group-strict".to_string(),
                percent: -5.2,
            }),
            Rc::new(TieredLegacyLimitGroupAdjuster {
                bounds: vec![6894.0, 8667.0],
                rates: vec![0.89, 1.1],
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 23036.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_limit_group_engine(&flags::new_registry());
        let item = new_legacy_limit_group("code-550", "label-712");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
