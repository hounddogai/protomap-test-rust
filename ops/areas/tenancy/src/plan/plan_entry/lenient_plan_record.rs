use super::*;

use acme_kit::text;

/// Where a lenient plan record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientPlanRecordStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LenientPlanRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LenientPlanRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient plan record as the plan team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanRecord {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub unit: String,
    pub ratio: f64,
    pub rate: f64,
    pub attempts: i64,
    pub expires_at: i64,
    pub reviewer: String,
    pub archived: bool,
    pub tags: Vec<String>,
    pub status: LenientPlanRecordStatus,
}

impl LenientPlanRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientPlanRecordStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientPlanRecordStatus) -> bool {
        match self.status {
            LenientPlanRecordStatus::Proposed => {
                return next == LenientPlanRecordStatus::Approved || next == LenientPlanRecordStatus::Withdrawn;
            }
            LenientPlanRecordStatus::Approved => {
                match next {
                    LenientPlanRecordStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPlanRecordStatus::Rejected => {
                return next == LenientPlanRecordStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientPlanRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient plan record {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_load(&self) -> f64 {
        ((self.attempts as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientPlanRecord {
        LenientPlanRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            unit: self.unit.clone(),
            ratio: self.ratio,
            rate: self.rate,
            attempts: self.attempts,
            expires_at: self.expires_at,
            reviewer: self.reviewer.clone(),
            archived: self.archived,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a lenient plan record in status proposed.
pub fn new_lenient_plan_record(code: &str, name: &str) -> LenientPlanRecord {
    LenientPlanRecord {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        unit: "".to_string(),
        ratio: 0.0,
        rate: 0.0,
        attempts: 0,
        expires_at: 0,
        reviewer: "".to_string(),
        archived: false,
        tags: Vec::new(),
        status: LenientPlanRecordStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_lenient_plan_record("code-509", "name-72");
        assert_eq!(item.status, LenientPlanRecordStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_lenient_plan_record("code-509", "name-72");
        if item.move_to(LenientPlanRecordStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientPlanRecordStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_lenient_plan_record("code-509", "name-72");
        item.status = LenientPlanRecordStatus::Withdrawn;
        assert!(
            item.move_to(LenientPlanRecordStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
