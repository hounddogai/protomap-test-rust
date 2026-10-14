use super::*;

use acme_kit::text;

/// Where a release rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseRuleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ReleaseRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ReleaseRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release rule as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseRule {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub archived: bool,
    pub grace_days: i64,
    pub starts_at: i64,
    pub reviewer: String,
    pub tenant_id: String,
    pub revision: i64,
    pub status: ReleaseRuleStatus,
}

impl ReleaseRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseRuleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseRuleStatus) -> bool {
        match self.status {
            ReleaseRuleStatus::New => {
                return next == ReleaseRuleStatus::Running;
            }
            ReleaseRuleStatus::Queued => {
                return next == ReleaseRuleStatus::Failed;
            }
            ReleaseRuleStatus::Running => {
                match next {
                    ReleaseRuleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseRuleStatus::Done => {
                match next {
                    ReleaseRuleStatus::Failed => {
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

    pub fn move_to(&mut self, next: ReleaseRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.revision as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseRule {
        ReleaseRule {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            archived: self.archived,
            grace_days: self.grace_days,
            starts_at: self.starts_at,
            reviewer: self.reviewer.clone(),
            tenant_id: self.tenant_id.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a release rule in status new.
pub fn new_release_rule(code: &str, name: &str) -> ReleaseRule {
    ReleaseRule {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        archived: false,
        grace_days: 0,
        starts_at: 0,
        reviewer: "".to_string(),
        tenant_id: "".to_string(),
        revision: 0,
        status: ReleaseRuleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_release_rule("code-619", "name-975");
        assert_eq!(item.status, ReleaseRuleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_release_rule("code-619", "name-975");
        if item.move_to(ReleaseRuleStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseRuleStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_release_rule("code-619", "name-975");
        item.status = ReleaseRuleStatus::Failed;
        assert!(item.move_to(ReleaseRuleStatus::New).is_err(), "expected move to fail");
    }
}
