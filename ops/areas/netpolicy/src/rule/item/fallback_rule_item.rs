use super::*;

use acme_kit::text;

/// Where a fallback rule item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRuleItemStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FallbackRuleItemStatus {
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

impl std::fmt::Display for FallbackRuleItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback rule item and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRuleItem {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub notes: String,
    pub cost: f64,
    pub namespace: String,
    pub floor_value: f64,
    pub limit: i64,
    pub status: FallbackRuleItemStatus,
}

impl FallbackRuleItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRuleItemStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRuleItemStatus) -> bool {
        match self.status {
            FallbackRuleItemStatus::Draft => {
                return next == FallbackRuleItemStatus::Pending;
            }
            FallbackRuleItemStatus::Pending => {
                return next == FallbackRuleItemStatus::Active || next == FallbackRuleItemStatus::Suspended;
            }
            FallbackRuleItemStatus::Active => {
                match next {
                    FallbackRuleItemStatus::Suspended | FallbackRuleItemStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRuleItemStatus::Suspended => {
                match next {
                    FallbackRuleItemStatus::Closed => {
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

    pub fn move_to(&mut self, next: FallbackRuleItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback rule item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.margin - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRuleItem {
        FallbackRuleItem {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            notes: self.notes.clone(),
            cost: self.cost,
            namespace: self.namespace.clone(),
            floor_value: self.floor_value,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a fallback rule item in status draft.
pub fn new_fallback_rule_item(code: &str, title: &str) -> FallbackRuleItem {
    FallbackRuleItem {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        notes: "".to_string(),
        cost: 0.0,
        namespace: "".to_string(),
        floor_value: 0.0,
        limit: 0,
        status: FallbackRuleItemStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_fallback_rule_item("code-599", "title-669");
        assert_eq!(item.status, FallbackRuleItemStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_fallback_rule_item("code-599", "title-669");
        if item.move_to(FallbackRuleItemStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackRuleItemStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_fallback_rule_item("code-599", "title-669");
        item.status = FallbackRuleItemStatus::Closed;
        assert!(
            item.move_to(FallbackRuleItemStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
