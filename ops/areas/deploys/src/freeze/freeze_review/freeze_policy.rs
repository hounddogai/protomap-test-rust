use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a freeze policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezePolicyStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FreezePolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FreezePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze policy as the freeze team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezePolicy {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub minimum: f64,
    pub score: f64,
    pub key_id: String,
    pub created_at: i64,
    pub amount: f64,
    pub attempts: i64,
    pub scope: String,
    pub status: FreezePolicyStatus,
}

impl FreezePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezePolicyStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezePolicyStatus) -> bool {
        match self.status {
            FreezePolicyStatus::Created => {
                match next {
                    FreezePolicyStatus::Published | FreezePolicyStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezePolicyStatus::Validated => {
                match next {
                    FreezePolicyStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezePolicyStatus::Published => {
                return next == FreezePolicyStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze policy {} cannot move from {} to {}",
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
        format!("{} {}", "freeze policy", self.code)
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

    pub fn projected_load(&self) -> f64 {
        (self.amount * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezePolicy {
        FreezePolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            minimum: self.minimum,
            score: self.score,
            key_id: self.key_id.clone(),
            created_at: self.created_at,
            amount: self.amount,
            attempts: self.attempts,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a freeze policy in status created.
pub fn new_freeze_policy(code: &str, label: &str) -> FreezePolicy {
    FreezePolicy {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        minimum: 0.0,
        score: 0.0,
        key_id: "".to_string(),
        created_at: 0,
        amount: 0.0,
        attempts: 0,
        scope: "".to_string(),
        status: FreezePolicyStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_freeze_policy("code-784", "label-750");
        assert_eq!(item.status, FreezePolicyStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_freeze_policy("code-784", "label-750");
        if item.move_to(FreezePolicyStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezePolicyStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_freeze_policy("code-784", "label-750");
        item.status = FreezePolicyStatus::Retired;
        assert!(
            item.move_to(FreezePolicyStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
