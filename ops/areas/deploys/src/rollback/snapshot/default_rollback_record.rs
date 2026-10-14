use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a default rollback record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRollbackRecordStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultRollbackRecordStatus {
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

impl std::fmt::Display for DefaultRollbackRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default rollback record as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRollbackRecord {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub description: String,
    pub discount: f64,
    pub port: i64,
    pub batch_size: i64,
    pub attributes: BTreeMap<String, String>,
    pub audience: String,
    pub status: DefaultRollbackRecordStatus,
}

impl DefaultRollbackRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRollbackRecordStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRollbackRecordStatus) -> bool {
        match self.status {
            DefaultRollbackRecordStatus::Draft => {
                match next {
                    DefaultRollbackRecordStatus::Pending | DefaultRollbackRecordStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRollbackRecordStatus::Pending => {
                return next == DefaultRollbackRecordStatus::Active;
            }
            DefaultRollbackRecordStatus::Active => {
                return next == DefaultRollbackRecordStatus::Suspended || next == DefaultRollbackRecordStatus::Closed;
            }
            DefaultRollbackRecordStatus::Suspended => {
                return next == DefaultRollbackRecordStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultRollbackRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default rollback record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default rollback record", self.code)
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

    pub fn weighted_value(&self) -> f64 {
        ((self.batch_size as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultRollbackRecord {
        DefaultRollbackRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            cipher: self.cipher.clone(),
            description: self.description.clone(),
            discount: self.discount,
            port: self.port,
            batch_size: self.batch_size,
            attributes: self.attributes.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default rollback record in status draft.
pub fn new_default_rollback_record(code: &str, name: &str) -> DefaultRollbackRecord {
    DefaultRollbackRecord {
        code: code.to_string(),
        name: name.to_string(),
        cipher: "".to_string(),
        description: "".to_string(),
        discount: 0.0,
        port: 0,
        batch_size: 0,
        attributes: BTreeMap::new(),
        audience: "".to_string(),
        status: DefaultRollbackRecordStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_default_rollback_record("code-655", "name-955");
        assert_eq!(item.status, DefaultRollbackRecordStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_default_rollback_record("code-655", "name-955");
        if item.move_to(DefaultRollbackRecordStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultRollbackRecordStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_default_rollback_record("code-655", "name-955");
        item.status = DefaultRollbackRecordStatus::Closed;
        assert!(
            item.move_to(DefaultRollbackRecordStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
