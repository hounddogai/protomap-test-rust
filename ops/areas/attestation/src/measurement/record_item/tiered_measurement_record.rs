use super::*;

use acme_kit::text;

/// Where a tiered measurement record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredMeasurementRecordStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TieredMeasurementRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for TieredMeasurementRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered measurement record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementRecord {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub limit: i64,
    pub principal: String,
    pub unit: String,
    pub threshold: f64,
    pub pinned: bool,
    pub approved: bool,
    pub ratio: f64,
    pub ends_at: i64,
    pub status: TieredMeasurementRecordStatus,
}

impl TieredMeasurementRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredMeasurementRecordStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredMeasurementRecordStatus) -> bool {
        match self.status {
            TieredMeasurementRecordStatus::Draft => {
                match next {
                    TieredMeasurementRecordStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredMeasurementRecordStatus::Pending => {
                match next {
                    TieredMeasurementRecordStatus::Active | TieredMeasurementRecordStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredMeasurementRecordStatus::Active => {
                match next {
                    TieredMeasurementRecordStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredMeasurementRecordStatus::Suspended => {
                match next {
                    TieredMeasurementRecordStatus::Closed => {
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

    pub fn move_to(&mut self, next: TieredMeasurementRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered measurement record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered measurement record", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.ratio + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredMeasurementRecord {
        TieredMeasurementRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            limit: self.limit,
            principal: self.principal.clone(),
            unit: self.unit.clone(),
            threshold: self.threshold,
            pinned: self.pinned,
            approved: self.approved,
            ratio: self.ratio,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a tiered measurement record in status draft.
pub fn new_tiered_measurement_record(code: &str, title: &str) -> TieredMeasurementRecord {
    TieredMeasurementRecord {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        limit: 0,
        principal: "".to_string(),
        unit: "".to_string(),
        threshold: 0.0,
        pinned: false,
        approved: false,
        ratio: 0.0,
        ends_at: 0,
        status: TieredMeasurementRecordStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_tiered_measurement_record("code-223", "title-155");
        assert_eq!(item.status, TieredMeasurementRecordStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_tiered_measurement_record("code-223", "title-155");
        if item.move_to(TieredMeasurementRecordStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredMeasurementRecordStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_tiered_measurement_record("code-223", "title-155");
        item.status = TieredMeasurementRecordStatus::Closed;
        assert!(
            item.move_to(TieredMeasurementRecordStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
