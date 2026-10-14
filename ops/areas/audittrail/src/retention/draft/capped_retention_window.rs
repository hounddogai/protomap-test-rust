use super::*;

use acme_kit::text;

/// Where a capped retention window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedRetentionWindowStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CappedRetentionWindowStatus {
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

impl std::fmt::Display for CappedRetentionWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped retention window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRetentionWindow {
    pub code: String,
    pub label: String,
    pub name: String,
    pub issuer: String,
    pub maximum: f64,
    pub window_days: i64,
    pub quantity: i64,
    pub starts_at: i64,
    pub rotation_days: i64,
    pub status: CappedRetentionWindowStatus,
}

impl CappedRetentionWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedRetentionWindowStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedRetentionWindowStatus) -> bool {
        match self.status {
            CappedRetentionWindowStatus::New => {
                match next {
                    CappedRetentionWindowStatus::Queued | CappedRetentionWindowStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRetentionWindowStatus::Queued => {
                match next {
                    CappedRetentionWindowStatus::Running | CappedRetentionWindowStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRetentionWindowStatus::Running => {
                return next == CappedRetentionWindowStatus::Failed;
            }
            CappedRetentionWindowStatus::Done => {
                return next == CappedRetentionWindowStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedRetentionWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped retention window {} cannot move from {} to {}",
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
        format!("{} {}", "capped retention window", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.maximum + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedRetentionWindow {
        CappedRetentionWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            issuer: self.issuer.clone(),
            maximum: self.maximum,
            window_days: self.window_days,
            quantity: self.quantity,
            starts_at: self.starts_at,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a capped retention window in status new.
pub fn new_capped_retention_window(code: &str, label: &str) -> CappedRetentionWindow {
    CappedRetentionWindow {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        issuer: "".to_string(),
        maximum: 0.0,
        window_days: 0,
        quantity: 0,
        starts_at: 0,
        rotation_days: 0,
        status: CappedRetentionWindowStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_capped_retention_window("code-253", "label-669");
        assert_eq!(item.status, CappedRetentionWindowStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_capped_retention_window("code-253", "label-669");
        if item.move_to(CappedRetentionWindowStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedRetentionWindowStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_capped_retention_window("code-253", "label-669");
        item.status = CappedRetentionWindowStatus::Failed;
        assert!(
            item.move_to(CappedRetentionWindowStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
