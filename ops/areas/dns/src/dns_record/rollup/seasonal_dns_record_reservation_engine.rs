use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal dns record reservation calculation.
pub trait SeasonalDnsRecordReservationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalDnsRecordReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalDnsRecordReservationAdjuster {
    pub minimum: f64,
}

impl SeasonalDnsRecordReservationAdjuster for MinimumSeasonalDnsRecordReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalDnsRecordReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalDnsRecordReservationAdjuster {
    pub step: f64,
}

impl SeasonalDnsRecordReservationAdjuster for RoundingSeasonalDnsRecordReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalDnsRecordReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelSeasonalDnsRecordReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalDnsRecordReservationAdjuster for ChannelSeasonalDnsRecordReservationAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &SeasonalDnsRecordReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalDnsRecordReservationAdjuster {
    pub percent: f64,
}

impl SeasonalDnsRecordReservationAdjuster for PercentSeasonalDnsRecordReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalDnsRecordReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalDnsRecordReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalDnsRecordReservationAdjuster for FlaggedSeasonalDnsRecordReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalDnsRecordReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the seasonal dns record reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalDnsRecordReservationEngine {
    pub steps: Vec<Rc<dyn SeasonalDnsRecordReservationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalDnsRecordReservationEngine {
    pub fn evaluate(&mut self, item: &SeasonalDnsRecordReservation) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalDnsRecordReservationAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalDnsRecordReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_seasonal_dns_record_reservation_engine(flags: &flags::Registry) -> SeasonalDnsRecordReservationEngine {
    SeasonalDnsRecordReservationEngine {
        steps: vec![
            Rc::new(MinimumSeasonalDnsRecordReservationAdjuster { minimum: 31.6 })
                as Rc<dyn SeasonalDnsRecordReservationAdjuster>,
            Rc::new(RoundingSeasonalDnsRecordReservationAdjuster { step: 0.05 }),
            Rc::new(ChannelSeasonalDnsRecordReservationAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.29),
                    ("east".to_string(), 1.07),
                    ("west".to_string(), 0.98),
                ]),
            }),
            Rc::new(PercentSeasonalDnsRecordReservationAdjuster { percent: -8.5 }),
            Rc::new(FlaggedSeasonalDnsRecordReservationAdjuster {
                flags: flags.clone(),
                flag: "seasonal-dns-record-reservation-scheduled".to_string(),
                percent: 8.8,
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 63632.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_dns_record_reservation_engine(&flags::new_registry());
        let item = new_seasonal_dns_record_reservation("code-395", "name-880");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
