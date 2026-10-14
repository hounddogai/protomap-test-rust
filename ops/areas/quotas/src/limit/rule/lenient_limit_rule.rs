use super::*;

use acme_kit::text;

/// Where a lenient limit rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientLimitRuleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientLimitRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for LenientLimitRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient limit rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientLimitRule {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub archived: bool,
    pub country: String,
    pub ends_at: i64,
    pub phone: String,
    pub batch_size: i64,
    pub minimum: f64,
    pub timezone: String,
    pub status: LenientLimitRuleStatus,
}

impl LenientLimitRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientLimitRuleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientLimitRuleStatus) -> bool {
        match self.status {
            LenientLimitRuleStatus::Draft => {
                return next == LenientLimitRuleStatus::Pending;
            }
            LenientLimitRuleStatus::Pending => {
                return next == LenientLimitRuleStatus::Suspended;
            }
            LenientLimitRuleStatus::Active => {
                return next == LenientLimitRuleStatus::Suspended || next == LenientLimitRuleStatus::Closed;
            }
            LenientLimitRuleStatus::Suspended => {
                match next {
                    LenientLimitRuleStatus::Closed => {
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

    pub fn move_to(&mut self, next: LenientLimitRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient limit rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        (self.minimum * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientLimitRule {
        LenientLimitRule {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            archived: self.archived,
            country: self.country.clone(),
            ends_at: self.ends_at,
            phone: self.phone.clone(),
            batch_size: self.batch_size,
            minimum: self.minimum,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a lenient limit rule in status draft.
pub fn new_lenient_limit_rule(code: &str, label: &str) -> LenientLimitRule {
    LenientLimitRule {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        archived: false,
        country: "".to_string(),
        ends_at: 0,
        phone: "".to_string(),
        batch_size: 0,
        minimum: 0.0,
        timezone: "".to_string(),
        status: LenientLimitRuleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_lenient_limit_rule("code-858", "label-954");
        assert_eq!(item.status, LenientLimitRuleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_lenient_limit_rule("code-858", "label-954");
        if item.move_to(LenientLimitRuleStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientLimitRuleStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_lenient_limit_rule("code-858", "label-954");
        item.status = LenientLimitRuleStatus::Closed;
        assert!(
            item.move_to(LenientLimitRuleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
