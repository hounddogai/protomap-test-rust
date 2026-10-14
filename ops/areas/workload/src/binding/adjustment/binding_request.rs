use super::*;

use acme_kit::text;

/// Where a binding request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingRequestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BindingRequestStatus {
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

impl std::fmt::Display for BindingRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding request as the binding team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingRequest {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub rate: f64,
    pub limit: i64,
    pub threshold: f64,
    pub ends_at: i64,
    pub window_days: i64,
    pub weight: f64,
    pub batch_size: i64,
    pub updated_at: i64,
    pub status: BindingRequestStatus,
}

impl BindingRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingRequestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingRequestStatus) -> bool {
        match self.status {
            BindingRequestStatus::Draft => {
                match next {
                    BindingRequestStatus::Suspended | BindingRequestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingRequestStatus::Pending => {
                return next == BindingRequestStatus::Suspended;
            }
            BindingRequestStatus::Active => {
                match next {
                    BindingRequestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingRequestStatus::Suspended => {
                match next {
                    BindingRequestStatus::Closed => {
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

    pub fn move_to(&mut self, next: BindingRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding request {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.rate - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingRequest {
        BindingRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            rate: self.rate,
            limit: self.limit,
            threshold: self.threshold,
            ends_at: self.ends_at,
            window_days: self.window_days,
            weight: self.weight,
            batch_size: self.batch_size,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a binding request in status draft.
pub fn new_binding_request(code: &str, title: &str) -> BindingRequest {
    BindingRequest {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        rate: 0.0,
        limit: 0,
        threshold: 0.0,
        ends_at: 0,
        window_days: 0,
        weight: 0.0,
        batch_size: 0,
        updated_at: 0,
        status: BindingRequestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_binding_request("code-452", "title-209");
        assert_eq!(item.status, BindingRequestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_binding_request("code-452", "title-209");
        if item.move_to(BindingRequestStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingRequestStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_binding_request("code-452", "title-209");
        item.status = BindingRequestStatus::Closed;
        assert!(
            item.move_to(BindingRequestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
