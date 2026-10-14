use super::*;

use acme_kit::text;

/// Where a capped query rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedQueryRollupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CappedQueryRollupStatus {
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

impl std::fmt::Display for CappedQueryRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped query rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryRollup {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub limit: i64,
    pub rotation_days: i64,
    pub host: String,
    pub floor_value: f64,
    pub status: CappedQueryRollupStatus,
}

impl CappedQueryRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedQueryRollupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedQueryRollupStatus) -> bool {
        match self.status {
            CappedQueryRollupStatus::New => {
                return next == CappedQueryRollupStatus::Running;
            }
            CappedQueryRollupStatus::Queued => {
                match next {
                    CappedQueryRollupStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedQueryRollupStatus::Running => {
                return next == CappedQueryRollupStatus::Done;
            }
            CappedQueryRollupStatus::Done => {
                match next {
                    CappedQueryRollupStatus::Failed => {
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

    pub fn move_to(&mut self, next: CappedQueryRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped query rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "capped query rollup", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.rotation_days as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedQueryRollup {
        CappedQueryRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            limit: self.limit,
            rotation_days: self.rotation_days,
            host: self.host.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a capped query rollup in status new.
pub fn new_capped_query_rollup(code: &str, label: &str) -> CappedQueryRollup {
    CappedQueryRollup {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        limit: 0,
        rotation_days: 0,
        host: "".to_string(),
        floor_value: 0.0,
        status: CappedQueryRollupStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_capped_query_rollup("code-270", "label-384");
        assert_eq!(item.status, CappedQueryRollupStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_capped_query_rollup("code-270", "label-384");
        if item.move_to(CappedQueryRollupStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedQueryRollupStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_capped_query_rollup("code-270", "label-384");
        item.status = CappedQueryRollupStatus::Failed;
        assert!(
            item.move_to(CappedQueryRollupStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
