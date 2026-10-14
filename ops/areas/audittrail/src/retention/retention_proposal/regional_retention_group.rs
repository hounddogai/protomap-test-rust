use super::*;

use acme_kit::text;

/// Where a regional retention group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRetentionGroupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalRetentionGroupStatus {
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

impl std::fmt::Display for RegionalRetentionGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional retention group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRetentionGroup {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub created_at: i64,
    pub minimum: f64,
    pub channel: String,
    pub status: RegionalRetentionGroupStatus,
}

impl RegionalRetentionGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRetentionGroupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRetentionGroupStatus) -> bool {
        match self.status {
            RegionalRetentionGroupStatus::Open => {
                match next {
                    RegionalRetentionGroupStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRetentionGroupStatus::InReview => {
                match next {
                    RegionalRetentionGroupStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRetentionGroupStatus::Resolved => {
                return next == RegionalRetentionGroupStatus::Archived;
            }
            RegionalRetentionGroupStatus::Reopened => {
                match next {
                    RegionalRetentionGroupStatus::Archived => {
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

    pub fn move_to(&mut self, next: RegionalRetentionGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional retention group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional retention group", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.version as f64 - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalRetentionGroup {
        RegionalRetentionGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            created_at: self.created_at,
            minimum: self.minimum,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a regional retention group in status open.
pub fn new_regional_retention_group(code: &str, label: &str) -> RegionalRetentionGroup {
    RegionalRetentionGroup {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        created_at: 0,
        minimum: 0.0,
        channel: "".to_string(),
        status: RegionalRetentionGroupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_regional_retention_group("code-498", "label-197");
        assert_eq!(item.status, RegionalRetentionGroupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_regional_retention_group("code-498", "label-197");
        if item.move_to(RegionalRetentionGroupStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalRetentionGroupStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_regional_retention_group("code-498", "label-197");
        item.status = RegionalRetentionGroupStatus::Archived;
        assert!(
            item.move_to(RegionalRetentionGroupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
