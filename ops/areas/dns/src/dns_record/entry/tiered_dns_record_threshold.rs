use super::*;

use acme_kit::text;

/// Where a tiered dns record threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredDnsRecordThresholdStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TieredDnsRecordThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for TieredDnsRecordThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered dns record threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordThreshold {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub rotation_days: i64,
    pub budget: f64,
    pub external_ref: String,
    pub status: TieredDnsRecordThresholdStatus,
}

impl TieredDnsRecordThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredDnsRecordThresholdStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredDnsRecordThresholdStatus) -> bool {
        match self.status {
            TieredDnsRecordThresholdStatus::Open => {
                return next == TieredDnsRecordThresholdStatus::Archived;
            }
            TieredDnsRecordThresholdStatus::InReview => {
                match next {
                    TieredDnsRecordThresholdStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredDnsRecordThresholdStatus::Resolved => {
                match next {
                    TieredDnsRecordThresholdStatus::Reopened | TieredDnsRecordThresholdStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredDnsRecordThresholdStatus::Reopened => {
                match next {
                    TieredDnsRecordThresholdStatus::Archived => {
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

    pub fn move_to(&mut self, next: TieredDnsRecordThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered dns record threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered dns record threshold", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.rotation_days as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredDnsRecordThreshold {
        TieredDnsRecordThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            rotation_days: self.rotation_days,
            budget: self.budget,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered dns record threshold in status open.
pub fn new_tiered_dns_record_threshold(code: &str, name: &str) -> TieredDnsRecordThreshold {
    TieredDnsRecordThreshold {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        rotation_days: 0,
        budget: 0.0,
        external_ref: "".to_string(),
        status: TieredDnsRecordThresholdStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tiered_dns_record_threshold("code-872", "name-804");
        assert_eq!(item.status, TieredDnsRecordThresholdStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_tiered_dns_record_threshold("code-872", "name-804");
        if item.move_to(TieredDnsRecordThresholdStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredDnsRecordThresholdStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tiered_dns_record_threshold("code-872", "name-804");
        item.status = TieredDnsRecordThresholdStatus::Archived;
        assert!(
            item.move_to(TieredDnsRecordThresholdStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
