use super::*;

use acme_kit::text;

/// Where a probe adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeAdjustmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ProbeAdjustmentStatus {
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

impl std::fmt::Display for ProbeAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One probe adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeAdjustment {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub cipher: String,
    pub weight: f64,
    pub budget: f64,
    pub status: ProbeAdjustmentStatus,
}

impl ProbeAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeAdjustmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeAdjustmentStatus) -> bool {
        match self.status {
            ProbeAdjustmentStatus::Draft => {
                return next == ProbeAdjustmentStatus::Pending || next == ProbeAdjustmentStatus::Closed;
            }
            ProbeAdjustmentStatus::Pending => {
                match next {
                    ProbeAdjustmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeAdjustmentStatus::Active => {
                match next {
                    ProbeAdjustmentStatus::Suspended | ProbeAdjustmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeAdjustmentStatus::Suspended => {
                match next {
                    ProbeAdjustmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: ProbeAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "probe adjustment", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.budget - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeAdjustment {
        ProbeAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            cipher: self.cipher.clone(),
            weight: self.weight,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a probe adjustment in status draft.
pub fn new_probe_adjustment(code: &str, name: &str) -> ProbeAdjustment {
    ProbeAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        cipher: "".to_string(),
        weight: 0.0,
        budget: 0.0,
        status: ProbeAdjustmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_probe_adjustment("code-382", "name-914");
        assert_eq!(item.status, ProbeAdjustmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_probe_adjustment("code-382", "name-914");
        if item.move_to(ProbeAdjustmentStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeAdjustmentStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_probe_adjustment("code-382", "name-914");
        item.status = ProbeAdjustmentStatus::Closed;
        assert!(
            item.move_to(ProbeAdjustmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
