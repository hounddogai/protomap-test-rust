use super::*;

use acme_kit::text;

/// Where a canary request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryRequestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CanaryRequestStatus {
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

impl std::fmt::Display for CanaryRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a canary request and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryRequest {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub currency: String,
    pub title: String,
    pub scope: String,
    pub rotation_days: i64,
    pub rate: f64,
    pub cost: f64,
    pub ceiling: f64,
    pub status: CanaryRequestStatus,
}

impl CanaryRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryRequestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryRequestStatus) -> bool {
        match self.status {
            CanaryRequestStatus::Draft => {
                match next {
                    CanaryRequestStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryRequestStatus::Pending => {
                return next == CanaryRequestStatus::Closed;
            }
            CanaryRequestStatus::Active => {
                return next == CanaryRequestStatus::Suspended;
            }
            CanaryRequestStatus::Suspended => {
                match next {
                    CanaryRequestStatus::Closed => {
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

    pub fn move_to(&mut self, next: CanaryRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.cost + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryRequest {
        CanaryRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            currency: self.currency.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            rotation_days: self.rotation_days,
            rate: self.rate,
            cost: self.cost,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a canary request in status draft.
pub fn new_canary_request(code: &str, name: &str) -> CanaryRequest {
    CanaryRequest {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        currency: "".to_string(),
        title: "".to_string(),
        scope: "".to_string(),
        rotation_days: 0,
        rate: 0.0,
        cost: 0.0,
        ceiling: 0.0,
        status: CanaryRequestStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_canary_request("code-454", "name-646");
        assert_eq!(item.status, CanaryRequestStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_canary_request("code-454", "name-646");
        if item.move_to(CanaryRequestStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryRequestStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_canary_request("code-454", "name-646");
        item.status = CanaryRequestStatus::Closed;
        assert!(
            item.move_to(CanaryRequestStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
