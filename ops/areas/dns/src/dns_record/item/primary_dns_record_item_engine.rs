use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary dns record item calculation.
pub trait PrimaryDnsRecordItemModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryDnsRecordItem, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitPrimaryDnsRecordItemAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryDnsRecordItemModifier for UnitPrimaryDnsRecordItemAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &PrimaryDnsRecordItem, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryDnsRecordItemAdjuster {
    pub cap: f64,
}

impl PrimaryDnsRecordItemModifier for CapPrimaryDnsRecordItemAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryDnsRecordItem, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordItemGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryDnsRecordItemModifier for PrimaryDnsRecordItemGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryDnsRecordItem, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryDnsRecordItemAdjuster {
    pub step: f64,
}

impl PrimaryDnsRecordItemModifier for RoundingPrimaryDnsRecordItemAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryDnsRecordItem, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryDnsRecordItemAdjuster {
    pub minimum: f64,
}

impl PrimaryDnsRecordItemModifier for MinimumPrimaryDnsRecordItemAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryDnsRecordItem, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryDnsRecordItemAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryDnsRecordItemModifier for FlaggedPrimaryDnsRecordItemAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryDnsRecordItem, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary dns record item steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryDnsRecordItemEngine {
    pub steps: Vec<Rc<dyn PrimaryDnsRecordItemModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryDnsRecordItemEngine {
    pub fn compute(&mut self, item: &PrimaryDnsRecordItem) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryDnsRecordItemModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryDnsRecordItem]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_dns_record_item_engine(flags: &flags::Registry) -> PrimaryDnsRecordItemEngine {
    PrimaryDnsRecordItemEngine {
        steps: vec![
            Rc::new(UnitPrimaryDnsRecordItemAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.75), ("central".to_string(), 0.95)]),
            }) as Rc<dyn PrimaryDnsRecordItemModifier>,
            Rc::new(CapPrimaryDnsRecordItemAdjuster { cap: 24830.0 }),
            Rc::new(PrimaryDnsRecordItemGraceDaysOffsetAdjuster { weight: 0.85 }),
            Rc::new(RoundingPrimaryDnsRecordItemAdjuster { step: 5.0 }),
            Rc::new(MinimumPrimaryDnsRecordItemAdjuster { minimum: 6.9 }),
            Rc::new(FlaggedPrimaryDnsRecordItemAdjuster {
                flags: flags.clone(),
                flag: "primary-dns-record-item-adaptive".to_string(),
                percent: -8.0,
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 56195.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_dns_record_item_engine(&flags::new_registry());
        let item = new_primary_dns_record_item("code-366", "name-659");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
