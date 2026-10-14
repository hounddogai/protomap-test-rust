use super::*;

use acme_kit::text;

/// Where a limit request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitRequestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LimitRequestStatus {
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

impl std::fmt::Display for LimitRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit request as the limit team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitRequest {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub timezone: String,
    pub retries: i64,
    pub revision: i64,
    pub contact_email: String,
    pub visible: bool,
    pub notes: String,
    pub currency: String,
    pub status: LimitRequestStatus,
}

impl LimitRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitRequestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitRequestStatus) -> bool {
        match self.status {
            LimitRequestStatus::Open => {
                return next == LimitRequestStatus::Reopened;
            }
            LimitRequestStatus::InReview => {
                match next {
                    LimitRequestStatus::Resolved | LimitRequestStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitRequestStatus::Resolved => {
                match next {
                    LimitRequestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitRequestStatus::Reopened => {
                return next == LimitRequestStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.retries as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitRequest {
        LimitRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            timezone: self.timezone.clone(),
            retries: self.retries,
            revision: self.revision,
            contact_email: self.contact_email.clone(),
            visible: self.visible,
            notes: self.notes.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a limit request in status open.
pub fn new_limit_request(code: &str, title: &str) -> LimitRequest {
    LimitRequest {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        timezone: "".to_string(),
        retries: 0,
        revision: 0,
        contact_email: "".to_string(),
        visible: false,
        notes: "".to_string(),
        currency: "".to_string(),
        status: LimitRequestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_limit_request("code-485", "title-722");
        assert_eq!(item.status, LimitRequestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_limit_request("code-485", "title-722");
        if item.move_to(LimitRequestStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitRequestStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_limit_request("code-485", "title-722");
        item.status = LimitRequestStatus::Archived;
        assert!(item.move_to(LimitRequestStatus::Open).is_err(), "expected move to fail");
    }
}
