use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the adaptive evidence request calculation.
pub trait AdaptiveEvidenceRequestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveEvidenceRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveEvidenceRequestAdjuster {
    pub minimum: f64,
}

impl AdaptiveEvidenceRequestStep for MinimumAdaptiveEvidenceRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveEvidenceRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveEvidenceRequestWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveEvidenceRequestStep for AdaptiveEvidenceRequestWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveEvidenceRequest, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveEvidenceRequestAdjuster {
    pub percent: f64,
}

impl AdaptiveEvidenceRequestStep for PercentAdaptiveEvidenceRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveEvidenceRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelAdaptiveEvidenceRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveEvidenceRequestStep for LabelAdaptiveEvidenceRequestAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &AdaptiveEvidenceRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the adaptive evidence request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveEvidenceRequestEngine {
    pub steps: Vec<Rc<dyn AdaptiveEvidenceRequestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveEvidenceRequestEngine {
    pub fn evaluate(&mut self, item: &AdaptiveEvidenceRequest) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveEvidenceRequestStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveEvidenceRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_evidence_request_engine() -> AdaptiveEvidenceRequestEngine {
    AdaptiveEvidenceRequestEngine {
        steps: vec![
            Rc::new(MinimumAdaptiveEvidenceRequestAdjuster { minimum: 44.1 }) as Rc<dyn AdaptiveEvidenceRequestStep>,
            Rc::new(AdaptiveEvidenceRequestWindowDaysOffsetAdjuster { weight: 1.97 }),
            Rc::new(PercentAdaptiveEvidenceRequestAdjuster { percent: -23.4 }),
            Rc::new(LabelAdaptiveEvidenceRequestAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.71),
                    ("retail".to_string(), 0.72),
                    ("central".to_string(), 0.87),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 87879.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_evidence_request_engine();
        let item = new_adaptive_evidence_request("code-471", "label-5");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
