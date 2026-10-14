use super::*;

use acme_kit::text;

/// Where a lenient plan cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientPlanCycleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LenientPlanCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LenientPlanCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient plan cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanCycle {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub threshold: f64,
    pub locale: String,
    pub starts_at: i64,
    pub status: LenientPlanCycleStatus,
}

impl LenientPlanCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientPlanCycleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientPlanCycleStatus) -> bool {
        match self.status {
            LenientPlanCycleStatus::Proposed => {
                match next {
                    LenientPlanCycleStatus::Approved | LenientPlanCycleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPlanCycleStatus::Approved => {
                return next == LenientPlanCycleStatus::Rejected || next == LenientPlanCycleStatus::Withdrawn;
            }
            LenientPlanCycleStatus::Rejected => {
                return next == LenientPlanCycleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientPlanCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient plan cycle {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.retries as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientPlanCycle {
        LenientPlanCycle {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            threshold: self.threshold,
            locale: self.locale.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a lenient plan cycle in status proposed.
pub fn new_lenient_plan_cycle(code: &str, title: &str) -> LenientPlanCycle {
    LenientPlanCycle {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        threshold: 0.0,
        locale: "".to_string(),
        starts_at: 0,
        status: LenientPlanCycleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_lenient_plan_cycle("code-791", "title-357");
        assert_eq!(item.status, LenientPlanCycleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_lenient_plan_cycle("code-791", "title-357");
        if item.move_to(LenientPlanCycleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientPlanCycleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_lenient_plan_cycle("code-791", "title-357");
        item.status = LenientPlanCycleStatus::Withdrawn;
        assert!(
            item.move_to(LenientPlanCycleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
