use super::*;

use acme_kit::text;

/// Where a health check entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckEntryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl HealthCheckEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for HealthCheckEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a health check entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckEntry {
    pub code: String,
    pub label: String,
    pub category: String,
    pub unit: String,
    pub score: f64,
    pub scope: String,
    pub status: HealthCheckEntryStatus,
}

impl HealthCheckEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckEntryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckEntryStatus) -> bool {
        match self.status {
            HealthCheckEntryStatus::Requested => {
                return next == HealthCheckEntryStatus::Expired;
            }
            HealthCheckEntryStatus::Granted => {
                return next == HealthCheckEntryStatus::Revoked || next == HealthCheckEntryStatus::Expired;
            }
            HealthCheckEntryStatus::Revoked => {
                match next {
                    HealthCheckEntryStatus::Expired => {
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

    pub fn move_to(&mut self, next: HealthCheckEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> HealthCheckEntry {
        HealthCheckEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            unit: self.unit.clone(),
            score: self.score,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a health check entry in status requested.
pub fn new_health_check_entry(code: &str, label: &str) -> HealthCheckEntry {
    HealthCheckEntry {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        unit: "".to_string(),
        score: 0.0,
        scope: "".to_string(),
        status: HealthCheckEntryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_health_check_entry("code-767", "label-734");
        assert_eq!(item.status, HealthCheckEntryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_health_check_entry("code-767", "label-734");
        if item.move_to(HealthCheckEntryStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckEntryStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_health_check_entry("code-767", "label-734");
        item.status = HealthCheckEntryStatus::Expired;
        assert!(
            item.move_to(HealthCheckEntryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
