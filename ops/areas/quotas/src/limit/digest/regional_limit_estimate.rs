use super::*;

use acme_kit::text;

/// Where a regional limit estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalLimitEstimateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RegionalLimitEstimateStatus {
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

impl std::fmt::Display for RegionalLimitEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional limit estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalLimitEstimate {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub fingerprint: String,
    pub updated_at: i64,
    pub host: String,
    pub status: RegionalLimitEstimateStatus,
}

impl RegionalLimitEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalLimitEstimateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalLimitEstimateStatus) -> bool {
        match self.status {
            RegionalLimitEstimateStatus::Planned => {
                match next {
                    RegionalLimitEstimateStatus::Completed | RegionalLimitEstimateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalLimitEstimateStatus::Scheduled => {
                match next {
                    RegionalLimitEstimateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalLimitEstimateStatus::InProgress => {
                match next {
                    RegionalLimitEstimateStatus::Completed | RegionalLimitEstimateStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalLimitEstimateStatus::Completed => {
                return next == RegionalLimitEstimateStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalLimitEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional limit estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional limit estimate", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.updated_at as f64
    }

    pub fn copy(&self) -> RegionalLimitEstimate {
        RegionalLimitEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            fingerprint: self.fingerprint.clone(),
            updated_at: self.updated_at,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional limit estimate in status planned.
pub fn new_regional_limit_estimate(code: &str, label: &str) -> RegionalLimitEstimate {
    RegionalLimitEstimate {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        fingerprint: "".to_string(),
        updated_at: 0,
        host: "".to_string(),
        status: RegionalLimitEstimateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_regional_limit_estimate("code-315", "label-442");
        assert_eq!(item.status, RegionalLimitEstimateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_regional_limit_estimate("code-315", "label-442");
        if item.move_to(RegionalLimitEstimateStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalLimitEstimateStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_regional_limit_estimate("code-315", "label-442");
        item.status = RegionalLimitEstimateStatus::Cancelled;
        assert!(
            item.move_to(RegionalLimitEstimateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
