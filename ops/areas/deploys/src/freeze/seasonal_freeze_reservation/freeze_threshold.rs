use super::*;

use acme_kit::text;

/// Where a freeze threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeThresholdStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FreezeThresholdStatus {
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

impl std::fmt::Display for FreezeThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeThreshold {
    pub code: String,
    pub label: String,
    pub title: String,
    pub currency: String,
    pub tags: Vec<String>,
    pub cipher: String,
    pub budget: f64,
    pub ttl_seconds: i64,
    pub retries: i64,
    pub pinned: bool,
    pub sequence: i64,
    pub status: FreezeThresholdStatus,
}

impl FreezeThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeThresholdStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeThresholdStatus) -> bool {
        match self.status {
            FreezeThresholdStatus::Open => {
                match next {
                    FreezeThresholdStatus::Resolved | FreezeThresholdStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeThresholdStatus::InReview => {
                return next == FreezeThresholdStatus::Resolved || next == FreezeThresholdStatus::Reopened;
            }
            FreezeThresholdStatus::Resolved => {
                match next {
                    FreezeThresholdStatus::Reopened | FreezeThresholdStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeThresholdStatus::Reopened => {
                match next {
                    FreezeThresholdStatus::Archived => {
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

    pub fn move_to(&mut self, next: FreezeThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "freeze threshold", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_total(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeThreshold {
        FreezeThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            tags: self.tags.clone(),
            cipher: self.cipher.clone(),
            budget: self.budget,
            ttl_seconds: self.ttl_seconds,
            retries: self.retries,
            pinned: self.pinned,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a freeze threshold in status open.
pub fn new_freeze_threshold(code: &str, label: &str) -> FreezeThreshold {
    FreezeThreshold {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        currency: "".to_string(),
        tags: Vec::new(),
        cipher: "".to_string(),
        budget: 0.0,
        ttl_seconds: 0,
        retries: 0,
        pinned: false,
        sequence: 0,
        status: FreezeThresholdStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_freeze_threshold("code-507", "label-324");
        assert_eq!(item.status, FreezeThresholdStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_freeze_threshold("code-507", "label-324");
        if item.move_to(FreezeThresholdStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeThresholdStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_freeze_threshold("code-507", "label-324");
        item.status = FreezeThresholdStatus::Archived;
        assert!(
            item.move_to(FreezeThresholdStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
