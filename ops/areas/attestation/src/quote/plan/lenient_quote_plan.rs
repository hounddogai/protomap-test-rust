use super::*;

use acme_kit::text;

/// Where a lenient quote plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientQuotePlanStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientQuotePlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientQuotePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient quote plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientQuotePlan {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub duration_minutes: i64,
    pub ceiling: f64,
    pub quantity: i64,
    pub description: String,
    pub summary: String,
    pub status: LenientQuotePlanStatus,
}

impl LenientQuotePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientQuotePlanStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientQuotePlanStatus) -> bool {
        match self.status {
            LenientQuotePlanStatus::Created => {
                match next {
                    LenientQuotePlanStatus::Validated | LenientQuotePlanStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientQuotePlanStatus::Validated => {
                match next {
                    LenientQuotePlanStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientQuotePlanStatus::Published => {
                match next {
                    LenientQuotePlanStatus::Retired => {
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

    pub fn move_to(&mut self, next: LenientQuotePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient quote plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.duration_minutes as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientQuotePlan {
        LenientQuotePlan {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            duration_minutes: self.duration_minutes,
            ceiling: self.ceiling,
            quantity: self.quantity,
            description: self.description.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient quote plan in status created.
pub fn new_lenient_quote_plan(code: &str, title: &str) -> LenientQuotePlan {
    LenientQuotePlan {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        duration_minutes: 0,
        ceiling: 0.0,
        quantity: 0,
        description: "".to_string(),
        summary: "".to_string(),
        status: LenientQuotePlanStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_quote_plan("code-647", "title-923");
        assert_eq!(item.status, LenientQuotePlanStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_lenient_quote_plan("code-647", "title-923");
        if item.move_to(LenientQuotePlanStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientQuotePlanStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_quote_plan("code-647", "title-923");
        item.status = LenientQuotePlanStatus::Retired;
        assert!(
            item.move_to(LenientQuotePlanStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
