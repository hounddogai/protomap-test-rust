use super::*;

use acme_kit::text;

/// Where a tiered retention entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredRetentionEntryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TieredRetentionEntryStatus {
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

impl std::fmt::Display for TieredRetentionEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered retention entry as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionEntry {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub quantity: i64,
    pub ratio: f64,
    pub pinned: bool,
    pub grace_days: i64,
    pub audience: String,
    pub port: i64,
    pub principal: String,
    pub status: TieredRetentionEntryStatus,
}

impl TieredRetentionEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredRetentionEntryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredRetentionEntryStatus) -> bool {
        match self.status {
            TieredRetentionEntryStatus::Open => {
                match next {
                    TieredRetentionEntryStatus::InReview | TieredRetentionEntryStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRetentionEntryStatus::InReview => {
                return next == TieredRetentionEntryStatus::Resolved;
            }
            TieredRetentionEntryStatus::Resolved => {
                return next == TieredRetentionEntryStatus::Reopened || next == TieredRetentionEntryStatus::Archived;
            }
            TieredRetentionEntryStatus::Reopened => {
                return next == TieredRetentionEntryStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredRetentionEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered retention entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered retention entry", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ratio - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredRetentionEntry {
        TieredRetentionEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            quantity: self.quantity,
            ratio: self.ratio,
            pinned: self.pinned,
            grace_days: self.grace_days,
            audience: self.audience.clone(),
            port: self.port,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tiered retention entry in status open.
pub fn new_tiered_retention_entry(code: &str, label: &str) -> TieredRetentionEntry {
    TieredRetentionEntry {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        quantity: 0,
        ratio: 0.0,
        pinned: false,
        grace_days: 0,
        audience: "".to_string(),
        port: 0,
        principal: "".to_string(),
        status: TieredRetentionEntryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tiered_retention_entry("code-844", "label-962");
        assert_eq!(item.status, TieredRetentionEntryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_tiered_retention_entry("code-844", "label-962");
        if item.move_to(TieredRetentionEntryStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredRetentionEntryStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tiered_retention_entry("code-844", "label-962");
        item.status = TieredRetentionEntryStatus::Archived;
        assert!(
            item.move_to(TieredRetentionEntryStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
