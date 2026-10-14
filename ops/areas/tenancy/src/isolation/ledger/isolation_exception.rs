use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a isolation exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationExceptionStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl IsolationExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for IsolationExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A isolation exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationException {
    pub code: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub namespace: String,
    pub reviewer: String,
    pub attributes: BTreeMap<String, String>,
    pub key_id: String,
    pub owner_id: String,
    pub locale: String,
    pub batch_size: i64,
    pub burst: i64,
    pub status: IsolationExceptionStatus,
}

impl IsolationException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationExceptionStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationExceptionStatus) -> bool {
        match self.status {
            IsolationExceptionStatus::Created => {
                match next {
                    IsolationExceptionStatus::Validated | IsolationExceptionStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationExceptionStatus::Validated => {
                match next {
                    IsolationExceptionStatus::Published | IsolationExceptionStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationExceptionStatus::Published => {
                match next {
                    IsolationExceptionStatus::Retired => {
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

    pub fn move_to(&mut self, next: IsolationExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
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
        ((self.burst as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationException {
        IsolationException {
            code: self.code.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            namespace: self.namespace.clone(),
            reviewer: self.reviewer.clone(),
            attributes: self.attributes.clone(),
            key_id: self.key_id.clone(),
            owner_id: self.owner_id.clone(),
            locale: self.locale.clone(),
            batch_size: self.batch_size,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a isolation exception in status created.
pub fn new_isolation_exception(code: &str, name: &str) -> IsolationException {
    IsolationException {
        code: code.to_string(),
        name: name.to_string(),
        ttl_seconds: 0,
        namespace: "".to_string(),
        reviewer: "".to_string(),
        attributes: BTreeMap::new(),
        key_id: "".to_string(),
        owner_id: "".to_string(),
        locale: "".to_string(),
        batch_size: 0,
        burst: 0,
        status: IsolationExceptionStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_isolation_exception("code-566", "name-67");
        assert_eq!(item.status, IsolationExceptionStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_isolation_exception("code-566", "name-67");
        if item.move_to(IsolationExceptionStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationExceptionStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_isolation_exception("code-566", "name-67");
        item.status = IsolationExceptionStatus::Retired;
        assert!(
            item.move_to(IsolationExceptionStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
