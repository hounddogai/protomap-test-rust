use super::*;

use acme_kit::text;

/// Where a weighted overage digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedOverageDigestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl WeightedOverageDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for WeightedOverageDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted overage digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedOverageDigest {
    pub code: String,
    pub label: String,
    pub sequence: i64,
    pub attempts: i64,
    pub unit: String,
    pub contact_email: String,
    pub retries: i64,
    pub status: WeightedOverageDigestStatus,
}

impl WeightedOverageDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedOverageDigestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedOverageDigestStatus) -> bool {
        match self.status {
            WeightedOverageDigestStatus::Proposed => {
                return next == WeightedOverageDigestStatus::Approved;
            }
            WeightedOverageDigestStatus::Approved => {
                match next {
                    WeightedOverageDigestStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedOverageDigestStatus::Rejected => {
                return next == WeightedOverageDigestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedOverageDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted overage digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "weighted overage digest", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.attempts as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedOverageDigest {
        WeightedOverageDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            sequence: self.sequence,
            attempts: self.attempts,
            unit: self.unit.clone(),
            contact_email: self.contact_email.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a weighted overage digest in status proposed.
pub fn new_weighted_overage_digest(code: &str, label: &str) -> WeightedOverageDigest {
    WeightedOverageDigest {
        code: code.to_string(),
        label: label.to_string(),
        sequence: 0,
        attempts: 0,
        unit: "".to_string(),
        contact_email: "".to_string(),
        retries: 0,
        status: WeightedOverageDigestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_weighted_overage_digest("code-190", "label-689");
        assert_eq!(item.status, WeightedOverageDigestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_weighted_overage_digest("code-190", "label-689");
        if item.move_to(WeightedOverageDigestStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedOverageDigestStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_weighted_overage_digest("code-190", "label-689");
        item.status = WeightedOverageDigestStatus::Withdrawn;
        assert!(
            item.move_to(WeightedOverageDigestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
