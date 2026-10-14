use super::*;

use acme_kit::text;

/// Where a rolling sink mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingSinkMappingStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingSinkMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingSinkMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling sink mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSinkMapping {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub external_ref: String,
    pub timezone: String,
    pub threshold: f64,
    pub cipher: String,
    pub cost: f64,
    pub principal: String,
    pub status: RollingSinkMappingStatus,
}

impl RollingSinkMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingSinkMappingStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingSinkMappingStatus) -> bool {
        match self.status {
            RollingSinkMappingStatus::Requested => {
                return next == RollingSinkMappingStatus::Granted || next == RollingSinkMappingStatus::Revoked;
            }
            RollingSinkMappingStatus::Granted => {
                match next {
                    RollingSinkMappingStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingSinkMappingStatus::Revoked => {
                return next == RollingSinkMappingStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingSinkMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling sink mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling sink mapping", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.threshold * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingSinkMapping {
        RollingSinkMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            external_ref: self.external_ref.clone(),
            timezone: self.timezone.clone(),
            threshold: self.threshold,
            cipher: self.cipher.clone(),
            cost: self.cost,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rolling sink mapping in status requested.
pub fn new_rolling_sink_mapping(code: &str, label: &str) -> RollingSinkMapping {
    RollingSinkMapping {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        external_ref: "".to_string(),
        timezone: "".to_string(),
        threshold: 0.0,
        cipher: "".to_string(),
        cost: 0.0,
        principal: "".to_string(),
        status: RollingSinkMappingStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rolling_sink_mapping("code-218", "label-984");
        assert_eq!(item.status, RollingSinkMappingStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_rolling_sink_mapping("code-218", "label-984");
        if item.move_to(RollingSinkMappingStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingSinkMappingStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rolling_sink_mapping("code-218", "label-984");
        item.status = RollingSinkMappingStatus::Expired;
        assert!(
            item.move_to(RollingSinkMappingStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
