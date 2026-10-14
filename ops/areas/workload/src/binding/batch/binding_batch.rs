use super::*;

use acme_kit::text;

/// Where a binding batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingBatchStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BindingBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BindingBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a binding batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingBatch {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub segment: String,
    pub grace_days: i64,
    pub amount: f64,
    pub budget: f64,
    pub created_at: i64,
    pub retries: i64,
    pub external_ref: String,
    pub ratio: f64,
    pub status: BindingBatchStatus,
}

impl BindingBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingBatchStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingBatchStatus) -> bool {
        match self.status {
            BindingBatchStatus::Requested => {
                match next {
                    BindingBatchStatus::Granted | BindingBatchStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingBatchStatus::Granted => {
                match next {
                    BindingBatchStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingBatchStatus::Revoked => {
                match next {
                    BindingBatchStatus::Expired => {
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

    pub fn move_to(&mut self, next: BindingBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding batch {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        (self.budget * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingBatch {
        BindingBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            segment: self.segment.clone(),
            grace_days: self.grace_days,
            amount: self.amount,
            budget: self.budget,
            created_at: self.created_at,
            retries: self.retries,
            external_ref: self.external_ref.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a binding batch in status requested.
pub fn new_binding_batch(code: &str, name: &str) -> BindingBatch {
    BindingBatch {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        segment: "".to_string(),
        grace_days: 0,
        amount: 0.0,
        budget: 0.0,
        created_at: 0,
        retries: 0,
        external_ref: "".to_string(),
        ratio: 0.0,
        status: BindingBatchStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_binding_batch("code-570", "name-872");
        assert_eq!(item.status, BindingBatchStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_binding_batch("code-570", "name-872");
        if item.move_to(BindingBatchStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingBatchStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_binding_batch("code-570", "name-872");
        item.status = BindingBatchStatus::Expired;
        assert!(
            item.move_to(BindingBatchStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
