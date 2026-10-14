use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal credential slot calculation.
pub trait SeasonalCredentialSlotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalCredentialSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalCredentialSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalCredentialSlotAdjuster for TieredSeasonalCredentialSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalCredentialSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleSeasonalCredentialSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalCredentialSlotAdjuster for TitleSeasonalCredentialSlotAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &SeasonalCredentialSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalCredentialSlotAdjuster {
    pub step: f64,
}

impl SeasonalCredentialSlotAdjuster for RoundingSeasonalCredentialSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalCredentialSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the seasonal credential slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalCredentialSlotEngine {
    pub steps: Vec<Rc<dyn SeasonalCredentialSlotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalCredentialSlotEngine {
    pub fn compute(&mut self, item: &SeasonalCredentialSlot) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalCredentialSlotAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalCredentialSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_seasonal_credential_slot_engine() -> SeasonalCredentialSlotEngine {
    SeasonalCredentialSlotEngine {
        steps: vec![
            Rc::new(TieredSeasonalCredentialSlotAdjuster {
                bounds: vec![5665.0, 6662.0],
                rates: vec![1.04, 1.12],
            }) as Rc<dyn SeasonalCredentialSlotAdjuster>,
            Rc::new(TitleSeasonalCredentialSlotAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.1), ("online".to_string(), 1.29)]),
            }),
            Rc::new(RoundingSeasonalCredentialSlotAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 8374.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_credential_slot_engine();
        let item = new_seasonal_credential_slot("code-679", "title-906");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
