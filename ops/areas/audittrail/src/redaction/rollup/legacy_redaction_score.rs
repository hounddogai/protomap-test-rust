use super::*;

use acme_kit::text;

/// Where a legacy redaction score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRedactionScoreStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyRedactionScoreStatus {
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

impl std::fmt::Display for LegacyRedactionScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy redaction score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRedactionScore {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub threshold: f64,
    pub summary: String,
    pub duration_minutes: i64,
    pub minimum: f64,
    pub status: LegacyRedactionScoreStatus,
}

impl LegacyRedactionScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRedactionScoreStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRedactionScoreStatus) -> bool {
        match self.status {
            LegacyRedactionScoreStatus::Open => {
                match next {
                    LegacyRedactionScoreStatus::InReview | LegacyRedactionScoreStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRedactionScoreStatus::InReview => {
                match next {
                    LegacyRedactionScoreStatus::Reopened | LegacyRedactionScoreStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRedactionScoreStatus::Resolved => {
                match next {
                    LegacyRedactionScoreStatus::Reopened | LegacyRedactionScoreStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRedactionScoreStatus::Reopened => {
                return next == LegacyRedactionScoreStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyRedactionScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy redaction score {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.duration_minutes as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyRedactionScore {
        LegacyRedactionScore {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            threshold: self.threshold,
            summary: self.summary.clone(),
            duration_minutes: self.duration_minutes,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a legacy redaction score in status open.
pub fn new_legacy_redaction_score(code: &str, label: &str) -> LegacyRedactionScore {
    LegacyRedactionScore {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        threshold: 0.0,
        summary: "".to_string(),
        duration_minutes: 0,
        minimum: 0.0,
        status: LegacyRedactionScoreStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_legacy_redaction_score("code-893", "label-740");
        assert_eq!(item.status, LegacyRedactionScoreStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_legacy_redaction_score("code-893", "label-740");
        if item.move_to(LegacyRedactionScoreStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyRedactionScoreStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_legacy_redaction_score("code-893", "label-740");
        item.status = LegacyRedactionScoreStatus::Archived;
        assert!(
            item.move_to(LegacyRedactionScoreStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
