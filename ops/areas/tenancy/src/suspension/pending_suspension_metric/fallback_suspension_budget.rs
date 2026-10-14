use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a fallback suspension budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackSuspensionBudgetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackSuspensionBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for FallbackSuspensionBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback suspension budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSuspensionBudget {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub attributes: BTreeMap<String, String>,
    pub category: String,
    pub tenant_id: String,
    pub created_at: i64,
    pub ends_at: i64,
    pub weight: f64,
    pub retries: i64,
    pub status: FallbackSuspensionBudgetStatus,
}

impl FallbackSuspensionBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackSuspensionBudgetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackSuspensionBudgetStatus) -> bool {
        match self.status {
            FallbackSuspensionBudgetStatus::Planned => {
                return next == FallbackSuspensionBudgetStatus::Cancelled;
            }
            FallbackSuspensionBudgetStatus::Scheduled => {
                match next {
                    FallbackSuspensionBudgetStatus::Completed | FallbackSuspensionBudgetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackSuspensionBudgetStatus::InProgress => {
                match next {
                    FallbackSuspensionBudgetStatus::Completed | FallbackSuspensionBudgetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackSuspensionBudgetStatus::Completed => {
                return next == FallbackSuspensionBudgetStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackSuspensionBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback suspension budget {} cannot move from {} to {}",
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
        format!("{} {}", "fallback suspension budget", self.code)
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

    pub fn net_exposure(&self) -> f64 {
        ((self.weight - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackSuspensionBudget {
        FallbackSuspensionBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            attributes: self.attributes.clone(),
            category: self.category.clone(),
            tenant_id: self.tenant_id.clone(),
            created_at: self.created_at,
            ends_at: self.ends_at,
            weight: self.weight,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a fallback suspension budget in status planned.
pub fn new_fallback_suspension_budget(code: &str, label: &str) -> FallbackSuspensionBudget {
    FallbackSuspensionBudget {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        attributes: BTreeMap::new(),
        category: "".to_string(),
        tenant_id: "".to_string(),
        created_at: 0,
        ends_at: 0,
        weight: 0.0,
        retries: 0,
        status: FallbackSuspensionBudgetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_fallback_suspension_budget("code-50", "label-85");
        assert_eq!(item.status, FallbackSuspensionBudgetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_fallback_suspension_budget("code-50", "label-85");
        if item.move_to(FallbackSuspensionBudgetStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackSuspensionBudgetStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_fallback_suspension_budget("code-50", "label-85");
        item.status = FallbackSuspensionBudgetStatus::Cancelled;
        assert!(
            item.move_to(FallbackSuspensionBudgetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
