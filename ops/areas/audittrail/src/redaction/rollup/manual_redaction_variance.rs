use super::*;

use acme_kit::text;

/// Where a manual redaction variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRedactionVarianceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ManualRedactionVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for ManualRedactionVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual redaction variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRedactionVariance {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub window_days: i64,
    pub budget: f64,
    pub score: f64,
    pub weight: f64,
    pub host: String,
    pub rate: f64,
    pub notes: String,
    pub status: ManualRedactionVarianceStatus,
}

impl ManualRedactionVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRedactionVarianceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRedactionVarianceStatus) -> bool {
        match self.status {
            ManualRedactionVarianceStatus::Planned => {
                match next {
                    ManualRedactionVarianceStatus::Completed | ManualRedactionVarianceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRedactionVarianceStatus::Scheduled => {
                return next == ManualRedactionVarianceStatus::InProgress
                    || next == ManualRedactionVarianceStatus::Cancelled;
            }
            ManualRedactionVarianceStatus::InProgress => {
                match next {
                    ManualRedactionVarianceStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRedactionVarianceStatus::Completed => {
                return next == ManualRedactionVarianceStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualRedactionVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual redaction variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.weight + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualRedactionVariance {
        ManualRedactionVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            window_days: self.window_days,
            budget: self.budget,
            score: self.score,
            weight: self.weight,
            host: self.host.clone(),
            rate: self.rate,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual redaction variance in status planned.
pub fn new_manual_redaction_variance(code: &str, name: &str) -> ManualRedactionVariance {
    ManualRedactionVariance {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        window_days: 0,
        budget: 0.0,
        score: 0.0,
        weight: 0.0,
        host: "".to_string(),
        rate: 0.0,
        notes: "".to_string(),
        status: ManualRedactionVarianceStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_manual_redaction_variance("code-938", "name-702");
        assert_eq!(item.status, ManualRedactionVarianceStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_manual_redaction_variance("code-938", "name-702");
        if item.move_to(ManualRedactionVarianceStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualRedactionVarianceStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_manual_redaction_variance("code-938", "name-702");
        item.status = ManualRedactionVarianceStatus::Cancelled;
        assert!(
            item.move_to(ManualRedactionVarianceStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
