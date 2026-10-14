use super::*;

use acme_kit::text;

/// Where a primary identity review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryIdentityReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PrimaryIdentityReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for PrimaryIdentityReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary identity review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityReview {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub channel: String,
    pub key_id: String,
    pub notes: String,
    pub audience: String,
    pub priority: i64,
    pub created_at: i64,
    pub status: PrimaryIdentityReviewStatus,
}

impl PrimaryIdentityReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryIdentityReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryIdentityReviewStatus) -> bool {
        match self.status {
            PrimaryIdentityReviewStatus::Idle => {
                match next {
                    PrimaryIdentityReviewStatus::Warming | PrimaryIdentityReviewStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryIdentityReviewStatus::Warming => {
                return next == PrimaryIdentityReviewStatus::Draining;
            }
            PrimaryIdentityReviewStatus::Ready => {
                match next {
                    PrimaryIdentityReviewStatus::Draining | PrimaryIdentityReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryIdentityReviewStatus::Draining => {
                match next {
                    PrimaryIdentityReviewStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PrimaryIdentityReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary identity review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary identity review", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.cost - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryIdentityReview {
        PrimaryIdentityReview {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            channel: self.channel.clone(),
            key_id: self.key_id.clone(),
            notes: self.notes.clone(),
            audience: self.audience.clone(),
            priority: self.priority,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a primary identity review in status idle.
pub fn new_primary_identity_review(code: &str, label: &str) -> PrimaryIdentityReview {
    PrimaryIdentityReview {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        channel: "".to_string(),
        key_id: "".to_string(),
        notes: "".to_string(),
        audience: "".to_string(),
        priority: 0,
        created_at: 0,
        status: PrimaryIdentityReviewStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_primary_identity_review("code-792", "label-314");
        assert_eq!(item.status, PrimaryIdentityReviewStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_primary_identity_review("code-792", "label-314");
        if item.move_to(PrimaryIdentityReviewStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryIdentityReviewStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_primary_identity_review("code-792", "label-314");
        item.status = PrimaryIdentityReviewStatus::Stopped;
        assert!(
            item.move_to(PrimaryIdentityReviewStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
