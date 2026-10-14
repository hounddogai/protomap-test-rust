use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual evidence template calculation.
pub trait ManualEvidenceTemplateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualEvidenceTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualEvidenceTemplateAdjuster {
    pub cap: f64,
}

impl ManualEvidenceTemplateStep for CapManualEvidenceTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualEvidenceTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameManualEvidenceTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualEvidenceTemplateStep for NameManualEvidenceTemplateAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ManualEvidenceTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualEvidenceTemplateAdjuster {
    pub step: f64,
}

impl ManualEvidenceTemplateStep for RoundingManualEvidenceTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualEvidenceTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualEvidenceTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualEvidenceTemplateStep for TieredManualEvidenceTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualEvidenceTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceTemplateMarginOffsetAdjuster {
    pub weight: f64,
}

impl ManualEvidenceTemplateStep for ManualEvidenceTemplateMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &ManualEvidenceTemplate, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

/// Runs the manual evidence template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualEvidenceTemplateEngine {
    pub steps: Vec<Rc<dyn ManualEvidenceTemplateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualEvidenceTemplateEngine {
    pub fn price(&mut self, item: &ManualEvidenceTemplate) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[ManualEvidenceTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_evidence_template_engine() -> ManualEvidenceTemplateEngine {
    ManualEvidenceTemplateEngine {
        steps: vec![
            Rc::new(CapManualEvidenceTemplateAdjuster { cap: 15931.0 }) as Rc<dyn ManualEvidenceTemplateStep>,
            Rc::new(NameManualEvidenceTemplateAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.19), ("retail".to_string(), 1.37)]),
            }),
            Rc::new(RoundingManualEvidenceTemplateAdjuster { step: 5.0 }),
            Rc::new(TieredManualEvidenceTemplateAdjuster {
                bounds: vec![4589.0, 9545.0],
                rates: vec![1.03, 0.89],
            }),
            Rc::new(ManualEvidenceTemplateMarginOffsetAdjuster { weight: 0.58 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 36971.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_evidence_template_engine();
        let item = new_manual_evidence_template("code-882", "name-65");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
