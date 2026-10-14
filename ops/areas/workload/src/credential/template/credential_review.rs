use super::*;

use acme_kit::text;

/// Where a credential review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CredentialReviewStatus {
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

impl std::fmt::Display for CredentialReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a credential review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialReview {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub priority: i64,
    pub floor_value: f64,
    pub approved: bool,
    pub status: CredentialReviewStatus,
}

impl CredentialReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialReviewStatus) -> bool {
        match self.status {
            CredentialReviewStatus::Idle => {
                return next == CredentialReviewStatus::Draining || next == CredentialReviewStatus::Stopped;
            }
            CredentialReviewStatus::Warming => {
                match next {
                    CredentialReviewStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialReviewStatus::Ready => {
                return next == CredentialReviewStatus::Stopped;
            }
            CredentialReviewStatus::Draining => {
                match next {
                    CredentialReviewStatus::Stopped => {
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

    pub fn move_to(&mut self, next: CredentialReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.retries as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialReview {
        CredentialReview {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            priority: self.priority,
            floor_value: self.floor_value,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a credential review in status idle.
pub fn new_credential_review(code: &str, name: &str) -> CredentialReview {
    CredentialReview {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        priority: 0,
        floor_value: 0.0,
        approved: false,
        status: CredentialReviewStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_credential_review("code-787", "name-832");
        assert_eq!(item.status, CredentialReviewStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_credential_review("code-787", "name-832");
        if item.move_to(CredentialReviewStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialReviewStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_credential_review("code-787", "name-832");
        item.status = CredentialReviewStatus::Stopped;
        assert!(
            item.move_to(CredentialReviewStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
