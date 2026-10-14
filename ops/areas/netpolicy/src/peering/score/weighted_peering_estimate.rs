use super::*;

use acme_kit::text;

/// Where a weighted peering estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedPeeringEstimateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl WeightedPeeringEstimateStatus {
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

impl std::fmt::Display for WeightedPeeringEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted peering estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPeeringEstimate {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub description: String,
    pub scope: String,
    pub score: f64,
    pub status: WeightedPeeringEstimateStatus,
}

impl WeightedPeeringEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedPeeringEstimateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedPeeringEstimateStatus) -> bool {
        match self.status {
            WeightedPeeringEstimateStatus::Open => {
                match next {
                    WeightedPeeringEstimateStatus::InReview | WeightedPeeringEstimateStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedPeeringEstimateStatus::InReview => {
                match next {
                    WeightedPeeringEstimateStatus::Resolved | WeightedPeeringEstimateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedPeeringEstimateStatus::Resolved => {
                match next {
                    WeightedPeeringEstimateStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedPeeringEstimateStatus::Reopened => {
                match next {
                    WeightedPeeringEstimateStatus::Archived => {
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

    pub fn move_to(&mut self, next: WeightedPeeringEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted peering estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> WeightedPeeringEstimate {
        WeightedPeeringEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            description: self.description.clone(),
            scope: self.scope.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a weighted peering estimate in status open.
pub fn new_weighted_peering_estimate(code: &str, title: &str) -> WeightedPeeringEstimate {
    WeightedPeeringEstimate {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        description: "".to_string(),
        scope: "".to_string(),
        score: 0.0,
        status: WeightedPeeringEstimateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_weighted_peering_estimate("code-490", "title-245");
        assert_eq!(item.status, WeightedPeeringEstimateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_weighted_peering_estimate("code-490", "title-245");
        if item.move_to(WeightedPeeringEstimateStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedPeeringEstimateStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_weighted_peering_estimate("code-490", "title-245");
        item.status = WeightedPeeringEstimateStatus::Archived;
        assert!(
            item.move_to(WeightedPeeringEstimateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
