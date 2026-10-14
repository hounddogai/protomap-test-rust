use super::*;

use acme_kit::text;

/// Where a limit target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitTargetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LimitTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for LimitTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit target as the limit team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitTarget {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub title: String,
    pub duration_minutes: i64,
    pub budget: f64,
    pub batch_size: i64,
    pub unit: String,
    pub ratio: f64,
    pub quota_units: i64,
    pub status: LimitTargetStatus,
}

impl LimitTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitTargetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitTargetStatus) -> bool {
        match self.status {
            LimitTargetStatus::Draft => {
                return next == LimitTargetStatus::Pending || next == LimitTargetStatus::Active;
            }
            LimitTargetStatus::Pending => {
                match next {
                    LimitTargetStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitTargetStatus::Active => {
                return next == LimitTargetStatus::Suspended || next == LimitTargetStatus::Closed;
            }
            LimitTargetStatus::Suspended => {
                match next {
                    LimitTargetStatus::Closed => {
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

    pub fn move_to(&mut self, next: LimitTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.ratio + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitTarget {
        LimitTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            budget: self.budget,
            batch_size: self.batch_size,
            unit: self.unit.clone(),
            ratio: self.ratio,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a limit target in status draft.
pub fn new_limit_target(code: &str, label: &str) -> LimitTarget {
    LimitTarget {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        title: "".to_string(),
        duration_minutes: 0,
        budget: 0.0,
        batch_size: 0,
        unit: "".to_string(),
        ratio: 0.0,
        quota_units: 0,
        status: LimitTargetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_limit_target("code-201", "label-934");
        assert_eq!(item.status, LimitTargetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_limit_target("code-201", "label-934");
        if item.move_to(LimitTargetStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitTargetStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_limit_target("code-201", "label-934");
        item.status = LimitTargetStatus::Closed;
        assert!(item.move_to(LimitTargetStatus::Draft).is_err(), "expected move to fail");
    }
}
