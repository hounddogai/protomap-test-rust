use super::*;

use acme_kit::text;

/// Where a capped action item rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedActionItemRuleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CappedActionItemRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CappedActionItemRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped action item rule as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedActionItemRule {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub fingerprint: String,
    pub country: String,
    pub approved: bool,
    pub port: i64,
    pub ends_at: i64,
    pub status: CappedActionItemRuleStatus,
}

impl CappedActionItemRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedActionItemRuleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedActionItemRuleStatus) -> bool {
        match self.status {
            CappedActionItemRuleStatus::Created => {
                match next {
                    CappedActionItemRuleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedActionItemRuleStatus::Validated => {
                return next == CappedActionItemRuleStatus::Published;
            }
            CappedActionItemRuleStatus::Published => {
                return next == CappedActionItemRuleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedActionItemRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped action item rule {} cannot move from {} to {}",
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
        format!("{} {}", "capped action item rule", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_value(&self) -> f64 {
        ((self.port as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedActionItemRule {
        CappedActionItemRule {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            fingerprint: self.fingerprint.clone(),
            country: self.country.clone(),
            approved: self.approved,
            port: self.port,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a capped action item rule in status created.
pub fn new_capped_action_item_rule(code: &str, label: &str) -> CappedActionItemRule {
    CappedActionItemRule {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        fingerprint: "".to_string(),
        country: "".to_string(),
        approved: false,
        port: 0,
        ends_at: 0,
        status: CappedActionItemRuleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_capped_action_item_rule("code-574", "label-618");
        assert_eq!(item.status, CappedActionItemRuleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_capped_action_item_rule("code-574", "label-618");
        if item.move_to(CappedActionItemRuleStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedActionItemRuleStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_capped_action_item_rule("code-574", "label-618");
        item.status = CappedActionItemRuleStatus::Retired;
        assert!(
            item.move_to(CappedActionItemRuleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
