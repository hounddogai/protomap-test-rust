use super::*;

use acme_kit::text;

/// Where a region pin review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionPinReviewStatus {
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

impl std::fmt::Display for RegionPinReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a region pin review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinReview {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub namespace: String,
    pub cluster: String,
    pub score: f64,
    pub owner_id: String,
    pub status: RegionPinReviewStatus,
}

impl RegionPinReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinReviewStatus) -> bool {
        match self.status {
            RegionPinReviewStatus::Idle => {
                match next {
                    RegionPinReviewStatus::Ready | RegionPinReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinReviewStatus::Warming => {
                match next {
                    RegionPinReviewStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinReviewStatus::Ready => {
                match next {
                    RegionPinReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinReviewStatus::Draining => {
                return next == RegionPinReviewStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin review {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.score - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinReview {
        RegionPinReview {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            namespace: self.namespace.clone(),
            cluster: self.cluster.clone(),
            score: self.score,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a region pin review in status idle.
pub fn new_region_pin_review(code: &str, label: &str) -> RegionPinReview {
    RegionPinReview {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        namespace: "".to_string(),
        cluster: "".to_string(),
        score: 0.0,
        owner_id: "".to_string(),
        status: RegionPinReviewStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_region_pin_review("code-320", "label-909");
        assert_eq!(item.status, RegionPinReviewStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_region_pin_review("code-320", "label-909");
        if item.move_to(RegionPinReviewStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinReviewStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_region_pin_review("code-320", "label-909");
        item.status = RegionPinReviewStatus::Stopped;
        assert!(
            item.move_to(RegionPinReviewStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
