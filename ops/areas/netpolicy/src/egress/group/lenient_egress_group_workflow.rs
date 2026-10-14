use super::*;

use std::rc::Rc;

use acme_kit::clock;

/// Moves lenient egress groups through their life cycle and keeps a journal of what it did.
#[derive(Debug, Clone)]
pub struct LenientEgressGroupManager {
    pub store: Rc<dyn LenientEgressGroupStore>,
    pub validator: LenientEgressGroupValidator,
    pub clock: Rc<dyn clock::Clock>,
    pub journal: Vec<String>,
}

impl LenientEgressGroupManager {
    fn note(&mut self, message: &str) {
        self.journal.push(format!("{} {}", self.clock.now(), message));
    }

    pub fn advance(&mut self, code: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("lenient egress group {} not found", code));
        };
        self.validator
            .require_valid(&item)
            .map_err(|err| format!("validate: {err}"))?;
        item.move_to(LenientEgressGroupStatus::Queued)
            .map_err(|err| format!("move: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is queued", code));
        Ok(())
    }

    pub fn finish(&mut self, code: &str, reason: &str) -> Result<(), String> {
        let Some(mut item) = self.store.find(code) else {
            return Err(format!("lenient egress group {} not found", code));
        };
        if item.is_terminal() {
            self.note(&format!("{} already closed", code));
            return Ok(());
        }
        item.move_to(LenientEgressGroupStatus::Failed)
            .map_err(|err| format!("close: {err}"))?;
        self.store.save(&item);
        self.note(&format!("{} is failed: {}", code, reason));
        Ok(())
    }

    pub fn scan(&mut self, cutoff: i64) -> i64 {
        let mut changed = 0;
        for item in self.store.list() {
            if item.is_terminal() {
                continue;
            }
            let problems = self.validator.validate(&item);
            if (problems.len() as i64) == 0 {
                continue;
            }
            self.note(&format!("{} needs attention: {}", item.code, problems.join(", ")));
            changed += 1;
        }
        changed
    }
}

pub fn new_lenient_egress_group_manager(
    store: Rc<dyn LenientEgressGroupStore>,
    clock: Rc<dyn clock::Clock>,
) -> LenientEgressGroupManager {
    LenientEgressGroupManager {
        store: store.clone(),
        validator: default_lenient_egress_group_validator(),
        clock: clock.clone(),
        journal: Vec::new(),
    }
}
