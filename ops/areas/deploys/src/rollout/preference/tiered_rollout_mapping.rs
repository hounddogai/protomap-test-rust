use super::*;

use acme_kit::text;

/// Where a tiered rollout mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredRolloutMappingStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TieredRolloutMappingStatus {
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

impl std::fmt::Display for TieredRolloutMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered rollout mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutMapping {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub scope: String,
    pub duration_minutes: i64,
    pub ceiling: f64,
    pub status: TieredRolloutMappingStatus,
}

impl TieredRolloutMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredRolloutMappingStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredRolloutMappingStatus) -> bool {
        match self.status {
            TieredRolloutMappingStatus::New => {
                match next {
                    TieredRolloutMappingStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRolloutMappingStatus::Queued => {
                match next {
                    TieredRolloutMappingStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRolloutMappingStatus::Running => {
                match next {
                    TieredRolloutMappingStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredRolloutMappingStatus::Done => {
                return next == TieredRolloutMappingStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredRolloutMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered rollout mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredRolloutMapping {
        TieredRolloutMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            scope: self.scope.clone(),
            duration_minutes: self.duration_minutes,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a tiered rollout mapping in status new.
pub fn new_tiered_rollout_mapping(code: &str, label: &str) -> TieredRolloutMapping {
    TieredRolloutMapping {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        scope: "".to_string(),
        duration_minutes: 0,
        ceiling: 0.0,
        status: TieredRolloutMappingStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_tiered_rollout_mapping("code-734", "label-881");
        assert_eq!(item.status, TieredRolloutMappingStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_tiered_rollout_mapping("code-734", "label-881");
        if item.move_to(TieredRolloutMappingStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredRolloutMappingStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_tiered_rollout_mapping("code-734", "label-881");
        item.status = TieredRolloutMappingStatus::Failed;
        assert!(
            item.move_to(TieredRolloutMappingStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
