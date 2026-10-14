use super::*;

use acme_kit::text;

/// Where a lenient rule plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRulePlanStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientRulePlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientRulePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient rule plan as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRulePlan {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub notes: String,
    pub rate: f64,
    pub country: String,
    pub owner_id: String,
    pub checksum: String,
    pub starts_at: i64,
    pub pinned: bool,
    pub status: LenientRulePlanStatus,
}

impl LenientRulePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRulePlanStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRulePlanStatus) -> bool {
        match self.status {
            LenientRulePlanStatus::Created => {
                return next == LenientRulePlanStatus::Validated;
            }
            LenientRulePlanStatus::Validated => {
                match next {
                    LenientRulePlanStatus::Published | LenientRulePlanStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRulePlanStatus::Published => {
                match next {
                    LenientRulePlanStatus::Retired => {
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

    pub fn move_to(&mut self, next: LenientRulePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient rule plan {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.rate - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientRulePlan {
        LenientRulePlan {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            notes: self.notes.clone(),
            rate: self.rate,
            country: self.country.clone(),
            owner_id: self.owner_id.clone(),
            checksum: self.checksum.clone(),
            starts_at: self.starts_at,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a lenient rule plan in status created.
pub fn new_lenient_rule_plan(code: &str, label: &str) -> LenientRulePlan {
    LenientRulePlan {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        notes: "".to_string(),
        rate: 0.0,
        country: "".to_string(),
        owner_id: "".to_string(),
        checksum: "".to_string(),
        starts_at: 0,
        pinned: false,
        status: LenientRulePlanStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_rule_plan("code-447", "label-523");
        assert_eq!(item.status, LenientRulePlanStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_lenient_rule_plan("code-447", "label-523");
        if item.move_to(LenientRulePlanStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientRulePlanStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_rule_plan("code-447", "label-523");
        item.status = LenientRulePlanStatus::Retired;
        assert!(
            item.move_to(LenientRulePlanStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
