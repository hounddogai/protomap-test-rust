use super::*;

use acme_kit::text;

/// Where a failover rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverRollupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FailoverRollupStatus {
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

impl std::fmt::Display for FailoverRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a failover rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRollup {
    pub code: String,
    pub label: String,
    pub title: String,
    pub tags: Vec<String>,
    pub approved: bool,
    pub quota_units: i64,
    pub status: FailoverRollupStatus,
}

impl FailoverRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverRollupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverRollupStatus) -> bool {
        match self.status {
            FailoverRollupStatus::Draft => {
                return next == FailoverRollupStatus::Suspended || next == FailoverRollupStatus::Closed;
            }
            FailoverRollupStatus::Pending => {
                return next == FailoverRollupStatus::Suspended;
            }
            FailoverRollupStatus::Active => {
                match next {
                    FailoverRollupStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverRollupStatus::Suspended => {
                match next {
                    FailoverRollupStatus::Closed => {
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

    pub fn move_to(&mut self, next: FailoverRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.quota_units as f64
    }

    pub fn copy(&self) -> FailoverRollup {
        FailoverRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            approved: self.approved,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a failover rollup in status draft.
pub fn new_failover_rollup(code: &str, label: &str) -> FailoverRollup {
    FailoverRollup {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        tags: Vec::new(),
        approved: false,
        quota_units: 0,
        status: FailoverRollupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_failover_rollup("code-57", "label-103");
        assert_eq!(item.status, FailoverRollupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_failover_rollup("code-57", "label-103");
        if item.move_to(FailoverRollupStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverRollupStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_failover_rollup("code-57", "label-103");
        item.status = FailoverRollupStatus::Closed;
        assert!(
            item.move_to(FailoverRollupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
