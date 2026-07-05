#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObservabilityCounter {
    ConnectionsActive,
    CommandsPending,
    CommandsAccepted,
    CommandsRejected,
    SnapshotsBuilt,
    SnapshotsDropped,
    VisibleEntities,
    RenderProxyCount,
    BackpressureEvents,
    RedactionEvents,
}

impl ObservabilityCounter {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ConnectionsActive => "connections_active",
            Self::CommandsPending => "commands_pending",
            Self::CommandsAccepted => "commands_accepted",
            Self::CommandsRejected => "commands_rejected",
            Self::SnapshotsBuilt => "snapshots_built",
            Self::SnapshotsDropped => "snapshots_dropped",
            Self::VisibleEntities => "visible_entities",
            Self::RenderProxyCount => "render_proxy_count",
            Self::BackpressureEvents => "backpressure_events",
            Self::RedactionEvents => "redaction_events",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CounterSample {
    pub counter: ObservabilityCounter,
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservabilityCounters {
    values: Vec<CounterSample>,
}

impl ObservabilityCounters {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn set(&mut self, counter: ObservabilityCounter, value: u64) {
        if let Some(sample) = self
            .values
            .iter_mut()
            .find(|sample| sample.counter == counter)
        {
            sample.value = value;
            return;
        }

        self.values.push(CounterSample { counter, value });
        self.values.sort_by_key(|sample| sample.counter);
    }

    pub fn increment(&mut self, counter: ObservabilityCounter) -> u64 {
        self.add(counter, 1)
    }

    pub fn add(&mut self, counter: ObservabilityCounter, delta: u64) -> u64 {
        let next = self.get(counter).saturating_add(delta);
        self.set(counter, next);
        next
    }

    pub fn get(&self, counter: ObservabilityCounter) -> u64 {
        self.values
            .iter()
            .find(|sample| sample.counter == counter)
            .map(|sample| sample.value)
            .unwrap_or(0)
    }

    pub fn snapshot(&self) -> Vec<CounterSample> {
        self.values.clone()
    }

    pub fn snapshot_pairs(&self) -> Vec<(&'static str, u64)> {
        self.values
            .iter()
            .map(|sample| (sample.counter.as_str(), sample.value))
            .collect()
    }
}

impl Default for ObservabilityCounters {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_default_to_zero_and_increment() {
        let mut counters = ObservabilityCounters::new();

        assert_eq!(counters.get(ObservabilityCounter::CommandsAccepted), 0);
        assert_eq!(
            counters.increment(ObservabilityCounter::CommandsAccepted),
            1
        );
        assert_eq!(counters.add(ObservabilityCounter::CommandsAccepted, 2), 3);
        assert_eq!(counters.get(ObservabilityCounter::CommandsAccepted), 3);
    }

    #[test]
    fn counter_snapshot_is_stable_by_counter_order() {
        let mut counters = ObservabilityCounters::new();
        counters.set(ObservabilityCounter::SnapshotsBuilt, 4);
        counters.set(ObservabilityCounter::ConnectionsActive, 2);

        assert_eq!(
            counters.snapshot_pairs(),
            vec![("connections_active", 2), ("snapshots_built", 4)]
        );
    }

    #[test]
    fn counter_catalog_uses_log_schema_names() {
        let expected = [
            "connections_active",
            "commands_pending",
            "commands_accepted",
            "commands_rejected",
            "snapshots_built",
            "snapshots_dropped",
            "visible_entities",
            "render_proxy_count",
            "backpressure_events",
            "redaction_events",
        ];
        let actual = [
            ObservabilityCounter::ConnectionsActive.as_str(),
            ObservabilityCounter::CommandsPending.as_str(),
            ObservabilityCounter::CommandsAccepted.as_str(),
            ObservabilityCounter::CommandsRejected.as_str(),
            ObservabilityCounter::SnapshotsBuilt.as_str(),
            ObservabilityCounter::SnapshotsDropped.as_str(),
            ObservabilityCounter::VisibleEntities.as_str(),
            ObservabilityCounter::RenderProxyCount.as_str(),
            ObservabilityCounter::BackpressureEvents.as_str(),
            ObservabilityCounter::RedactionEvents.as_str(),
        ];

        assert_eq!(actual, expected);
    }
}
