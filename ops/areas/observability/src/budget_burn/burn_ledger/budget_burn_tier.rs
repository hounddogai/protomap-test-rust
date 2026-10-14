use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a budget burn tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnTierStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BudgetBurnTierStatus {
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

impl std::fmt::Display for BudgetBurnTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn tier as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnTier {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub window_days: i64,
    pub namespace: String,
    pub summary: String,
    pub locale: String,
    pub ends_at: i64,
    pub status: BudgetBurnTierStatus,
}

impl BudgetBurnTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnTierStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnTierStatus) -> bool {
        match self.status {
            BudgetBurnTierStatus::Draft => {
                match next {
                    BudgetBurnTierStatus::Active | BudgetBurnTierStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnTierStatus::Pending => {
                match next {
                    BudgetBurnTierStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnTierStatus::Active => {
                match next {
                    BudgetBurnTierStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnTierStatus::Suspended => {
                return next == BudgetBurnTierStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BudgetBurnTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn tier {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn net_value(&self) -> f64 {
        ((self.ends_at as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnTier {
        BudgetBurnTier {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            window_days: self.window_days,
            namespace: self.namespace.clone(),
            summary: self.summary.clone(),
            locale: self.locale.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a budget burn tier in status draft.
pub fn new_budget_burn_tier(code: &str, label: &str) -> BudgetBurnTier {
    BudgetBurnTier {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        window_days: 0,
        namespace: "".to_string(),
        summary: "".to_string(),
        locale: "".to_string(),
        ends_at: 0,
        status: BudgetBurnTierStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_budget_burn_tier("code-921", "label-936");
        assert_eq!(item.status, BudgetBurnTierStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_budget_burn_tier("code-921", "label-936");
        if item.move_to(BudgetBurnTierStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnTierStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_budget_burn_tier("code-921", "label-936");
        item.status = BudgetBurnTierStatus::Closed;
        assert!(
            item.move_to(BudgetBurnTierStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
