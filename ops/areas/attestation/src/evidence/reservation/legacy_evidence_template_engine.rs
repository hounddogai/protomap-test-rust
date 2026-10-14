use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the legacy evidence template calculation.
pub trait LegacyEvidenceTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyEvidenceTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyEvidenceTemplateAdjuster {
    pub percent: f64,
}

impl LegacyEvidenceTemplateModifier for PercentLegacyEvidenceTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyEvidenceTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelLegacyEvidenceTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LegacyEvidenceTemplateModifier for ChannelLegacyEvidenceTemplateAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &LegacyEvidenceTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceTemplateRateOffsetAdjuster {
    pub weight: f64,
}

impl LegacyEvidenceTemplateModifier for LegacyEvidenceTemplateRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyEvidenceTemplate, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

/// Runs the legacy evidence template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyEvidenceTemplateEngine {
    pub steps: Vec<Rc<dyn LegacyEvidenceTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyEvidenceTemplateEngine {
    pub fn price(&mut self, item: &LegacyEvidenceTemplate) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyEvidenceTemplateModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyEvidenceTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_evidence_template_engine() -> LegacyEvidenceTemplateEngine {
    LegacyEvidenceTemplateEngine {
        steps: vec![
            Rc::new(PercentLegacyEvidenceTemplateAdjuster { percent: 8.7 }) as Rc<dyn LegacyEvidenceTemplateModifier>,
            Rc::new(ChannelLegacyEvidenceTemplateAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.07), ("online".to_string(), 1.37)]),
            }),
            Rc::new(LegacyEvidenceTemplateRateOffsetAdjuster { weight: 1.65 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 87248.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_evidence_template_engine();
        let item = new_legacy_evidence_template("code-469", "label-669");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
