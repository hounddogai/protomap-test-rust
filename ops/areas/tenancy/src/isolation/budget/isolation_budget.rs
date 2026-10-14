use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a isolation budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationBudgetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl IsolationBudgetStatus {
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

impl std::fmt::Display for IsolationBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One isolation budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationBudget {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub currency: String,
    pub revision: i64,
    pub host: String,
    pub attributes: BTreeMap<String, String>,
    pub status: IsolationBudgetStatus,
}

impl IsolationBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationBudgetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationBudgetStatus) -> bool {
        match self.status {
            IsolationBudgetStatus::Draft => {
                return next == IsolationBudgetStatus::Closed;
            }
            IsolationBudgetStatus::Pending => {
                return next == IsolationBudgetStatus::Active;
            }
            IsolationBudgetStatus::Active => {
                match next {
                    IsolationBudgetStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationBudgetStatus::Suspended => {
                match next {
                    IsolationBudgetStatus::Closed => {
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

    pub fn move_to(&mut self, next: IsolationBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "isolation budget", self.code)
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

    pub fn measured_value(&self) -> f64 {
        self.revision as f64
    }

    pub fn copy(&self) -> IsolationBudget {
        IsolationBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            currency: self.currency.clone(),
            revision: self.revision,
            host: self.host.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a isolation budget in status draft.
pub fn new_isolation_budget(code: &str, title: &str) -> IsolationBudget {
    IsolationBudget {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        currency: "".to_string(),
        revision: 0,
        host: "".to_string(),
        attributes: BTreeMap::new(),
        status: IsolationBudgetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_isolation_budget("code-445", "title-45");
        assert_eq!(item.status, IsolationBudgetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_isolation_budget("code-445", "title-45");
        if item.move_to(IsolationBudgetStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationBudgetStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_isolation_budget("code-445", "title-45");
        item.status = IsolationBudgetStatus::Closed;
        assert!(
            item.move_to(IsolationBudgetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
