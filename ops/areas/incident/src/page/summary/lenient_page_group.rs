use super::*;

use acme_kit::text;

/// Where a lenient page group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientPageGroupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientPageGroupStatus {
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

impl std::fmt::Display for LenientPageGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient page group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPageGroup {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub archived: bool,
    pub contact_email: String,
    pub batch_size: i64,
    pub issuer: String,
    pub port: i64,
    pub cost: f64,
    pub owner_id: String,
    pub status: LenientPageGroupStatus,
}

impl LenientPageGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientPageGroupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientPageGroupStatus) -> bool {
        match self.status {
            LenientPageGroupStatus::Open => {
                match next {
                    LenientPageGroupStatus::InReview | LenientPageGroupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPageGroupStatus::InReview => {
                match next {
                    LenientPageGroupStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPageGroupStatus::Resolved => {
                match next {
                    LenientPageGroupStatus::Reopened | LenientPageGroupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPageGroupStatus::Reopened => {
                match next {
                    LenientPageGroupStatus::Archived => {
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

    pub fn move_to(&mut self, next: LenientPageGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient page group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.port as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientPageGroup {
        LenientPageGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            archived: self.archived,
            contact_email: self.contact_email.clone(),
            batch_size: self.batch_size,
            issuer: self.issuer.clone(),
            port: self.port,
            cost: self.cost,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient page group in status open.
pub fn new_lenient_page_group(code: &str, name: &str) -> LenientPageGroup {
    LenientPageGroup {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        archived: false,
        contact_email: "".to_string(),
        batch_size: 0,
        issuer: "".to_string(),
        port: 0,
        cost: 0.0,
        owner_id: "".to_string(),
        status: LenientPageGroupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_page_group("code-593", "name-902");
        assert_eq!(item.status, LenientPageGroupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_lenient_page_group("code-593", "name-902");
        if item.move_to(LenientPageGroupStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientPageGroupStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_page_group("code-593", "name-902");
        item.status = LenientPageGroupStatus::Archived;
        assert!(
            item.move_to(LenientPageGroupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
