use super::*;

use acme_kit::text;

/// Where a primary zone metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryZoneMetricStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimaryZoneMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PrimaryZoneMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary zone metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryZoneMetric {
    pub code: String,
    pub name: String,
    pub rotation_days: i64,
    pub currency: String,
    pub pinned: bool,
    pub issuer: String,
    pub unit: String,
    pub status: PrimaryZoneMetricStatus,
}

impl PrimaryZoneMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryZoneMetricStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryZoneMetricStatus) -> bool {
        match self.status {
            PrimaryZoneMetricStatus::New => {
                match next {
                    PrimaryZoneMetricStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryZoneMetricStatus::Queued => {
                return next == PrimaryZoneMetricStatus::Failed;
            }
            PrimaryZoneMetricStatus::Running => {
                match next {
                    PrimaryZoneMetricStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryZoneMetricStatus::Done => {
                match next {
                    PrimaryZoneMetricStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryZoneMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary zone metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> PrimaryZoneMetric {
        PrimaryZoneMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            currency: self.currency.clone(),
            pinned: self.pinned,
            issuer: self.issuer.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a primary zone metric in status new.
pub fn new_primary_zone_metric(code: &str, name: &str) -> PrimaryZoneMetric {
    PrimaryZoneMetric {
        code: code.to_string(),
        name: name.to_string(),
        rotation_days: 0,
        currency: "".to_string(),
        pinned: false,
        issuer: "".to_string(),
        unit: "".to_string(),
        status: PrimaryZoneMetricStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_primary_zone_metric("code-722", "name-104");
        assert_eq!(item.status, PrimaryZoneMetricStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_primary_zone_metric("code-722", "name-104");
        if item.move_to(PrimaryZoneMetricStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryZoneMetricStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_primary_zone_metric("code-722", "name-104");
        item.status = PrimaryZoneMetricStatus::Failed;
        assert!(
            item.move_to(PrimaryZoneMetricStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
