use super::*;

use acme_kit::text;

/// Where a verifier assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierAssignmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl VerifierAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for VerifierAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a verifier assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAssignment {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub updated_at: i64,
    pub discount: f64,
    pub maximum: f64,
    pub ratio: f64,
    pub host: String,
    pub status: VerifierAssignmentStatus,
}

impl VerifierAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierAssignmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierAssignmentStatus) -> bool {
        match self.status {
            VerifierAssignmentStatus::Created => {
                match next {
                    VerifierAssignmentStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierAssignmentStatus::Validated => {
                return next == VerifierAssignmentStatus::Retired;
            }
            VerifierAssignmentStatus::Published => {
                match next {
                    VerifierAssignmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: VerifierAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier assignment {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "verifier assignment", self.code)
    }

    pub fn net_load(&self) -> f64 {
        (self.maximum * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierAssignment {
        VerifierAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            updated_at: self.updated_at,
            discount: self.discount,
            maximum: self.maximum,
            ratio: self.ratio,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a verifier assignment in status created.
pub fn new_verifier_assignment(code: &str, label: &str) -> VerifierAssignment {
    VerifierAssignment {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        updated_at: 0,
        discount: 0.0,
        maximum: 0.0,
        ratio: 0.0,
        host: "".to_string(),
        status: VerifierAssignmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_verifier_assignment("code-802", "label-796");
        assert_eq!(item.status, VerifierAssignmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_verifier_assignment("code-802", "label-796");
        if item.move_to(VerifierAssignmentStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierAssignmentStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_verifier_assignment("code-802", "label-796");
        item.status = VerifierAssignmentStatus::Retired;
        assert!(
            item.move_to(VerifierAssignmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
