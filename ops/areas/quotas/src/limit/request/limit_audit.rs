use super::*;

use acme_kit::text;

/// Where a limit audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitAuditStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LimitAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for LimitAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One limit audit, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitAudit {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub score: f64,
    pub archived: bool,
    pub audience: String,
    pub grace_days: i64,
    pub sequence: i64,
    pub cipher: String,
    pub status: LimitAuditStatus,
}

impl LimitAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitAuditStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitAuditStatus) -> bool {
        match self.status {
            LimitAuditStatus::Draft => {
                match next {
                    LimitAuditStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitAuditStatus::Pending => {
                match next {
                    LimitAuditStatus::Active | LimitAuditStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitAuditStatus::Active => {
                return next == LimitAuditStatus::Suspended || next == LimitAuditStatus::Closed;
            }
            LimitAuditStatus::Suspended => {
                match next {
                    LimitAuditStatus::Closed => {
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

    pub fn move_to(&mut self, next: LimitAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.sequence as f64 + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitAudit {
        LimitAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            score: self.score,
            archived: self.archived,
            audience: self.audience.clone(),
            grace_days: self.grace_days,
            sequence: self.sequence,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a limit audit in status draft.
pub fn new_limit_audit(code: &str, title: &str) -> LimitAudit {
    LimitAudit {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        score: 0.0,
        archived: false,
        audience: "".to_string(),
        grace_days: 0,
        sequence: 0,
        cipher: "".to_string(),
        status: LimitAuditStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_limit_audit("code-334", "title-470");
        assert_eq!(item.status, LimitAuditStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_limit_audit("code-334", "title-470");
        if item.move_to(LimitAuditStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitAuditStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_limit_audit("code-334", "title-470");
        item.status = LimitAuditStatus::Closed;
        assert!(item.move_to(LimitAuditStatus::Draft).is_err(), "expected move to fail");
    }
}
