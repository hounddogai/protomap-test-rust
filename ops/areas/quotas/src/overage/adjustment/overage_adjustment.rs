use super::*;

use acme_kit::text;

/// Where a overage adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl OverageAdjustmentStatus {
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

impl std::fmt::Display for OverageAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A overage adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageAdjustment {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub category: String,
    pub burst: i64,
    pub audience: String,
    pub key_id: String,
    pub active: bool,
    pub discount: f64,
    pub status: OverageAdjustmentStatus,
}

impl OverageAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageAdjustmentStatus) -> bool {
        match self.status {
            OverageAdjustmentStatus::New => {
                return next == OverageAdjustmentStatus::Running || next == OverageAdjustmentStatus::Failed;
            }
            OverageAdjustmentStatus::Queued => {
                match next {
                    OverageAdjustmentStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageAdjustmentStatus::Running => {
                match next {
                    OverageAdjustmentStatus::Done | OverageAdjustmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageAdjustmentStatus::Done => {
                return next == OverageAdjustmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: OverageAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage adjustment {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "overage adjustment", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        (self.discount * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageAdjustment {
        OverageAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            category: self.category.clone(),
            burst: self.burst,
            audience: self.audience.clone(),
            key_id: self.key_id.clone(),
            active: self.active,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a overage adjustment in status new.
pub fn new_overage_adjustment(code: &str, title: &str) -> OverageAdjustment {
    OverageAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        category: "".to_string(),
        burst: 0,
        audience: "".to_string(),
        key_id: "".to_string(),
        active: false,
        discount: 0.0,
        status: OverageAdjustmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_overage_adjustment("code-205", "title-995");
        assert_eq!(item.status, OverageAdjustmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_overage_adjustment("code-205", "title-995");
        if item.move_to(OverageAdjustmentStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageAdjustmentStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_overage_adjustment("code-205", "title-995");
        item.status = OverageAdjustmentStatus::Failed;
        assert!(
            item.move_to(OverageAdjustmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
