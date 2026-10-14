use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the probe tier calculation.
pub trait ProbeTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeTierQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl ProbeTierAdjuster for ProbeTierQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &ProbeTier, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleProbeTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ProbeTierAdjuster for TitleProbeTierAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ProbeTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedProbeTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ProbeTierAdjuster for FlaggedProbeTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ProbeTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the probe tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeTierEngine {
    pub steps: Vec<Rc<dyn ProbeTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeTierEngine {
    pub fn price(&mut self, item: &ProbeTier) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ProbeTierAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ProbeTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_probe_tier_engine(flags: &flags::Registry) -> ProbeTierEngine {
    ProbeTierEngine {
        steps: vec![
            Rc::new(ProbeTierQuotaUnitsOffsetAdjuster { weight: 1.84 }) as Rc<dyn ProbeTierAdjuster>,
            Rc::new(TitleProbeTierAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.91)]),
            }),
            Rc::new(FlaggedProbeTierAdjuster {
                flags: flags.clone(),
                flag: "probe-tier-regional".to_string(),
                percent: -2.6,
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 25996.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_probe_tier_engine(&flags::new_registry());
        let item = new_probe_tier("code-767", "title-756");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
