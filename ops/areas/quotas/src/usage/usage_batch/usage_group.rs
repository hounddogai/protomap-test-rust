use super::*;

use acme_kit::text;

/// Where a usage group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageGroupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl UsageGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for UsageGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A usage group. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageGroup {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub revision: i64,
    pub tags: Vec<String>,
    pub status: UsageGroupStatus,
}

impl UsageGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageGroupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageGroupStatus) -> bool {
        match self.status {
            UsageGroupStatus::Requested => {
                return next == UsageGroupStatus::Revoked || next == UsageGroupStatus::Expired;
            }
            UsageGroupStatus::Granted => {
                match next {
                    UsageGroupStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageGroupStatus::Revoked => {
                return next == UsageGroupStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_load(&self) -> f64 {
        ((self.sequence as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageGroup {
        UsageGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            revision: self.revision,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a usage group in status requested.
pub fn new_usage_group(code: &str, title: &str) -> UsageGroup {
    UsageGroup {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        revision: 0,
        tags: Vec::new(),
        status: UsageGroupStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_usage_group("code-980", "title-227");
        assert_eq!(item.status, UsageGroupStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_usage_group("code-980", "title-227");
        if item.move_to(UsageGroupStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageGroupStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_usage_group("code-980", "title-227");
        item.status = UsageGroupStatus::Expired;
        assert!(
            item.move_to(UsageGroupStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
