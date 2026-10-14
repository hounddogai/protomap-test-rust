use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the zone milestone calculation.
pub trait ZoneMilestoneFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneMilestoneExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl ZoneMilestoneFactor for ZoneMilestoneExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &ZoneMilestone, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapZoneMilestoneAdjuster {
    pub cap: f64,
}

impl ZoneMilestoneFactor for CapZoneMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ZoneMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleZoneMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ZoneMilestoneFactor for TitleZoneMilestoneAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ZoneMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the zone milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneMilestoneEngine {
    pub steps: Vec<Rc<dyn ZoneMilestoneFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneMilestoneEngine {
    pub fn compute(&mut self, item: &ZoneMilestone) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ZoneMilestoneFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ZoneMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_zone_milestone_engine() -> ZoneMilestoneEngine {
    ZoneMilestoneEngine {
        steps: vec![
            Rc::new(ZoneMilestoneExpiresAtOffsetAdjuster { weight: 1.29 }) as Rc<dyn ZoneMilestoneFactor>,
            Rc::new(CapZoneMilestoneAdjuster { cap: 46045.0 }),
            Rc::new(TitleZoneMilestoneAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.06),
                    ("west".to_string(), 0.71),
                    ("east".to_string(), 1.37),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 43718.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_zone_milestone_engine();
        let item = new_zone_milestone("code-393", "title-37");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
