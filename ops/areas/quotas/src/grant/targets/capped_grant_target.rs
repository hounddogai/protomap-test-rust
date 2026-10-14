use super::*;

use acme_kit::text;

/// Where a capped grant target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedGrantTargetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CappedGrantTargetStatus {
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

impl std::fmt::Display for CappedGrantTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped grant target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedGrantTarget {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub maximum: f64,
    pub channel: String,
    pub currency: String,
    pub ratio: f64,
    pub priority: i64,
    pub status: CappedGrantTargetStatus,
}

impl CappedGrantTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedGrantTargetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedGrantTargetStatus) -> bool {
        match self.status {
            CappedGrantTargetStatus::New => {
                return next == CappedGrantTargetStatus::Running || next == CappedGrantTargetStatus::Failed;
            }
            CappedGrantTargetStatus::Queued => {
                match next {
                    CappedGrantTargetStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedGrantTargetStatus::Running => {
                match next {
                    CappedGrantTargetStatus::Done | CappedGrantTargetStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedGrantTargetStatus::Done => {
                match next {
                    CappedGrantTargetStatus::Failed => {
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

    pub fn move_to(&mut self, next: CappedGrantTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped grant target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.priority as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedGrantTarget {
        CappedGrantTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            maximum: self.maximum,
            channel: self.channel.clone(),
            currency: self.currency.clone(),
            ratio: self.ratio,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a capped grant target in status new.
pub fn new_capped_grant_target(code: &str, name: &str) -> CappedGrantTarget {
    CappedGrantTarget {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        maximum: 0.0,
        channel: "".to_string(),
        currency: "".to_string(),
        ratio: 0.0,
        priority: 0,
        status: CappedGrantTargetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_capped_grant_target("code-994", "name-258");
        assert_eq!(item.status, CappedGrantTargetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_capped_grant_target("code-994", "name-258");
        if item.move_to(CappedGrantTargetStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedGrantTargetStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_capped_grant_target("code-994", "name-258");
        item.status = CappedGrantTargetStatus::Failed;
        assert!(
            item.move_to(CappedGrantTargetStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
