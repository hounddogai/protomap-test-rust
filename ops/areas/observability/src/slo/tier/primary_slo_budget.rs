use super::*;

use acme_kit::text;

/// Where a primary slo budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimarySloBudgetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PrimarySloBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for PrimarySloBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary slo budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloBudget {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub rate: f64,
    pub batch_size: i64,
    pub capacity: i64,
    pub pinned: bool,
    pub ratio: f64,
    pub minimum: f64,
    pub status: PrimarySloBudgetStatus,
}

impl PrimarySloBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimarySloBudgetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimarySloBudgetStatus) -> bool {
        match self.status {
            PrimarySloBudgetStatus::Idle => {
                return next == PrimarySloBudgetStatus::Ready;
            }
            PrimarySloBudgetStatus::Warming => {
                return next == PrimarySloBudgetStatus::Draining || next == PrimarySloBudgetStatus::Stopped;
            }
            PrimarySloBudgetStatus::Ready => {
                return next == PrimarySloBudgetStatus::Draining || next == PrimarySloBudgetStatus::Stopped;
            }
            PrimarySloBudgetStatus::Draining => {
                return next == PrimarySloBudgetStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimarySloBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary slo budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary slo budget", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.ratio + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimarySloBudget {
        PrimarySloBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            rate: self.rate,
            batch_size: self.batch_size,
            capacity: self.capacity,
            pinned: self.pinned,
            ratio: self.ratio,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary slo budget in status idle.
pub fn new_primary_slo_budget(code: &str, title: &str) -> PrimarySloBudget {
    PrimarySloBudget {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        rate: 0.0,
        batch_size: 0,
        capacity: 0,
        pinned: false,
        ratio: 0.0,
        minimum: 0.0,
        status: PrimarySloBudgetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_primary_slo_budget("code-821", "title-457");
        assert_eq!(item.status, PrimarySloBudgetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_primary_slo_budget("code-821", "title-457");
        if item.move_to(PrimarySloBudgetStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimarySloBudgetStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_primary_slo_budget("code-821", "title-457");
        item.status = PrimarySloBudgetStatus::Stopped;
        assert!(
            item.move_to(PrimarySloBudgetStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
