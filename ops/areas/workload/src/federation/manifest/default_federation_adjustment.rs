use super::*;

use acme_kit::text;

/// Where a default federation adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultFederationAdjustmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DefaultFederationAdjustmentStatus {
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

impl std::fmt::Display for DefaultFederationAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default federation adjustment as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFederationAdjustment {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub amount: f64,
    pub threshold: f64,
    pub minimum: f64,
    pub key_id: String,
    pub batch_size: i64,
    pub ends_at: i64,
    pub notes: String,
    pub status: DefaultFederationAdjustmentStatus,
}

impl DefaultFederationAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultFederationAdjustmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultFederationAdjustmentStatus) -> bool {
        match self.status {
            DefaultFederationAdjustmentStatus::Planned => {
                return next == DefaultFederationAdjustmentStatus::Cancelled;
            }
            DefaultFederationAdjustmentStatus::Scheduled => {
                return next == DefaultFederationAdjustmentStatus::InProgress
                    || next == DefaultFederationAdjustmentStatus::Completed;
            }
            DefaultFederationAdjustmentStatus::InProgress => {
                match next {
                    DefaultFederationAdjustmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultFederationAdjustmentStatus::Completed => {
                match next {
                    DefaultFederationAdjustmentStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: DefaultFederationAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default federation adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default federation adjustment", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.amount + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultFederationAdjustment {
        DefaultFederationAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            amount: self.amount,
            threshold: self.threshold,
            minimum: self.minimum,
            key_id: self.key_id.clone(),
            batch_size: self.batch_size,
            ends_at: self.ends_at,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default federation adjustment in status planned.
pub fn new_default_federation_adjustment(code: &str, name: &str) -> DefaultFederationAdjustment {
    DefaultFederationAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        audience: "".to_string(),
        amount: 0.0,
        threshold: 0.0,
        minimum: 0.0,
        key_id: "".to_string(),
        batch_size: 0,
        ends_at: 0,
        notes: "".to_string(),
        status: DefaultFederationAdjustmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_default_federation_adjustment("code-915", "name-97");
        assert_eq!(item.status, DefaultFederationAdjustmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_default_federation_adjustment("code-915", "name-97");
        if item.move_to(DefaultFederationAdjustmentStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultFederationAdjustmentStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_default_federation_adjustment("code-915", "name-97");
        item.status = DefaultFederationAdjustmentStatus::Cancelled;
        assert!(
            item.move_to(DefaultFederationAdjustmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
