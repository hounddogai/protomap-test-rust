use super::*;

use acme_kit::text;

/// Where a probe target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeTargetStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ProbeTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ProbeTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe target as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeTarget {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub description: String,
    pub maximum: f64,
    pub starts_at: i64,
    pub status: ProbeTargetStatus,
}

impl ProbeTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeTargetStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeTargetStatus) -> bool {
        match self.status {
            ProbeTargetStatus::Proposed => {
                match next {
                    ProbeTargetStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeTargetStatus::Approved => {
                match next {
                    ProbeTargetStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeTargetStatus::Rejected => {
                match next {
                    ProbeTargetStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ProbeTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.priority as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeTarget {
        ProbeTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            description: self.description.clone(),
            maximum: self.maximum,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a probe target in status proposed.
pub fn new_probe_target(code: &str, title: &str) -> ProbeTarget {
    ProbeTarget {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        description: "".to_string(),
        maximum: 0.0,
        starts_at: 0,
        status: ProbeTargetStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_probe_target("code-440", "title-154");
        assert_eq!(item.status, ProbeTargetStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_probe_target("code-440", "title-154");
        if item.move_to(ProbeTargetStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeTargetStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_probe_target("code-440", "title-154");
        item.status = ProbeTargetStatus::Withdrawn;
        assert!(
            item.move_to(ProbeTargetStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
