use super::*;

use acme_kit::text;

/// Where a pending egress review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingEgressReviewStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PendingEgressReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PendingEgressReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending egress review as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingEgressReview {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub weight: f64,
    pub reviewer: String,
    pub timezone: String,
    pub quantity: i64,
    pub scope: String,
    pub active: bool,
    pub issuer: String,
    pub status: PendingEgressReviewStatus,
}

impl PendingEgressReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingEgressReviewStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingEgressReviewStatus) -> bool {
        match self.status {
            PendingEgressReviewStatus::Requested => {
                match next {
                    PendingEgressReviewStatus::Granted | PendingEgressReviewStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingEgressReviewStatus::Granted => {
                return next == PendingEgressReviewStatus::Revoked || next == PendingEgressReviewStatus::Expired;
            }
            PendingEgressReviewStatus::Revoked => {
                return next == PendingEgressReviewStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingEgressReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending egress review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.weight - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingEgressReview {
        PendingEgressReview {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            weight: self.weight,
            reviewer: self.reviewer.clone(),
            timezone: self.timezone.clone(),
            quantity: self.quantity,
            scope: self.scope.clone(),
            active: self.active,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a pending egress review in status requested.
pub fn new_pending_egress_review(code: &str, name: &str) -> PendingEgressReview {
    PendingEgressReview {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        weight: 0.0,
        reviewer: "".to_string(),
        timezone: "".to_string(),
        quantity: 0,
        scope: "".to_string(),
        active: false,
        issuer: "".to_string(),
        status: PendingEgressReviewStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_pending_egress_review("code-213", "name-499");
        assert_eq!(item.status, PendingEgressReviewStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_pending_egress_review("code-213", "name-499");
        if item.move_to(PendingEgressReviewStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingEgressReviewStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_pending_egress_review("code-213", "name-499");
        item.status = PendingEgressReviewStatus::Expired;
        assert!(
            item.move_to(PendingEgressReviewStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
