use super::*;

use acme_kit::text;

/// Where a lenient sink budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientSinkBudgetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientSinkBudgetStatus {
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

impl std::fmt::Display for LenientSinkBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient sink budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientSinkBudget {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub tenant_id: String,
    pub ends_at: i64,
    pub tags: Vec<String>,
    pub namespace: String,
    pub status: LenientSinkBudgetStatus,
}

impl LenientSinkBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientSinkBudgetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientSinkBudgetStatus) -> bool {
        match self.status {
            LenientSinkBudgetStatus::Draft => {
                match next {
                    LenientSinkBudgetStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSinkBudgetStatus::Pending => {
                match next {
                    LenientSinkBudgetStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientSinkBudgetStatus::Active => {
                return next == LenientSinkBudgetStatus::Closed;
            }
            LenientSinkBudgetStatus::Suspended => {
                return next == LenientSinkBudgetStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientSinkBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient sink budget {} cannot move from {} to {}",
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
        format!("{} {}", "lenient sink budget", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> LenientSinkBudget {
        LenientSinkBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            tenant_id: self.tenant_id.clone(),
            ends_at: self.ends_at,
            tags: self.tags.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a lenient sink budget in status draft.
pub fn new_lenient_sink_budget(code: &str, label: &str) -> LenientSinkBudget {
    LenientSinkBudget {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        tenant_id: "".to_string(),
        ends_at: 0,
        tags: Vec::new(),
        namespace: "".to_string(),
        status: LenientSinkBudgetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_lenient_sink_budget("code-232", "label-411");
        assert_eq!(item.status, LenientSinkBudgetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_lenient_sink_budget("code-232", "label-411");
        if item.move_to(LenientSinkBudgetStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientSinkBudgetStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_lenient_sink_budget("code-232", "label-411");
        item.status = LenientSinkBudgetStatus::Closed;
        assert!(
            item.move_to(LenientSinkBudgetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
