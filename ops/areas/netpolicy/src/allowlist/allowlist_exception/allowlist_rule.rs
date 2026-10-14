use super::*;

use acme_kit::text;

/// Where a allowlist rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistRuleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AllowlistRuleStatus {
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

impl std::fmt::Display for AllowlistRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One allowlist rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRule {
    pub code: String,
    pub title: String,
    pub visible: bool,
    pub namespace: String,
    pub ratio: f64,
    pub archived: bool,
    pub approved: bool,
    pub checksum: String,
    pub cipher: String,
    pub revision: i64,
    pub reviewer: String,
    pub status: AllowlistRuleStatus,
}

impl AllowlistRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistRuleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistRuleStatus) -> bool {
        match self.status {
            AllowlistRuleStatus::New => {
                return next == AllowlistRuleStatus::Queued;
            }
            AllowlistRuleStatus::Queued => {
                match next {
                    AllowlistRuleStatus::Running | AllowlistRuleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistRuleStatus::Running => {
                match next {
                    AllowlistRuleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistRuleStatus::Done => {
                return next == AllowlistRuleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "allowlist rule", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.revision as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistRule {
        AllowlistRule {
            code: self.code.clone(),
            title: self.title.clone(),
            visible: self.visible,
            namespace: self.namespace.clone(),
            ratio: self.ratio,
            archived: self.archived,
            approved: self.approved,
            checksum: self.checksum.clone(),
            cipher: self.cipher.clone(),
            revision: self.revision,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a allowlist rule in status new.
pub fn new_allowlist_rule(code: &str, title: &str) -> AllowlistRule {
    AllowlistRule {
        code: code.to_string(),
        title: title.to_string(),
        visible: false,
        namespace: "".to_string(),
        ratio: 0.0,
        archived: false,
        approved: false,
        checksum: "".to_string(),
        cipher: "".to_string(),
        revision: 0,
        reviewer: "".to_string(),
        status: AllowlistRuleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_allowlist_rule("code-610", "title-854");
        assert_eq!(item.status, AllowlistRuleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_allowlist_rule("code-610", "title-854");
        if item.move_to(AllowlistRuleStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistRuleStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_allowlist_rule("code-610", "title-854");
        item.status = AllowlistRuleStatus::Failed;
        assert!(item.move_to(AllowlistRuleStatus::New).is_err(), "expected move to fail");
    }
}
