use super::*;

use acme_kit::text;

/// Where a legacy federation adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyFederationAdjustmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LegacyFederationAdjustmentStatus {
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

impl std::fmt::Display for LegacyFederationAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy federation adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationAdjustment {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub ceiling: f64,
    pub ends_at: i64,
    pub phone: String,
    pub segment: String,
    pub grace_days: i64,
    pub limit: i64,
    pub retries: i64,
    pub floor_value: f64,
    pub status: LegacyFederationAdjustmentStatus,
}

impl LegacyFederationAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyFederationAdjustmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyFederationAdjustmentStatus) -> bool {
        match self.status {
            LegacyFederationAdjustmentStatus::Planned => {
                match next {
                    LegacyFederationAdjustmentStatus::Scheduled | LegacyFederationAdjustmentStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyFederationAdjustmentStatus::Scheduled => {
                match next {
                    LegacyFederationAdjustmentStatus::InProgress | LegacyFederationAdjustmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyFederationAdjustmentStatus::InProgress => {
                match next {
                    LegacyFederationAdjustmentStatus::Completed | LegacyFederationAdjustmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyFederationAdjustmentStatus::Completed => {
                return next == LegacyFederationAdjustmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyFederationAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy federation adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy federation adjustment", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.expires_at as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyFederationAdjustment {
        LegacyFederationAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            ceiling: self.ceiling,
            ends_at: self.ends_at,
            phone: self.phone.clone(),
            segment: self.segment.clone(),
            grace_days: self.grace_days,
            limit: self.limit,
            retries: self.retries,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a legacy federation adjustment in status planned.
pub fn new_legacy_federation_adjustment(code: &str, name: &str) -> LegacyFederationAdjustment {
    LegacyFederationAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        ceiling: 0.0,
        ends_at: 0,
        phone: "".to_string(),
        segment: "".to_string(),
        grace_days: 0,
        limit: 0,
        retries: 0,
        floor_value: 0.0,
        status: LegacyFederationAdjustmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_legacy_federation_adjustment("code-138", "name-330");
        assert_eq!(item.status, LegacyFederationAdjustmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_legacy_federation_adjustment("code-138", "name-330");
        if item.move_to(LegacyFederationAdjustmentStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyFederationAdjustmentStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_legacy_federation_adjustment("code-138", "name-330");
        item.status = LegacyFederationAdjustmentStatus::Cancelled;
        assert!(
            item.move_to(LegacyFederationAdjustmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
