use super::*;

use acme_kit::text;

/// Where a batched ttl policy window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedTtlPolicyWindowStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedTtlPolicyWindowStatus {
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

impl std::fmt::Display for BatchedTtlPolicyWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched ttl policy window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTtlPolicyWindow {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub checksum: String,
    pub external_ref: String,
    pub locale: String,
    pub key_id: String,
    pub phone: String,
    pub region: String,
    pub threshold: f64,
    pub discount: f64,
    pub status: BatchedTtlPolicyWindowStatus,
}

impl BatchedTtlPolicyWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedTtlPolicyWindowStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedTtlPolicyWindowStatus) -> bool {
        match self.status {
            BatchedTtlPolicyWindowStatus::Draft => {
                match next {
                    BatchedTtlPolicyWindowStatus::Active | BatchedTtlPolicyWindowStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedTtlPolicyWindowStatus::Pending => {
                match next {
                    BatchedTtlPolicyWindowStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedTtlPolicyWindowStatus::Active => {
                return next == BatchedTtlPolicyWindowStatus::Suspended || next == BatchedTtlPolicyWindowStatus::Closed;
            }
            BatchedTtlPolicyWindowStatus::Suspended => {
                return next == BatchedTtlPolicyWindowStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedTtlPolicyWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched ttl policy window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.discount - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedTtlPolicyWindow {
        BatchedTtlPolicyWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            checksum: self.checksum.clone(),
            external_ref: self.external_ref.clone(),
            locale: self.locale.clone(),
            key_id: self.key_id.clone(),
            phone: self.phone.clone(),
            region: self.region.clone(),
            threshold: self.threshold,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a batched ttl policy window in status draft.
pub fn new_batched_ttl_policy_window(code: &str, name: &str) -> BatchedTtlPolicyWindow {
    BatchedTtlPolicyWindow {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        checksum: "".to_string(),
        external_ref: "".to_string(),
        locale: "".to_string(),
        key_id: "".to_string(),
        phone: "".to_string(),
        region: "".to_string(),
        threshold: 0.0,
        discount: 0.0,
        status: BatchedTtlPolicyWindowStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_batched_ttl_policy_window("code-285", "name-897");
        assert_eq!(item.status, BatchedTtlPolicyWindowStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_batched_ttl_policy_window("code-285", "name-897");
        if item.move_to(BatchedTtlPolicyWindowStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedTtlPolicyWindowStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_batched_ttl_policy_window("code-285", "name-897");
        item.status = BatchedTtlPolicyWindowStatus::Closed;
        assert!(
            item.move_to(BatchedTtlPolicyWindowStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
