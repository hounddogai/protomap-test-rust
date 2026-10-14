use super::*;

use acme_kit::text;

/// Where a scheduled quote group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledQuoteGroupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledQuoteGroupStatus {
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

impl std::fmt::Display for ScheduledQuoteGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled quote group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuoteGroup {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub owner_id: String,
    pub score: f64,
    pub status: ScheduledQuoteGroupStatus,
}

impl ScheduledQuoteGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledQuoteGroupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledQuoteGroupStatus) -> bool {
        match self.status {
            ScheduledQuoteGroupStatus::Open => {
                match next {
                    ScheduledQuoteGroupStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledQuoteGroupStatus::InReview => {
                match next {
                    ScheduledQuoteGroupStatus::Resolved | ScheduledQuoteGroupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledQuoteGroupStatus::Resolved => {
                return next == ScheduledQuoteGroupStatus::Reopened;
            }
            ScheduledQuoteGroupStatus::Reopened => {
                return next == ScheduledQuoteGroupStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledQuoteGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled quote group {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.score + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledQuoteGroup {
        ScheduledQuoteGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            owner_id: self.owner_id.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled quote group in status open.
pub fn new_scheduled_quote_group(code: &str, label: &str) -> ScheduledQuoteGroup {
    ScheduledQuoteGroup {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        owner_id: "".to_string(),
        score: 0.0,
        status: ScheduledQuoteGroupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_scheduled_quote_group("code-362", "label-321");
        assert_eq!(item.status, ScheduledQuoteGroupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_scheduled_quote_group("code-362", "label-321");
        if item.move_to(ScheduledQuoteGroupStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledQuoteGroupStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_scheduled_quote_group("code-362", "label-321");
        item.status = ScheduledQuoteGroupStatus::Archived;
        assert!(
            item.move_to(ScheduledQuoteGroupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
