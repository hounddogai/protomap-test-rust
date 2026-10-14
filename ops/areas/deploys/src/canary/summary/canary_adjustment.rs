use super::*;

use acme_kit::text;

/// Where a canary adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryAdjustmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CanaryAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CanaryAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One canary adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAdjustment {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub attempts: i64,
    pub source: String,
    pub pinned: bool,
    pub notes: String,
    pub status: CanaryAdjustmentStatus,
}

impl CanaryAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryAdjustmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryAdjustmentStatus) -> bool {
        match self.status {
            CanaryAdjustmentStatus::Created => {
                match next {
                    CanaryAdjustmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryAdjustmentStatus::Validated => {
                match next {
                    CanaryAdjustmentStatus::Published | CanaryAdjustmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryAdjustmentStatus::Published => {
                return next == CanaryAdjustmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.attempts as f64
    }

    pub fn copy(&self) -> CanaryAdjustment {
        CanaryAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            attempts: self.attempts,
            source: self.source.clone(),
            pinned: self.pinned,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a canary adjustment in status created.
pub fn new_canary_adjustment(code: &str, title: &str) -> CanaryAdjustment {
    CanaryAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        attempts: 0,
        source: "".to_string(),
        pinned: false,
        notes: "".to_string(),
        status: CanaryAdjustmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_canary_adjustment("code-968", "title-512");
        assert_eq!(item.status, CanaryAdjustmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_canary_adjustment("code-968", "title-512");
        if item.move_to(CanaryAdjustmentStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryAdjustmentStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_canary_adjustment("code-968", "title-512");
        item.status = CanaryAdjustmentStatus::Retired;
        assert!(
            item.move_to(CanaryAdjustmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
