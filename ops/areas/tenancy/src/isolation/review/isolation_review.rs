use super::*;

use acme_kit::text;

/// Where a isolation review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationReviewStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl IsolationReviewStatus {
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

impl std::fmt::Display for IsolationReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One isolation review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationReview {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub external_ref: String,
    pub title: String,
    pub threshold: f64,
    pub tags: Vec<String>,
    pub active: bool,
    pub retries: i64,
    pub country: String,
    pub key_id: String,
    pub status: IsolationReviewStatus,
}

impl IsolationReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationReviewStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationReviewStatus) -> bool {
        match self.status {
            IsolationReviewStatus::Planned => {
                return next == IsolationReviewStatus::InProgress || next == IsolationReviewStatus::Completed;
            }
            IsolationReviewStatus::Scheduled => {
                return next == IsolationReviewStatus::Completed;
            }
            IsolationReviewStatus::InProgress => {
                match next {
                    IsolationReviewStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationReviewStatus::Completed => {
                match next {
                    IsolationReviewStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: IsolationReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_load(&self) -> f64 {
        ((self.threshold - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationReview {
        IsolationReview {
            code: self.code.clone(),
            name: self.name.clone(),
            locale: self.locale.clone(),
            external_ref: self.external_ref.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            tags: self.tags.clone(),
            active: self.active,
            retries: self.retries,
            country: self.country.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a isolation review in status planned.
pub fn new_isolation_review(code: &str, name: &str) -> IsolationReview {
    IsolationReview {
        code: code.to_string(),
        name: name.to_string(),
        locale: "".to_string(),
        external_ref: "".to_string(),
        title: "".to_string(),
        threshold: 0.0,
        tags: Vec::new(),
        active: false,
        retries: 0,
        country: "".to_string(),
        key_id: "".to_string(),
        status: IsolationReviewStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_isolation_review("code-574", "name-912");
        assert_eq!(item.status, IsolationReviewStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_isolation_review("code-574", "name-912");
        if item.move_to(IsolationReviewStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationReviewStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_isolation_review("code-574", "name-912");
        item.status = IsolationReviewStatus::Cancelled;
        assert!(
            item.move_to(IsolationReviewStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
