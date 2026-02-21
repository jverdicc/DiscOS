// Copyright 2026 Joseph Verdicchio
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::topicid::CANONICAL_OUTPUT_SCHEMA_ID;

pub const CBRN_SC_V1: &str = CANONICAL_OUTPUT_SCHEMA_ID;
pub const HEAVY_LANE: &str = "heavy";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DualUsePolicyConfig {
    pub require_structured_outputs: bool,
    pub deny_free_text_outputs: bool,
    pub force_heavy_lane_on_domain: Vec<String>,
    pub reject_on_high_risk_schema_mismatch: bool,
    pub production_mode: bool,
}

impl Default for DualUsePolicyConfig {
    fn default() -> Self {
        Self {
            require_structured_outputs: true,
            deny_free_text_outputs: true,
            force_heavy_lane_on_domain: vec!["CBRN".to_string()],
            reject_on_high_risk_schema_mismatch: true,
            production_mode: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimSafetyContext<'a> {
    pub domain: &'a str,
    pub lane: &'a str,
    pub output_schema_id: &'a str,
    pub requests_free_text_output: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnforcementDecision {
    Allow,
    ForceHeavyLane { required_lane: &'static str },
    Reject { reason: String },
}

fn domain_matches(candidate: &str, configured: &str) -> bool {
    candidate.eq_ignore_ascii_case(configured)
}

fn is_forced_heavy_domain(domain: &str, cfg: &DualUsePolicyConfig) -> bool {
    cfg.force_heavy_lane_on_domain
        .iter()
        .any(|configured| domain_matches(domain, configured))
}

pub fn enforce_dual_use_policy(
    cfg: &DualUsePolicyConfig,
    ctx: &ClaimSafetyContext<'_>,
) -> EnforcementDecision {
    let high_risk_domain = is_forced_heavy_domain(ctx.domain, cfg);

    if cfg.production_mode && cfg.deny_free_text_outputs && ctx.requests_free_text_output {
        return EnforcementDecision::Reject {
            reason: "free-text outputs are disabled in production".to_string(),
        };
    }

    if high_risk_domain {
        if cfg.require_structured_outputs && ctx.output_schema_id.trim().is_empty() {
            return EnforcementDecision::Reject {
                reason: "high-risk domains require structured outputs".to_string(),
            };
        }

        if !ctx.output_schema_id.eq_ignore_ascii_case(CBRN_SC_V1) {
            return if cfg.reject_on_high_risk_schema_mismatch {
                EnforcementDecision::Reject {
                    reason: format!(
                        "high-risk domain `{}` requires output schema `{}`",
                        ctx.domain, CBRN_SC_V1
                    ),
                }
            } else {
                EnforcementDecision::ForceHeavyLane {
                    required_lane: HEAVY_LANE,
                }
            };
        }

        if !ctx.lane.eq_ignore_ascii_case(HEAVY_LANE) {
            return EnforcementDecision::ForceHeavyLane {
                required_lane: HEAVY_LANE,
            };
        }
    }

    EnforcementDecision::Allow
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_fail_closed_for_high_risk() {
        let cfg = DualUsePolicyConfig::default();
        assert!(cfg.require_structured_outputs);
        assert!(cfg.deny_free_text_outputs);
        assert_eq!(cfg.force_heavy_lane_on_domain, vec!["CBRN"]);
        assert!(cfg.reject_on_high_risk_schema_mismatch);
        assert!(cfg.production_mode);
    }

    #[test]
    fn high_risk_schema_mismatch_rejected() {
        let cfg = DualUsePolicyConfig::default();
        let decision = enforce_dual_use_policy(
            &cfg,
            &ClaimSafetyContext {
                domain: "CBRN",
                lane: "heavy",
                output_schema_id: "other-schema.v9",
                requests_free_text_output: false,
            },
        );
        assert!(matches!(decision, EnforcementDecision::Reject { .. }));
    }

    #[test]
    fn high_risk_non_heavy_lane_is_forced_to_heavy() {
        let cfg = DualUsePolicyConfig::default();
        let decision = enforce_dual_use_policy(
            &cfg,
            &ClaimSafetyContext {
                domain: "CBRN",
                lane: "fast",
                output_schema_id: CBRN_SC_V1,
                requests_free_text_output: false,
            },
        );
        assert_eq!(
            decision,
            EnforcementDecision::ForceHeavyLane {
                required_lane: HEAVY_LANE
            }
        );
    }

    #[test]
    fn free_text_disabled_in_production() {
        let cfg = DualUsePolicyConfig::default();
        let decision = enforce_dual_use_policy(
            &cfg,
            &ClaimSafetyContext {
                domain: "general",
                lane: "standard",
                output_schema_id: "",
                requests_free_text_output: true,
            },
        );
        assert!(matches!(decision, EnforcementDecision::Reject { .. }));
    }

    #[test]
    fn non_high_risk_structured_claim_allowed() {
        let cfg = DualUsePolicyConfig::default();
        let decision = enforce_dual_use_policy(
            &cfg,
            &ClaimSafetyContext {
                domain: "BENIGN",
                lane: "standard",
                output_schema_id: "summary.v1",
                requests_free_text_output: false,
            },
        );
        assert_eq!(decision, EnforcementDecision::Allow);
    }
}
