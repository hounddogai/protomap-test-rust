use super::*;

use acme_kit::text;

/// Where a slo plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloPlanStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SloPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SloPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo plan as the slo team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloPlan {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub sequence: i64,
    pub ends_at: i64,
    pub ceiling: f64,
    pub principal: String,
    pub status: SloPlanStatus,
}

impl SloPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloPlanStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloPlanStatus) -> bool {
        match self.status {
            SloPlanStatus::Requested => {
                return next == SloPlanStatus::Granted || next == SloPlanStatus::Expired;
            }
            SloPlanStatus::Granted => {
                match next {
                    SloPlanStatus::Revoked | SloPlanStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloPlanStatus::Revoked => {
                match next {
                    SloPlanStatus::Expired => {
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

    pub fn move_to(&mut self, next: SloPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo plan {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.ends_at as f64) * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloPlan {
        SloPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            sequence: self.sequence,
            ends_at: self.ends_at,
            ceiling: self.ceiling,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a slo plan in status requested.
pub fn new_slo_plan(code: &str, label: &str) -> SloPlan {
    SloPlan {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        sequence: 0,
        ends_at: 0,
        ceiling: 0.0,
        principal: "".to_string(),
        status: SloPlanStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_slo_plan("code-320", "label-767");
        assert_eq!(item.status, SloPlanStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_slo_plan("code-320", "label-767");
        if item.move_to(SloPlanStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SloPlanStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_slo_plan("code-320", "label-767");
        item.status = SloPlanStatus::Expired;
        assert!(item.move_to(SloPlanStatus::Requested).is_err(), "expected move to fail");
    }
}
