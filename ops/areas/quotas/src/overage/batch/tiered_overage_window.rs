use super::*;

use acme_kit::text;

/// Where a tiered overage window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredOverageWindowStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TieredOverageWindowStatus {
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

impl std::fmt::Display for TieredOverageWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered overage window and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageWindow {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub rotation_days: i64,
    pub key_id: String,
    pub source: String,
    pub limit: i64,
    pub owner_id: String,
    pub checksum: String,
    pub status: TieredOverageWindowStatus,
}

impl TieredOverageWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredOverageWindowStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredOverageWindowStatus) -> bool {
        match self.status {
            TieredOverageWindowStatus::New => {
                match next {
                    TieredOverageWindowStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredOverageWindowStatus::Queued => {
                return next == TieredOverageWindowStatus::Running || next == TieredOverageWindowStatus::Failed;
            }
            TieredOverageWindowStatus::Running => {
                return next == TieredOverageWindowStatus::Done || next == TieredOverageWindowStatus::Failed;
            }
            TieredOverageWindowStatus::Done => {
                match next {
                    TieredOverageWindowStatus::Failed => {
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

    pub fn move_to(&mut self, next: TieredOverageWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered overage window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.rotation_days as f64) * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredOverageWindow {
        TieredOverageWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            rotation_days: self.rotation_days,
            key_id: self.key_id.clone(),
            source: self.source.clone(),
            limit: self.limit,
            owner_id: self.owner_id.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a tiered overage window in status new.
pub fn new_tiered_overage_window(code: &str, title: &str) -> TieredOverageWindow {
    TieredOverageWindow {
        code: code.to_string(),
        title: title.to_string(),
        tenant_id: "".to_string(),
        rotation_days: 0,
        key_id: "".to_string(),
        source: "".to_string(),
        limit: 0,
        owner_id: "".to_string(),
        checksum: "".to_string(),
        status: TieredOverageWindowStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_tiered_overage_window("code-657", "title-822");
        assert_eq!(item.status, TieredOverageWindowStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_tiered_overage_window("code-657", "title-822");
        if item.move_to(TieredOverageWindowStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredOverageWindowStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_tiered_overage_window("code-657", "title-822");
        item.status = TieredOverageWindowStatus::Failed;
        assert!(
            item.move_to(TieredOverageWindowStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
