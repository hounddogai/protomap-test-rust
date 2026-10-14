use super::*;

use acme_kit::text;

/// Where a batched egress request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedEgressRequestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedEgressRequestStatus {
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

impl std::fmt::Display for BatchedEgressRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched egress request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEgressRequest {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub weight: f64,
    pub timezone: String,
    pub maximum: f64,
    pub reviewer: String,
    pub updated_at: i64,
    pub principal: String,
    pub status: BatchedEgressRequestStatus,
}

impl BatchedEgressRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedEgressRequestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedEgressRequestStatus) -> bool {
        match self.status {
            BatchedEgressRequestStatus::Draft => {
                return next == BatchedEgressRequestStatus::Pending || next == BatchedEgressRequestStatus::Closed;
            }
            BatchedEgressRequestStatus::Pending => {
                return next == BatchedEgressRequestStatus::Closed;
            }
            BatchedEgressRequestStatus::Active => {
                return next == BatchedEgressRequestStatus::Suspended || next == BatchedEgressRequestStatus::Closed;
            }
            BatchedEgressRequestStatus::Suspended => {
                match next {
                    BatchedEgressRequestStatus::Closed => {
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

    pub fn move_to(&mut self, next: BatchedEgressRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched egress request {} cannot move from {} to {}",
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
        format!("{} {}", "batched egress request", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.maximum * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedEgressRequest {
        BatchedEgressRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            weight: self.weight,
            timezone: self.timezone.clone(),
            maximum: self.maximum,
            reviewer: self.reviewer.clone(),
            updated_at: self.updated_at,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched egress request in status draft.
pub fn new_batched_egress_request(code: &str, title: &str) -> BatchedEgressRequest {
    BatchedEgressRequest {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        weight: 0.0,
        timezone: "".to_string(),
        maximum: 0.0,
        reviewer: "".to_string(),
        updated_at: 0,
        principal: "".to_string(),
        status: BatchedEgressRequestStatus::Draft,
    }
}
