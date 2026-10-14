use super::*;

use acme_kit::text;

/// Where a suspension variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionVarianceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SuspensionVarianceStatus {
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

impl std::fmt::Display for SuspensionVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a suspension variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionVariance {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub batch_size: i64,
    pub starts_at: i64,
    pub expires_at: i64,
    pub threshold: f64,
    pub status: SuspensionVarianceStatus,
}

impl SuspensionVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionVarianceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionVarianceStatus) -> bool {
        match self.status {
            SuspensionVarianceStatus::Planned => {
                match next {
                    SuspensionVarianceStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionVarianceStatus::Scheduled => {
                return next == SuspensionVarianceStatus::InProgress || next == SuspensionVarianceStatus::Cancelled;
            }
            SuspensionVarianceStatus::InProgress => {
                return next == SuspensionVarianceStatus::Completed;
            }
            SuspensionVarianceStatus::Completed => {
                return next == SuspensionVarianceStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension variance {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.starts_at as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionVariance {
        SuspensionVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            batch_size: self.batch_size,
            starts_at: self.starts_at,
            expires_at: self.expires_at,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a suspension variance in status planned.
pub fn new_suspension_variance(code: &str, label: &str) -> SuspensionVariance {
    SuspensionVariance {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        batch_size: 0,
        starts_at: 0,
        expires_at: 0,
        threshold: 0.0,
        status: SuspensionVarianceStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_suspension_variance("code-339", "label-483");
        assert_eq!(item.status, SuspensionVarianceStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_suspension_variance("code-339", "label-483");
        if item.move_to(SuspensionVarianceStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionVarianceStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_suspension_variance("code-339", "label-483");
        item.status = SuspensionVarianceStatus::Cancelled;
        assert!(
            item.move_to(SuspensionVarianceStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
