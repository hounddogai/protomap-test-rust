use super::*;

use acme_kit::text;

/// Where a tiered measurement score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredMeasurementScoreStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TieredMeasurementScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TieredMeasurementScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered measurement score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementScore {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub active: bool,
    pub title: String,
    pub revision: i64,
    pub issuer: String,
    pub status: TieredMeasurementScoreStatus,
}

impl TieredMeasurementScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredMeasurementScoreStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredMeasurementScoreStatus) -> bool {
        match self.status {
            TieredMeasurementScoreStatus::Requested => {
                match next {
                    TieredMeasurementScoreStatus::Granted | TieredMeasurementScoreStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredMeasurementScoreStatus::Granted => {
                match next {
                    TieredMeasurementScoreStatus::Revoked | TieredMeasurementScoreStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredMeasurementScoreStatus::Revoked => {
                return next == TieredMeasurementScoreStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredMeasurementScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered measurement score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.revision as f64
    }

    pub fn copy(&self) -> TieredMeasurementScore {
        TieredMeasurementScore {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            active: self.active,
            title: self.title.clone(),
            revision: self.revision,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a tiered measurement score in status requested.
pub fn new_tiered_measurement_score(code: &str, name: &str) -> TieredMeasurementScore {
    TieredMeasurementScore {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        active: false,
        title: "".to_string(),
        revision: 0,
        issuer: "".to_string(),
        status: TieredMeasurementScoreStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_tiered_measurement_score("code-37", "name-308");
        assert_eq!(item.status, TieredMeasurementScoreStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_tiered_measurement_score("code-37", "name-308");
        if item.move_to(TieredMeasurementScoreStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredMeasurementScoreStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_tiered_measurement_score("code-37", "name-308");
        item.status = TieredMeasurementScoreStatus::Expired;
        assert!(
            item.move_to(TieredMeasurementScoreStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
