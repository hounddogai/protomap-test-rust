use super::*;

use acme_kit::text;

/// Where a failover segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverSegmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FailoverSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FailoverSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSegment {
    pub code: String,
    pub label: String,
    pub owner_id: String,
    pub currency: String,
    pub score: f64,
    pub version: i64,
    pub status: FailoverSegmentStatus,
}

impl FailoverSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverSegmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverSegmentStatus) -> bool {
        match self.status {
            FailoverSegmentStatus::Requested => {
                match next {
                    FailoverSegmentStatus::Revoked | FailoverSegmentStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverSegmentStatus::Granted => {
                return next == FailoverSegmentStatus::Revoked || next == FailoverSegmentStatus::Expired;
            }
            FailoverSegmentStatus::Revoked => {
                return next == FailoverSegmentStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.version as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverSegment {
        FailoverSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            currency: self.currency.clone(),
            score: self.score,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a failover segment in status requested.
pub fn new_failover_segment(code: &str, label: &str) -> FailoverSegment {
    FailoverSegment {
        code: code.to_string(),
        label: label.to_string(),
        owner_id: "".to_string(),
        currency: "".to_string(),
        score: 0.0,
        version: 0,
        status: FailoverSegmentStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_failover_segment("code-967", "label-304");
        assert_eq!(item.status, FailoverSegmentStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_failover_segment("code-967", "label-304");
        if item.move_to(FailoverSegmentStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverSegmentStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_failover_segment("code-967", "label-304");
        item.status = FailoverSegmentStatus::Expired;
        assert!(
            item.move_to(FailoverSegmentStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
