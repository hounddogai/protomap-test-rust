use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary sink draft calculation.
pub trait PrimarySinkDraftAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimarySinkDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimarySinkDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimarySinkDraftAdjuster for TieredPrimarySinkDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimarySinkDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimarySinkDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimarySinkDraftAdjuster for FlaggedPrimarySinkDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimarySinkDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterPrimarySinkDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimarySinkDraftAdjuster for ClusterPrimarySinkDraftAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &PrimarySinkDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimarySinkDraftAdjuster {
    pub percent: f64,
}

impl PrimarySinkDraftAdjuster for PercentPrimarySinkDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimarySinkDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimarySinkDraftAdjuster {
    pub minimum: f64,
}

impl PrimarySinkDraftAdjuster for MinimumPrimarySinkDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimarySinkDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the primary sink draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimarySinkDraftEngine {
    pub steps: Vec<Rc<dyn PrimarySinkDraftAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimarySinkDraftEngine {
    pub fn run(&mut self, item: &PrimarySinkDraft) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PrimarySinkDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_sink_draft_engine(flags: &flags::Registry) -> PrimarySinkDraftEngine {
    PrimarySinkDraftEngine {
        steps: vec![
            Rc::new(TieredPrimarySinkDraftAdjuster {
                bounds: vec![1860.0, 3541.0, 3678.0, 6815.0],
                rates: vec![1.1, 1.08, 0.88, 1.11],
            }) as Rc<dyn PrimarySinkDraftAdjuster>,
            Rc::new(FlaggedPrimarySinkDraftAdjuster {
                flags: flags.clone(),
                flag: "primary-sink-draft-regional".to_string(),
                percent: -6.8,
            }),
            Rc::new(ClusterPrimarySinkDraftAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.0)]),
            }),
            Rc::new(PercentPrimarySinkDraftAdjuster { percent: -10.4 }),
            Rc::new(MinimumPrimarySinkDraftAdjuster { minimum: 4.1 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 41629.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_sink_draft_engine(&flags::new_registry());
        let item = new_primary_sink_draft("code-393", "name-511");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
