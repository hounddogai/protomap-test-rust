use super::*;

use acme_kit::text;

/// Where a verifier cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierCycleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl VerifierCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for VerifierCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One verifier cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierCycle {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub audience: String,
    pub description: String,
    pub source: String,
    pub pinned: bool,
    pub duration_minutes: i64,
    pub created_at: i64,
    pub status: VerifierCycleStatus,
}

impl VerifierCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierCycleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierCycleStatus) -> bool {
        match self.status {
            VerifierCycleStatus::Created => {
                match next {
                    VerifierCycleStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierCycleStatus::Validated => {
                match next {
                    VerifierCycleStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierCycleStatus::Published => {
                match next {
                    VerifierCycleStatus::Retired => {
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

    pub fn move_to(&mut self, next: VerifierCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierCycle {
        VerifierCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            audience: self.audience.clone(),
            description: self.description.clone(),
            source: self.source.clone(),
            pinned: self.pinned,
            duration_minutes: self.duration_minutes,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a verifier cycle in status created.
pub fn new_verifier_cycle(code: &str, label: &str) -> VerifierCycle {
    VerifierCycle {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        audience: "".to_string(),
        description: "".to_string(),
        source: "".to_string(),
        pinned: false,
        duration_minutes: 0,
        created_at: 0,
        status: VerifierCycleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_verifier_cycle("code-209", "label-335");
        assert_eq!(item.status, VerifierCycleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_verifier_cycle("code-209", "label-335");
        if item.move_to(VerifierCycleStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierCycleStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_verifier_cycle("code-209", "label-335");
        item.status = VerifierCycleStatus::Retired;
        assert!(
            item.move_to(VerifierCycleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
