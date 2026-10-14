use super::*;

use acme_kit::text;

/// Where a federation rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationRuleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FederationRuleStatus {
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

impl std::fmt::Display for FederationRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation rule as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationRule {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub summary: String,
    pub tenant_id: String,
    pub score: f64,
    pub created_at: i64,
    pub category: String,
    pub status: FederationRuleStatus,
}

impl FederationRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationRuleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationRuleStatus) -> bool {
        match self.status {
            FederationRuleStatus::Draft => {
                return next == FederationRuleStatus::Pending || next == FederationRuleStatus::Active;
            }
            FederationRuleStatus::Pending => {
                match next {
                    FederationRuleStatus::Active | FederationRuleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationRuleStatus::Active => {
                match next {
                    FederationRuleStatus::Suspended | FederationRuleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationRuleStatus::Suspended => {
                match next {
                    FederationRuleStatus::Closed => {
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

    pub fn move_to(&mut self, next: FederationRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        (self.score * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationRule {
        FederationRule {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            summary: self.summary.clone(),
            tenant_id: self.tenant_id.clone(),
            score: self.score,
            created_at: self.created_at,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a federation rule in status draft.
pub fn new_federation_rule(code: &str, label: &str) -> FederationRule {
    FederationRule {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        summary: "".to_string(),
        tenant_id: "".to_string(),
        score: 0.0,
        created_at: 0,
        category: "".to_string(),
        status: FederationRuleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_federation_rule("code-575", "label-524");
        assert_eq!(item.status, FederationRuleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_federation_rule("code-575", "label-524");
        if item.move_to(FederationRuleStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationRuleStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_federation_rule("code-575", "label-524");
        item.status = FederationRuleStatus::Closed;
        assert!(
            item.move_to(FederationRuleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
