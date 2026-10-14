use super::*;

use acme_kit::text;

/// Where a tiered overage threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredOverageThresholdStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TieredOverageThresholdStatus {
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

impl std::fmt::Display for TieredOverageThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered overage threshold as the overage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageThreshold {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub rate: f64,
    pub archived: bool,
    pub status: TieredOverageThresholdStatus,
}

impl TieredOverageThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredOverageThresholdStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredOverageThresholdStatus) -> bool {
        match self.status {
            TieredOverageThresholdStatus::New => {
                match next {
                    TieredOverageThresholdStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredOverageThresholdStatus::Queued => {
                match next {
                    TieredOverageThresholdStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredOverageThresholdStatus::Running => {
                match next {
                    TieredOverageThresholdStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredOverageThresholdStatus::Done => {
                match next {
                    TieredOverageThresholdStatus::Failed => {
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

    pub fn move_to(&mut self, next: TieredOverageThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered overage threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        (self.rate * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredOverageThreshold {
        TieredOverageThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            rate: self.rate,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered overage threshold in status new.
pub fn new_tiered_overage_threshold(code: &str, name: &str) -> TieredOverageThreshold {
    TieredOverageThreshold {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        rate: 0.0,
        archived: false,
        status: TieredOverageThresholdStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_tiered_overage_threshold("code-46", "name-195");
        assert_eq!(item.status, TieredOverageThresholdStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_tiered_overage_threshold("code-46", "name-195");
        if item.move_to(TieredOverageThresholdStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredOverageThresholdStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_tiered_overage_threshold("code-46", "name-195");
        item.status = TieredOverageThresholdStatus::Failed;
        assert!(
            item.move_to(TieredOverageThresholdStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
