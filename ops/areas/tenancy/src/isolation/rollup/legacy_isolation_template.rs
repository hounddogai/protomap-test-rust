use super::*;

use acme_kit::text;

/// Where a legacy isolation template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyIsolationTemplateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyIsolationTemplateStatus {
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

impl std::fmt::Display for LegacyIsolationTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy isolation template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationTemplate {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub floor_value: f64,
    pub summary: String,
    pub key_id: String,
    pub ceiling: f64,
    pub currency: String,
    pub attempts: i64,
    pub status: LegacyIsolationTemplateStatus,
}

impl LegacyIsolationTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyIsolationTemplateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyIsolationTemplateStatus) -> bool {
        match self.status {
            LegacyIsolationTemplateStatus::Open => {
                return next == LegacyIsolationTemplateStatus::Resolved
                    || next == LegacyIsolationTemplateStatus::Reopened;
            }
            LegacyIsolationTemplateStatus::InReview => {
                return next == LegacyIsolationTemplateStatus::Archived;
            }
            LegacyIsolationTemplateStatus::Resolved => {
                return next == LegacyIsolationTemplateStatus::Reopened
                    || next == LegacyIsolationTemplateStatus::Archived;
            }
            LegacyIsolationTemplateStatus::Reopened => {
                match next {
                    LegacyIsolationTemplateStatus::Archived => {
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

    pub fn move_to(&mut self, next: LegacyIsolationTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy isolation template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.ceiling - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyIsolationTemplate {
        LegacyIsolationTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            floor_value: self.floor_value,
            summary: self.summary.clone(),
            key_id: self.key_id.clone(),
            ceiling: self.ceiling,
            currency: self.currency.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy isolation template in status open.
pub fn new_legacy_isolation_template(code: &str, title: &str) -> LegacyIsolationTemplate {
    LegacyIsolationTemplate {
        code: code.to_string(),
        title: title.to_string(),
        contact_email: "".to_string(),
        floor_value: 0.0,
        summary: "".to_string(),
        key_id: "".to_string(),
        ceiling: 0.0,
        currency: "".to_string(),
        attempts: 0,
        status: LegacyIsolationTemplateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_legacy_isolation_template("code-114", "title-834");
        assert_eq!(item.status, LegacyIsolationTemplateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_legacy_isolation_template("code-114", "title-834");
        if item.move_to(LegacyIsolationTemplateStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyIsolationTemplateStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_legacy_isolation_template("code-114", "title-834");
        item.status = LegacyIsolationTemplateStatus::Archived;
        assert!(
            item.move_to(LegacyIsolationTemplateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
