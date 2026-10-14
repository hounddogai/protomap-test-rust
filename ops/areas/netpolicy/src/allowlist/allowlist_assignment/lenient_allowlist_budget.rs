use super::*;

use acme_kit::text;

/// Where a lenient allowlist budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientAllowlistBudgetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientAllowlistBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientAllowlistBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient allowlist budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientAllowlistBudget {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub label: String,
    pub region: String,
    pub floor_value: f64,
    pub country: String,
    pub ratio: f64,
    pub status: LenientAllowlistBudgetStatus,
}

impl LenientAllowlistBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientAllowlistBudgetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientAllowlistBudgetStatus) -> bool {
        match self.status {
            LenientAllowlistBudgetStatus::Created => {
                match next {
                    LenientAllowlistBudgetStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientAllowlistBudgetStatus::Validated => {
                match next {
                    LenientAllowlistBudgetStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientAllowlistBudgetStatus::Published => {
                match next {
                    LenientAllowlistBudgetStatus::Retired => {
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

    pub fn move_to(&mut self, next: LenientAllowlistBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient allowlist budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.floor_value + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientAllowlistBudget {
        LenientAllowlistBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            label: self.label.clone(),
            region: self.region.clone(),
            floor_value: self.floor_value,
            country: self.country.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a lenient allowlist budget in status created.
pub fn new_lenient_allowlist_budget(code: &str, name: &str) -> LenientAllowlistBudget {
    LenientAllowlistBudget {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        label: "".to_string(),
        region: "".to_string(),
        floor_value: 0.0,
        country: "".to_string(),
        ratio: 0.0,
        status: LenientAllowlistBudgetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_allowlist_budget("code-563", "name-118");
        assert_eq!(item.status, LenientAllowlistBudgetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_lenient_allowlist_budget("code-563", "name-118");
        if item.move_to(LenientAllowlistBudgetStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientAllowlistBudgetStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_allowlist_budget("code-563", "name-118");
        item.status = LenientAllowlistBudgetStatus::Retired;
        assert!(
            item.move_to(LenientAllowlistBudgetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
