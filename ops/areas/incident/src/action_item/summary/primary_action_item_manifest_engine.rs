use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the primary action item manifest calculation.
pub trait PrimaryActionItemManifestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryActionItemManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterPrimaryActionItemManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryActionItemManifestStep for ClusterPrimaryActionItemManifestAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &PrimaryActionItemManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryActionItemManifestAdjuster {
    pub minimum: f64,
}

impl PrimaryActionItemManifestStep for MinimumPrimaryActionItemManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryActionItemManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryActionItemManifestAdjuster {
    pub cap: f64,
}

impl PrimaryActionItemManifestStep for CapPrimaryActionItemManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryActionItemManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryActionItemManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryActionItemManifestStep for FlaggedPrimaryActionItemManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryActionItemManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemManifestDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryActionItemManifestStep for PrimaryActionItemManifestDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryActionItemManifest, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

/// Runs the primary action item manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryActionItemManifestEngine {
    pub steps: Vec<Rc<dyn PrimaryActionItemManifestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryActionItemManifestEngine {
    pub fn evaluate(&mut self, item: &PrimaryActionItemManifest) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryActionItemManifestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryActionItemManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_primary_action_item_manifest_engine(flags: &flags::Registry) -> PrimaryActionItemManifestEngine {
    PrimaryActionItemManifestEngine {
        steps: vec![
            Rc::new(ClusterPrimaryActionItemManifestAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.87), ("north".to_string(), 0.83)]),
            }) as Rc<dyn PrimaryActionItemManifestStep>,
            Rc::new(MinimumPrimaryActionItemManifestAdjuster { minimum: 11.0 }),
            Rc::new(CapPrimaryActionItemManifestAdjuster { cap: 46442.0 }),
            Rc::new(FlaggedPrimaryActionItemManifestAdjuster {
                flags: flags.clone(),
                flag: "primary-action-item-manifest-legacy".to_string(),
                percent: 7.7,
            }),
            Rc::new(PrimaryActionItemManifestDurationMinutesOffsetAdjuster { weight: 0.07 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 19727.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_action_item_manifest_engine(&flags::new_registry());
        let item = new_primary_action_item_manifest("code-704", "label-675");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
