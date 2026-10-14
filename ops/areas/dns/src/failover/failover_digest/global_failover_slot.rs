use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a global failover slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalFailoverSlotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalFailoverSlotStatus {
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

impl std::fmt::Display for GlobalFailoverSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global failover slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFailoverSlot {
    pub code: String,
    pub name: String,
    pub category: String,
    pub attributes: BTreeMap<String, String>,
    pub weight: f64,
    pub updated_at: i64,
    pub contact_email: String,
    pub port: i64,
    pub revision: i64,
    pub status: GlobalFailoverSlotStatus,
}

impl GlobalFailoverSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalFailoverSlotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalFailoverSlotStatus) -> bool {
        match self.status {
            GlobalFailoverSlotStatus::Draft => {
                match next {
                    GlobalFailoverSlotStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalFailoverSlotStatus::Pending => {
                return next == GlobalFailoverSlotStatus::Active;
            }
            GlobalFailoverSlotStatus::Active => {
                return next == GlobalFailoverSlotStatus::Suspended || next == GlobalFailoverSlotStatus::Closed;
            }
            GlobalFailoverSlotStatus::Suspended => {
                match next {
                    GlobalFailoverSlotStatus::Closed => {
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

    pub fn move_to(&mut self, next: GlobalFailoverSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global failover slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
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

    pub fn projected_total(&self) -> f64 {
        ((self.port as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalFailoverSlot {
        GlobalFailoverSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            attributes: self.attributes.clone(),
            weight: self.weight,
            updated_at: self.updated_at,
            contact_email: self.contact_email.clone(),
            port: self.port,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a global failover slot in status draft.
pub fn new_global_failover_slot(code: &str, name: &str) -> GlobalFailoverSlot {
    GlobalFailoverSlot {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        attributes: BTreeMap::new(),
        weight: 0.0,
        updated_at: 0,
        contact_email: "".to_string(),
        port: 0,
        revision: 0,
        status: GlobalFailoverSlotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_global_failover_slot("code-4", "name-800");
        assert_eq!(item.status, GlobalFailoverSlotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_global_failover_slot("code-4", "name-800");
        if item.move_to(GlobalFailoverSlotStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalFailoverSlotStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_global_failover_slot("code-4", "name-800");
        item.status = GlobalFailoverSlotStatus::Closed;
        assert!(
            item.move_to(GlobalFailoverSlotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
