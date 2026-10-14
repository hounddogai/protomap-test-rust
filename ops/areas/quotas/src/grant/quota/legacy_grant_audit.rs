use super::*;

use acme_kit::text;

/// Where a legacy grant audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyGrantAuditStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyGrantAuditStatus {
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

impl std::fmt::Display for LegacyGrantAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy grant audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantAudit {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub cluster: String,
    pub label: String,
    pub title: String,
    pub unit: String,
    pub cost: f64,
    pub starts_at: i64,
    pub status: LegacyGrantAuditStatus,
}

impl LegacyGrantAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyGrantAuditStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyGrantAuditStatus) -> bool {
        match self.status {
            LegacyGrantAuditStatus::New => {
                return next == LegacyGrantAuditStatus::Queued || next == LegacyGrantAuditStatus::Done;
            }
            LegacyGrantAuditStatus::Queued => {
                match next {
                    LegacyGrantAuditStatus::Running | LegacyGrantAuditStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyGrantAuditStatus::Running => {
                match next {
                    LegacyGrantAuditStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyGrantAuditStatus::Done => {
                return next == LegacyGrantAuditStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyGrantAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy grant audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        (self.cost * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyGrantAudit {
        LegacyGrantAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            cluster: self.cluster.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            unit: self.unit.clone(),
            cost: self.cost,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a legacy grant audit in status new.
pub fn new_legacy_grant_audit(code: &str, name: &str) -> LegacyGrantAudit {
    LegacyGrantAudit {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        cluster: "".to_string(),
        label: "".to_string(),
        title: "".to_string(),
        unit: "".to_string(),
        cost: 0.0,
        starts_at: 0,
        status: LegacyGrantAuditStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_legacy_grant_audit("code-364", "name-404");
        assert_eq!(item.status, LegacyGrantAuditStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_legacy_grant_audit("code-364", "name-404");
        if item.move_to(LegacyGrantAuditStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyGrantAuditStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_legacy_grant_audit("code-364", "name-404");
        item.status = LegacyGrantAuditStatus::Failed;
        assert!(
            item.move_to(LegacyGrantAuditStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
