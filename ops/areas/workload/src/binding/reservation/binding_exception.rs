use super::*;

use acme_kit::text;

/// Where a binding exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingExceptionStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BindingExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BindingExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingException {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub active: bool,
    pub cluster: String,
    pub updated_at: i64,
    pub sequence: i64,
    pub retries: i64,
    pub category: String,
    pub status: BindingExceptionStatus,
}

impl BindingException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingExceptionStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingExceptionStatus) -> bool {
        match self.status {
            BindingExceptionStatus::Created => {
                match next {
                    BindingExceptionStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingExceptionStatus::Validated => {
                return next == BindingExceptionStatus::Published || next == BindingExceptionStatus::Retired;
            }
            BindingExceptionStatus::Published => {
                match next {
                    BindingExceptionStatus::Retired => {
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

    pub fn move_to(&mut self, next: BindingExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding exception {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.updated_at as f64) * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingException {
        BindingException {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            active: self.active,
            cluster: self.cluster.clone(),
            updated_at: self.updated_at,
            sequence: self.sequence,
            retries: self.retries,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a binding exception in status created.
pub fn new_binding_exception(code: &str, title: &str) -> BindingException {
    BindingException {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        active: false,
        cluster: "".to_string(),
        updated_at: 0,
        sequence: 0,
        retries: 0,
        category: "".to_string(),
        status: BindingExceptionStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_binding_exception("code-417", "title-862");
        assert_eq!(item.status, BindingExceptionStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_binding_exception("code-417", "title-862");
        if item.move_to(BindingExceptionStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingExceptionStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_binding_exception("code-417", "title-862");
        item.status = BindingExceptionStatus::Retired;
        assert!(
            item.move_to(BindingExceptionStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
