use super::*;

use acme_kit::text;

/// Where a overage rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageRuleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl OverageRuleStatus {
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

impl std::fmt::Display for OverageRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a overage rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageRule {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub locale: String,
    pub created_at: i64,
    pub discount: f64,
    pub cluster: String,
    pub timezone: String,
    pub capacity: i64,
    pub status: OverageRuleStatus,
}

impl OverageRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageRuleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageRuleStatus) -> bool {
        match self.status {
            OverageRuleStatus::New => {
                return next == OverageRuleStatus::Done;
            }
            OverageRuleStatus::Queued => {
                match next {
                    OverageRuleStatus::Running | OverageRuleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageRuleStatus::Running => {
                return next == OverageRuleStatus::Done;
            }
            OverageRuleStatus::Done => {
                match next {
                    OverageRuleStatus::Failed => {
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

    pub fn move_to(&mut self, next: OverageRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage rule {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.version as f64) * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageRule {
        OverageRule {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            locale: self.locale.clone(),
            created_at: self.created_at,
            discount: self.discount,
            cluster: self.cluster.clone(),
            timezone: self.timezone.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a overage rule in status new.
pub fn new_overage_rule(code: &str, label: &str) -> OverageRule {
    OverageRule {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        locale: "".to_string(),
        created_at: 0,
        discount: 0.0,
        cluster: "".to_string(),
        timezone: "".to_string(),
        capacity: 0,
        status: OverageRuleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_overage_rule("code-734", "label-798");
        assert_eq!(item.status, OverageRuleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_overage_rule("code-734", "label-798");
        if item.move_to(OverageRuleStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageRuleStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_overage_rule("code-734", "label-798");
        item.status = OverageRuleStatus::Failed;
        assert!(item.move_to(OverageRuleStatus::New).is_err(), "expected move to fail");
    }
}
