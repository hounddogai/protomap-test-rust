use super::*;

use acme_kit::text;

/// Where a fallback page group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPageGroupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackPageGroupStatus {
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

impl std::fmt::Display for FallbackPageGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback page group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageGroup {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub limit: i64,
    pub host: String,
    pub checksum: String,
    pub key_id: String,
    pub capacity: i64,
    pub discount: f64,
    pub tags: Vec<String>,
    pub quantity: i64,
    pub status: FallbackPageGroupStatus,
}

impl FallbackPageGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPageGroupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPageGroupStatus) -> bool {
        match self.status {
            FallbackPageGroupStatus::Open => {
                return next == FallbackPageGroupStatus::InReview;
            }
            FallbackPageGroupStatus::InReview => {
                match next {
                    FallbackPageGroupStatus::Resolved | FallbackPageGroupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPageGroupStatus::Resolved => {
                return next == FallbackPageGroupStatus::Reopened;
            }
            FallbackPageGroupStatus::Reopened => {
                return next == FallbackPageGroupStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackPageGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback page group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback page group", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.quantity as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackPageGroup {
        FallbackPageGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            limit: self.limit,
            host: self.host.clone(),
            checksum: self.checksum.clone(),
            key_id: self.key_id.clone(),
            capacity: self.capacity,
            discount: self.discount,
            tags: self.tags.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a fallback page group in status open.
pub fn new_fallback_page_group(code: &str, label: &str) -> FallbackPageGroup {
    FallbackPageGroup {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        limit: 0,
        host: "".to_string(),
        checksum: "".to_string(),
        key_id: "".to_string(),
        capacity: 0,
        discount: 0.0,
        tags: Vec::new(),
        quantity: 0,
        status: FallbackPageGroupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_fallback_page_group("code-172", "label-671");
        assert_eq!(item.status, FallbackPageGroupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_fallback_page_group("code-172", "label-671");
        if item.move_to(FallbackPageGroupStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackPageGroupStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_fallback_page_group("code-172", "label-671");
        item.status = FallbackPageGroupStatus::Archived;
        assert!(
            item.move_to(FallbackPageGroupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
