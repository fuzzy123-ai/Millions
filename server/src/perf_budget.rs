#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerPerfBudgets {
    pub server_tick_p99_ms_max: f64,
    pub sim_1k_p95_ms_max: f64,
    pub sim_5k_p95_ms_max: f64,
    pub sim_10k_p95_ms_max: f64,
    pub normal_aoi_bandwidth_kb_s_p95_max: f64,
    pub stress_10k_bandwidth_kb_s_p95_max: f64,
    pub reconnect_full_snapshot_p95_s_max: f64,
}

impl Default for ServerPerfBudgets {
    fn default() -> Self {
        Self {
            server_tick_p99_ms_max: 50.0,
            sim_1k_p95_ms_max: 10.0,
            sim_5k_p95_ms_max: 20.0,
            sim_10k_p95_ms_max: 35.0,
            normal_aoi_bandwidth_kb_s_p95_max: 256.0,
            stress_10k_bandwidth_kb_s_p95_max: 768.0,
            reconnect_full_snapshot_p95_s_max: 3.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetResult {
    Pass,
    Fail,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerPerfReport {
    pub server_tick_p99_ms: Option<f64>,
    pub sim_p95_ms: Option<f64>,
    pub bandwidth_kb_s_per_client_p95: Option<f64>,
    pub reconnect_full_snapshot_p95_s: Option<f64>,
}

impl ServerPerfReport {
    pub fn server_tick(server_tick_p99_ms: f64) -> Self {
        Self {
            server_tick_p99_ms: Some(server_tick_p99_ms),
            sim_p95_ms: None,
            bandwidth_kb_s_per_client_p95: None,
            reconnect_full_snapshot_p95_s: None,
        }
    }

    pub fn sim_scale(server_tick_p99_ms: f64, sim_p95_ms: f64, bandwidth_p95: f64) -> Self {
        Self {
            server_tick_p99_ms: Some(server_tick_p99_ms),
            sim_p95_ms: Some(sim_p95_ms),
            bandwidth_kb_s_per_client_p95: Some(bandwidth_p95),
            reconnect_full_snapshot_p95_s: None,
        }
    }

    pub fn reconnect(server_tick_p99_ms: f64, reconnect_full_snapshot_p95_s: f64) -> Self {
        Self {
            server_tick_p99_ms: Some(server_tick_p99_ms),
            sim_p95_ms: None,
            bandwidth_kb_s_per_client_p95: None,
            reconnect_full_snapshot_p95_s: Some(reconnect_full_snapshot_p95_s),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerPerfScenario {
    ServerIdle20Hz,
    Sim1kSingleClient,
    Sim5kSingleClient,
    Sim10kSingleClient,
    ReconnectFullSnapshot1k,
}

pub fn evaluate_server_perf_budget(
    scenario: ServerPerfScenario,
    report: ServerPerfReport,
    budgets: ServerPerfBudgets,
) -> BudgetResult {
    match scenario {
        ServerPerfScenario::ServerIdle20Hz => {
            evaluate_required(report.server_tick_p99_ms, budgets.server_tick_p99_ms_max)
        }
        ServerPerfScenario::Sim1kSingleClient => evaluate_sim_budget(
            report,
            budgets,
            budgets.sim_1k_p95_ms_max,
            budgets.normal_aoi_bandwidth_kb_s_p95_max,
        ),
        ServerPerfScenario::Sim5kSingleClient => evaluate_sim_budget(
            report,
            budgets,
            budgets.sim_5k_p95_ms_max,
            budgets.normal_aoi_bandwidth_kb_s_p95_max,
        ),
        ServerPerfScenario::Sim10kSingleClient => evaluate_sim_budget(
            report,
            budgets,
            budgets.sim_10k_p95_ms_max,
            budgets.stress_10k_bandwidth_kb_s_p95_max,
        ),
        ServerPerfScenario::ReconnectFullSnapshot1k => {
            let tick = evaluate_required(report.server_tick_p99_ms, budgets.server_tick_p99_ms_max);
            let reconnect = evaluate_required(
                report.reconnect_full_snapshot_p95_s,
                budgets.reconnect_full_snapshot_p95_s_max,
            );
            combine_results([tick, reconnect])
        }
    }
}

fn evaluate_sim_budget(
    report: ServerPerfReport,
    budgets: ServerPerfBudgets,
    sim_p95_ms_max: f64,
    bandwidth_p95_max: f64,
) -> BudgetResult {
    combine_results([
        evaluate_required(report.server_tick_p99_ms, budgets.server_tick_p99_ms_max),
        evaluate_required(report.sim_p95_ms, sim_p95_ms_max),
        evaluate_required(report.bandwidth_kb_s_per_client_p95, bandwidth_p95_max),
    ])
}

fn evaluate_required(value: Option<f64>, max: f64) -> BudgetResult {
    match value {
        None => BudgetResult::Blocked,
        Some(value) if value <= max => BudgetResult::Pass,
        Some(_) => BudgetResult::Fail,
    }
}

fn combine_results(results: impl IntoIterator<Item = BudgetResult>) -> BudgetResult {
    let mut saw_blocked = false;
    for result in results {
        match result {
            BudgetResult::Fail => return BudgetResult::Fail,
            BudgetResult::Blocked => saw_blocked = true,
            BudgetResult::Pass => {}
        }
    }

    if saw_blocked {
        BudgetResult::Blocked
    } else {
        BudgetResult::Pass
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation_scale::{run_simulation_scale_scenario, simulation_scale_scenarios};

    #[test]
    fn server_tick_report_passes_when_within_budget() {
        let result = evaluate_server_perf_budget(
            ServerPerfScenario::ServerIdle20Hz,
            ServerPerfReport::server_tick(49.0),
            ServerPerfBudgets::default(),
        );

        assert_eq!(result, BudgetResult::Pass);
    }

    #[test]
    fn missing_required_metric_blocks_claim() {
        let result = evaluate_server_perf_budget(
            ServerPerfScenario::Sim1kSingleClient,
            ServerPerfReport::server_tick(49.0),
            ServerPerfBudgets::default(),
        );

        assert_eq!(result, BudgetResult::Blocked);
    }

    #[test]
    fn sim_report_fails_when_bandwidth_exceeds_budget() {
        let result = evaluate_server_perf_budget(
            ServerPerfScenario::Sim5kSingleClient,
            ServerPerfReport::sim_scale(49.0, 15.0, 300.0),
            ServerPerfBudgets::default(),
        );

        assert_eq!(result, BudgetResult::Fail);
    }

    #[test]
    fn reconnect_report_checks_tick_and_restore_time() {
        let pass = evaluate_server_perf_budget(
            ServerPerfScenario::ReconnectFullSnapshot1k,
            ServerPerfReport::reconnect(49.0, 2.5),
            ServerPerfBudgets::default(),
        );
        let fail = evaluate_server_perf_budget(
            ServerPerfScenario::ReconnectFullSnapshot1k,
            ServerPerfReport::reconnect(49.0, 3.5),
            ServerPerfBudgets::default(),
        );

        assert_eq!(pass, BudgetResult::Pass);
        assert_eq!(fail, BudgetResult::Fail);
    }

    #[test]
    fn simulation_scale_scenarios_feed_budget_scenario_surface() {
        for scenario in simulation_scale_scenarios() {
            let run = run_simulation_scale_scenario(*scenario);
            let result = evaluate_server_perf_budget(
                scenario.budget_scenario(),
                ServerPerfReport::sim_scale(50.0, 0.0, 0.0),
                ServerPerfBudgets::default(),
            );

            assert_eq!(run.entity_count, scenario.entity_count);
            assert_eq!(run.tick_count, scenario.tick_count);
            assert_eq!(run.snapshot_entities, scenario.entity_count);
            assert!(run.occupied_cells > 0);
            assert_eq!(result, BudgetResult::Pass);
        }
    }
}
