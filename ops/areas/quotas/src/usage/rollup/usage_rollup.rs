use super::*;

use acme_kit::text;

/// Where a usage rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageRollupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl UsageRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for UsageRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a usage rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageRollup {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub namespace: String,
    pub starts_at: i64,
    pub external_ref: String,
    pub label: String,
    pub status: UsageRollupStatus,
}

impl UsageRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageRollupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageRollupStatus) -> bool {
        match self.status {
            UsageRollupStatus::Proposed => {
                match next {
                    UsageRollupStatus::Rejected | UsageRollupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageRollupStatus::Approved => {
                match next {
                    UsageRollupStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageRollupStatus::Rejected => {
                match next {
                    UsageRollupStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: UsageRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.updated_at as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageRollup {
        UsageRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            namespace: self.namespace.clone(),
            starts_at: self.starts_at,
            external_ref: self.external_ref.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a usage rollup in status proposed.
pub fn new_usage_rollup(code: &str, title: &str) -> UsageRollup {
    UsageRollup {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        namespace: "".to_string(),
        starts_at: 0,
        external_ref: "".to_string(),
        label: "".to_string(),
        status: UsageRollupStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_usage_rollup("code-911", "title-586");
        assert_eq!(item.status, UsageRollupStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_usage_rollup("code-911", "title-586");
        if item.move_to(UsageRollupStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageRollupStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_usage_rollup("code-911", "title-586");
        item.status = UsageRollupStatus::Withdrawn;
        assert!(
            item.move_to(UsageRollupStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
