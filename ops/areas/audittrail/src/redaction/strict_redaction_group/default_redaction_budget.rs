use super::*;

use acme_kit::text;

/// Where a default redaction budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRedactionBudgetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultRedactionBudgetStatus {
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

impl std::fmt::Display for DefaultRedactionBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default redaction budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionBudget {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub port: i64,
    pub description: String,
    pub cluster: String,
    pub threshold: f64,
    pub quantity: i64,
    pub weight: f64,
    pub visible: bool,
    pub status: DefaultRedactionBudgetStatus,
}

impl DefaultRedactionBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRedactionBudgetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRedactionBudgetStatus) -> bool {
        match self.status {
            DefaultRedactionBudgetStatus::Draft => {
                return next == DefaultRedactionBudgetStatus::Closed;
            }
            DefaultRedactionBudgetStatus::Pending => {
                match next {
                    DefaultRedactionBudgetStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRedactionBudgetStatus::Active => {
                return next == DefaultRedactionBudgetStatus::Suspended;
            }
            DefaultRedactionBudgetStatus::Suspended => {
                match next {
                    DefaultRedactionBudgetStatus::Closed => {
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

    pub fn move_to(&mut self, next: DefaultRedactionBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default redaction budget {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.weight * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultRedactionBudget {
        DefaultRedactionBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            port: self.port,
            description: self.description.clone(),
            cluster: self.cluster.clone(),
            threshold: self.threshold,
            quantity: self.quantity,
            weight: self.weight,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a default redaction budget in status draft.
pub fn new_default_redaction_budget(code: &str, title: &str) -> DefaultRedactionBudget {
    DefaultRedactionBudget {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        port: 0,
        description: "".to_string(),
        cluster: "".to_string(),
        threshold: 0.0,
        quantity: 0,
        weight: 0.0,
        visible: false,
        status: DefaultRedactionBudgetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_default_redaction_budget("code-159", "title-812");
        assert_eq!(item.status, DefaultRedactionBudgetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_default_redaction_budget("code-159", "title-812");
        if item.move_to(DefaultRedactionBudgetStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultRedactionBudgetStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_default_redaction_budget("code-159", "title-812");
        item.status = DefaultRedactionBudgetStatus::Closed;
        assert!(
            item.move_to(DefaultRedactionBudgetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
