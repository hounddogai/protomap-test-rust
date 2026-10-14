use super::*;

use acme_kit::text;

/// Where a weighted peering review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedPeeringReviewStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl WeightedPeeringReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for WeightedPeeringReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted peering review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPeeringReview {
    pub code: String,
    pub title: String,
    pub threshold: f64,
    pub quota_units: i64,
    pub expires_at: i64,
    pub tenant_id: String,
    pub grace_days: i64,
    pub ceiling: f64,
    pub status: WeightedPeeringReviewStatus,
}

impl WeightedPeeringReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedPeeringReviewStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedPeeringReviewStatus) -> bool {
        match self.status {
            WeightedPeeringReviewStatus::Created => {
                match next {
                    WeightedPeeringReviewStatus::Validated | WeightedPeeringReviewStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedPeeringReviewStatus::Validated => {
                return next == WeightedPeeringReviewStatus::Published || next == WeightedPeeringReviewStatus::Retired;
            }
            WeightedPeeringReviewStatus::Published => {
                return next == WeightedPeeringReviewStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedPeeringReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted peering review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.expires_at as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedPeeringReview {
        WeightedPeeringReview {
            code: self.code.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            quota_units: self.quota_units,
            expires_at: self.expires_at,
            tenant_id: self.tenant_id.clone(),
            grace_days: self.grace_days,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a weighted peering review in status created.
pub fn new_weighted_peering_review(code: &str, title: &str) -> WeightedPeeringReview {
    WeightedPeeringReview {
        code: code.to_string(),
        title: title.to_string(),
        threshold: 0.0,
        quota_units: 0,
        expires_at: 0,
        tenant_id: "".to_string(),
        grace_days: 0,
        ceiling: 0.0,
        status: WeightedPeeringReviewStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_weighted_peering_review("code-243", "title-911");
        assert_eq!(item.status, WeightedPeeringReviewStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_weighted_peering_review("code-243", "title-911");
        if item.move_to(WeightedPeeringReviewStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedPeeringReviewStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_weighted_peering_review("code-243", "title-911");
        item.status = WeightedPeeringReviewStatus::Retired;
        assert!(
            item.move_to(WeightedPeeringReviewStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
