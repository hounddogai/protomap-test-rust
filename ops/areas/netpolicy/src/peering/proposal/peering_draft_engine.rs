use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the peering draft calculation.
pub trait PeeringDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringDraft, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPeeringDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PeeringDraftStep for FlaggedPeeringDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PeeringDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPeeringDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PeeringDraftStep for RegionPeeringDraftAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &PeeringDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPeeringDraftAdjuster {
    pub cap: f64,
}

impl PeeringDraftStep for CapPeeringDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PeeringDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringDraftTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl PeeringDraftStep for PeeringDraftTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &PeeringDraft, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

/// Runs the peering draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringDraftEngine {
    pub steps: Vec<Rc<dyn PeeringDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringDraftEngine {
    pub fn evaluate(&mut self, item: &PeeringDraft) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[PeeringDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_peering_draft_engine(flags: &flags::Registry) -> PeeringDraftEngine {
    PeeringDraftEngine {
        steps: vec![
            Rc::new(FlaggedPeeringDraftAdjuster {
                flags: flags.clone(),
                flag: "peering-draft-scheduled".to_string(),
                percent: 2.9,
            }) as Rc<dyn PeeringDraftStep>,
            Rc::new(RegionPeeringDraftAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.84),
                    ("central".to_string(), 0.87),
                    ("north".to_string(), 0.72),
                ]),
            }),
            Rc::new(CapPeeringDraftAdjuster { cap: 35081.0 }),
            Rc::new(PeeringDraftTtlSecondsOffsetAdjuster { weight: 1.84 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 32733.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_draft_engine(&flags::new_registry());
        let item = new_peering_draft("code-907", "name-137");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
