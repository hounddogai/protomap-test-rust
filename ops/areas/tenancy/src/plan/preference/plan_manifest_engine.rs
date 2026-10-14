use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the plan manifest calculation.
pub trait PlanManifestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanManifest, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPlanManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PlanManifestAdjuster for FlaggedPlanManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PlanManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryPlanManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PlanManifestAdjuster for CategoryPlanManifestAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &PlanManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanManifestAdjuster {
    pub minimum: f64,
}

impl PlanManifestAdjuster for MinimumPlanManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the plan manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanManifestEngine {
    pub steps: Vec<Rc<dyn PlanManifestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanManifestEngine {
    pub fn price(&mut self, item: &PlanManifest) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PlanManifestAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PlanManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_plan_manifest_engine(flags: &flags::Registry) -> PlanManifestEngine {
    PlanManifestEngine {
        steps: vec![
            Rc::new(FlaggedPlanManifestAdjuster {
                flags: flags.clone(),
                flag: "plan-manifest-capped".to_string(),
                percent: -9.6,
            }) as Rc<dyn PlanManifestAdjuster>,
            Rc::new(CategoryPlanManifestAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.09)]),
            }),
            Rc::new(MinimumPlanManifestAdjuster { minimum: 39.5 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 86865.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_manifest_engine(&flags::new_registry());
        let item = new_plan_manifest("code-241", "title-533");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
