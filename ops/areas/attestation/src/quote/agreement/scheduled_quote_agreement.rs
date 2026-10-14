use super::*;

use acme_kit::text;

/// Where a scheduled quote agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledQuoteAgreementStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ScheduledQuoteAgreementStatus {
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

impl std::fmt::Display for ScheduledQuoteAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled quote agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuoteAgreement {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub rate: f64,
    pub visible: bool,
    pub tags: Vec<String>,
    pub status: ScheduledQuoteAgreementStatus,
}

impl ScheduledQuoteAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledQuoteAgreementStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledQuoteAgreementStatus) -> bool {
        match self.status {
            ScheduledQuoteAgreementStatus::Draft => {
                return next == ScheduledQuoteAgreementStatus::Closed;
            }
            ScheduledQuoteAgreementStatus::Pending => {
                match next {
                    ScheduledQuoteAgreementStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledQuoteAgreementStatus::Active => {
                return next == ScheduledQuoteAgreementStatus::Suspended;
            }
            ScheduledQuoteAgreementStatus::Suspended => {
                return next == ScheduledQuoteAgreementStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledQuoteAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled quote agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_load(&self) -> f64 {
        ((self.starts_at as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledQuoteAgreement {
        ScheduledQuoteAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            rate: self.rate,
            visible: self.visible,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a scheduled quote agreement in status draft.
pub fn new_scheduled_quote_agreement(code: &str, title: &str) -> ScheduledQuoteAgreement {
    ScheduledQuoteAgreement {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        rate: 0.0,
        visible: false,
        tags: Vec::new(),
        status: ScheduledQuoteAgreementStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_scheduled_quote_agreement("code-535", "title-524");
        assert_eq!(item.status, ScheduledQuoteAgreementStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_scheduled_quote_agreement("code-535", "title-524");
        if item.move_to(ScheduledQuoteAgreementStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledQuoteAgreementStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_scheduled_quote_agreement("code-535", "title-524");
        item.status = ScheduledQuoteAgreementStatus::Closed;
        assert!(
            item.move_to(ScheduledQuoteAgreementStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
