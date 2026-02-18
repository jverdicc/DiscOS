use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopperAttempt {
    pub candidate_id: String,
    pub proxy_passed: bool,
    pub submitted: bool,
    pub certified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PopperReport {
    pub attempts: Vec<PopperAttempt>,
}

pub fn run_minimal_popper(candidates: &[String]) -> PopperReport {
    let attempts = candidates
        .iter()
        .map(|c| {
            let proxy_passed = c.len() % 2 == 0;
            let submitted = proxy_passed;
            let certified = submitted && c.as_bytes().iter().fold(0u8, |acc, b| acc ^ b) % 2 == 0;
            PopperAttempt {
                candidate_id: c.clone(),
                proxy_passed,
                submitted,
                certified,
            }
        })
        .collect();
    PopperReport { attempts }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_loop() {
        let c = vec!["a".to_string(), "bb".to_string()];
        let r1 = run_minimal_popper(&c);
        let r2 = run_minimal_popper(&c);
        assert_eq!(
            serde_json::to_string(&r1).unwrap(),
            serde_json::to_string(&r2).unwrap()
        );
    }
}
