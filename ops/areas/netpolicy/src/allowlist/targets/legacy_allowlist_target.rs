use super::*;

use acme_kit::text;

/// Where a legacy allowlist target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyAllowlistTargetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyAllowlistTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyAllowlistTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy allowlist target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistTarget {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub segment: String,
    pub ceiling: f64,
    pub currency: String,
    pub capacity: i64,
    pub status: LegacyAllowlistTargetStatus,
}

impl LegacyAllowlistTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyAllowlistTargetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyAllowlistTargetStatus) -> bool {
        match self.status {
            LegacyAllowlistTargetStatus::Created => {
                match next {
                    LegacyAllowlistTargetStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyAllowlistTargetStatus::Validated => {
                return next == LegacyAllowlistTargetStatus::Retired;
            }
            LegacyAllowlistTargetStatus::Published => {
                match next {
                    LegacyAllowlistTargetStatus::Retired => {
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

    pub fn move_to(&mut self, next: LegacyAllowlistTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy allowlist target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy allowlist target", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        (self.maximum * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyAllowlistTarget {
        LegacyAllowlistTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            segment: self.segment.clone(),
            ceiling: self.ceiling,
            currency: self.currency.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a legacy allowlist target in status created.
pub fn new_legacy_allowlist_target(code: &str, title: &str) -> LegacyAllowlistTarget {
    LegacyAllowlistTarget {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        segment: "".to_string(),
        ceiling: 0.0,
        currency: "".to_string(),
        capacity: 0,
        status: LegacyAllowlistTargetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_legacy_allowlist_target("code-802", "title-220");
        assert_eq!(item.status, LegacyAllowlistTargetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_legacy_allowlist_target("code-802", "title-220");
        if item.move_to(LegacyAllowlistTargetStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyAllowlistTargetStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_legacy_allowlist_target("code-802", "title-220");
        item.status = LegacyAllowlistTargetStatus::Retired;
        assert!(
            item.move_to(LegacyAllowlistTargetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
