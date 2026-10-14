use super::*;

use acme_kit::text;

/// Where a policy review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyReviewStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PolicyReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for PolicyReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One policy review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyReview {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub tags: Vec<String>,
    pub quantity: i64,
    pub segment: String,
    pub version: i64,
    pub visible: bool,
    pub tenant_id: String,
    pub port: i64,
    pub status: PolicyReviewStatus,
}

impl PolicyReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyReviewStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyReviewStatus) -> bool {
        match self.status {
            PolicyReviewStatus::Planned => {
                match next {
                    PolicyReviewStatus::Scheduled | PolicyReviewStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyReviewStatus::Scheduled => {
                match next {
                    PolicyReviewStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyReviewStatus::InProgress => {
                match next {
                    PolicyReviewStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyReviewStatus::Completed => {
                match next {
                    PolicyReviewStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PolicyReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.version as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyReview {
        PolicyReview {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            tags: self.tags.clone(),
            quantity: self.quantity,
            segment: self.segment.clone(),
            version: self.version,
            visible: self.visible,
            tenant_id: self.tenant_id.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a policy review in status planned.
pub fn new_policy_review(code: &str, title: &str) -> PolicyReview {
    PolicyReview {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        tags: Vec::new(),
        quantity: 0,
        segment: "".to_string(),
        version: 0,
        visible: false,
        tenant_id: "".to_string(),
        port: 0,
        status: PolicyReviewStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_policy_review("code-163", "title-813");
        assert_eq!(item.status, PolicyReviewStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_policy_review("code-163", "title-813");
        if item.move_to(PolicyReviewStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyReviewStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_policy_review("code-163", "title-813");
        item.status = PolicyReviewStatus::Cancelled;
        assert!(
            item.move_to(PolicyReviewStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
