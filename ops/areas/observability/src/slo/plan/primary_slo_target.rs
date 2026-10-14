use super::*;

use acme_kit::text;

/// Where a primary slo target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimarySloTargetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimarySloTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimarySloTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary slo target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloTarget {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub window_days: i64,
    pub quota_units: i64,
    pub discount: f64,
    pub cipher: String,
    pub ttl_seconds: i64,
    pub retries: i64,
    pub notes: String,
    pub principal: String,
    pub status: PrimarySloTargetStatus,
}

impl PrimarySloTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimarySloTargetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimarySloTargetStatus) -> bool {
        match self.status {
            PrimarySloTargetStatus::Proposed => {
                match next {
                    PrimarySloTargetStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimarySloTargetStatus::Approved => {
                return next == PrimarySloTargetStatus::Rejected;
            }
            PrimarySloTargetStatus::Rejected => {
                match next {
                    PrimarySloTargetStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PrimarySloTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary slo target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary slo target", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.discount - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimarySloTarget {
        PrimarySloTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            window_days: self.window_days,
            quota_units: self.quota_units,
            discount: self.discount,
            cipher: self.cipher.clone(),
            ttl_seconds: self.ttl_seconds,
            retries: self.retries,
            notes: self.notes.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary slo target in status proposed.
pub fn new_primary_slo_target(code: &str, label: &str) -> PrimarySloTarget {
    PrimarySloTarget {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        window_days: 0,
        quota_units: 0,
        discount: 0.0,
        cipher: "".to_string(),
        ttl_seconds: 0,
        retries: 0,
        notes: "".to_string(),
        principal: "".to_string(),
        status: PrimarySloTargetStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_primary_slo_target("code-574", "label-549");
        assert_eq!(item.status, PrimarySloTargetStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_primary_slo_target("code-574", "label-549");
        if item.move_to(PrimarySloTargetStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimarySloTargetStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_primary_slo_target("code-574", "label-549");
        item.status = PrimarySloTargetStatus::Withdrawn;
        assert!(
            item.move_to(PrimarySloTargetStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
