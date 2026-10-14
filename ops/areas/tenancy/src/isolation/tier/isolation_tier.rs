use super::*;

use acme_kit::text;

/// Where a isolation tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationTierStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl IsolationTierStatus {
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

impl std::fmt::Display for IsolationTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A isolation tier as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationTier {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub source: String,
    pub window_days: i64,
    pub summary: String,
    pub score: f64,
    pub fingerprint: String,
    pub revision: i64,
    pub host: String,
    pub status: IsolationTierStatus,
}

impl IsolationTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationTierStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationTierStatus) -> bool {
        match self.status {
            IsolationTierStatus::Planned => {
                match next {
                    IsolationTierStatus::InProgress | IsolationTierStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationTierStatus::Scheduled => {
                match next {
                    IsolationTierStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationTierStatus::InProgress => {
                return next == IsolationTierStatus::Completed;
            }
            IsolationTierStatus::Completed => {
                match next {
                    IsolationTierStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: IsolationTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "isolation tier", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        (self.weight * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationTier {
        IsolationTier {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            source: self.source.clone(),
            window_days: self.window_days,
            summary: self.summary.clone(),
            score: self.score,
            fingerprint: self.fingerprint.clone(),
            revision: self.revision,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a isolation tier in status planned.
pub fn new_isolation_tier(code: &str, label: &str) -> IsolationTier {
    IsolationTier {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        source: "".to_string(),
        window_days: 0,
        summary: "".to_string(),
        score: 0.0,
        fingerprint: "".to_string(),
        revision: 0,
        host: "".to_string(),
        status: IsolationTierStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_isolation_tier("code-538", "label-50");
        assert_eq!(item.status, IsolationTierStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_isolation_tier("code-538", "label-50");
        if item.move_to(IsolationTierStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationTierStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_isolation_tier("code-538", "label-50");
        item.status = IsolationTierStatus::Cancelled;
        assert!(
            item.move_to(IsolationTierStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
