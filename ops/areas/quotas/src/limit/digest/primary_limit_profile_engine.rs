use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the primary limit profile calculation.
pub trait PrimaryLimitProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryLimitProfile, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryLimitProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryLimitProfileAdjuster for FlaggedPrimaryLimitProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryLimitProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryLimitProfileAdjuster {
    pub percent: f64,
}

impl PrimaryLimitProfileAdjuster for PercentPrimaryLimitProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryLimitProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterPrimaryLimitProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryLimitProfileAdjuster for ClusterPrimaryLimitProfileAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &PrimaryLimitProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the primary limit profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryLimitProfileEngine {
    pub steps: Vec<Rc<dyn PrimaryLimitProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryLimitProfileEngine {
    pub fn compute(&mut self, item: &PrimaryLimitProfile) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryLimitProfileAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryLimitProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_limit_profile_engine(flags: &flags::Registry) -> PrimaryLimitProfileEngine {
    PrimaryLimitProfileEngine {
        steps: vec![
            Rc::new(FlaggedPrimaryLimitProfileAdjuster {
                flags: flags.clone(),
                flag: "primary-limit-profile-global".to_string(),
                percent: -0.3,
            }) as Rc<dyn PrimaryLimitProfileAdjuster>,
            Rc::new(PercentPrimaryLimitProfileAdjuster { percent: 13.6 }),
            Rc::new(ClusterPrimaryLimitProfileAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.94), ("central".to_string(), 0.95)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 81479.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_limit_profile_engine(&flags::new_registry());
        let item = new_primary_limit_profile("code-385", "title-384");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
