use super::*;

use acme_kit::text;

/// Where a limit mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitMappingStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LimitMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LimitMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One limit mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitMapping {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub sequence: i64,
    pub window_days: i64,
    pub approved: bool,
    pub budget: f64,
    pub retries: i64,
    pub status: LimitMappingStatus,
}

impl LimitMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitMappingStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitMappingStatus) -> bool {
        match self.status {
            LimitMappingStatus::Requested => {
                match next {
                    LimitMappingStatus::Revoked | LimitMappingStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitMappingStatus::Granted => {
                return next == LimitMappingStatus::Revoked || next == LimitMappingStatus::Expired;
            }
            LimitMappingStatus::Revoked => {
                match next {
                    LimitMappingStatus::Expired => {
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

    pub fn move_to(&mut self, next: LimitMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "limit mapping", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.window_days as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitMapping {
        LimitMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            sequence: self.sequence,
            window_days: self.window_days,
            approved: self.approved,
            budget: self.budget,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a limit mapping in status requested.
pub fn new_limit_mapping(code: &str, label: &str) -> LimitMapping {
    LimitMapping {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        sequence: 0,
        window_days: 0,
        approved: false,
        budget: 0.0,
        retries: 0,
        status: LimitMappingStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_limit_mapping("code-455", "label-549");
        assert_eq!(item.status, LimitMappingStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_limit_mapping("code-455", "label-549");
        if item.move_to(LimitMappingStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitMappingStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_limit_mapping("code-455", "label-549");
        item.status = LimitMappingStatus::Expired;
        assert!(
            item.move_to(LimitMappingStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
