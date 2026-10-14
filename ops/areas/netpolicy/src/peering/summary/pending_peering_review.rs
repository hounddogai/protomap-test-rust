use super::*;

use acme_kit::text;

/// Where a pending peering review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPeeringReviewStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PendingPeeringReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PendingPeeringReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending peering review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPeeringReview {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub quota_units: i64,
    pub visible: bool,
    pub priority: i64,
    pub source: String,
    pub status: PendingPeeringReviewStatus,
}

impl PendingPeeringReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPeeringReviewStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPeeringReviewStatus) -> bool {
        match self.status {
            PendingPeeringReviewStatus::Requested => {
                match next {
                    PendingPeeringReviewStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPeeringReviewStatus::Granted => {
                return next == PendingPeeringReviewStatus::Expired;
            }
            PendingPeeringReviewStatus::Revoked => {
                match next {
                    PendingPeeringReviewStatus::Expired => {
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

    pub fn move_to(&mut self, next: PendingPeeringReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending peering review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.quota_units as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingPeeringReview {
        PendingPeeringReview {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            quota_units: self.quota_units,
            visible: self.visible,
            priority: self.priority,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a pending peering review in status requested.
pub fn new_pending_peering_review(code: &str, title: &str) -> PendingPeeringReview {
    PendingPeeringReview {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        quota_units: 0,
        visible: false,
        priority: 0,
        source: "".to_string(),
        status: PendingPeeringReviewStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_pending_peering_review("code-843", "title-490");
        assert_eq!(item.status, PendingPeeringReviewStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_pending_peering_review("code-843", "title-490");
        if item.move_to(PendingPeeringReviewStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingPeeringReviewStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_pending_peering_review("code-843", "title-490");
        item.status = PendingPeeringReviewStatus::Expired;
        assert!(
            item.move_to(PendingPeeringReviewStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
