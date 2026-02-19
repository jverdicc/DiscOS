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

use std::sync::Arc;

use tokio::sync::{mpsc, Mutex, Semaphore};
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KernelStatus {
    Healthy,
    Frozen,
}

#[derive(Debug, Clone)]
struct FakeKernel {
    state: Arc<Mutex<KernelState>>,
}

#[derive(Debug)]
struct KernelState {
    status: KernelStatus,
    processed: usize,
    max_inflight: usize,
    inflight: usize,
}

impl FakeKernel {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(KernelState {
                status: KernelStatus::Healthy,
                processed: 0,
                max_inflight: 0,
                inflight: 0,
            })),
        }
    }

    async fn set_status(&self, status: KernelStatus) {
        self.state.lock().await.status = status;
    }

    async fn submit_claim_capsule(&self, _capsule_id: u64) -> Result<(), ()> {
        {
            let mut state = self.state.lock().await;
            if state.status == KernelStatus::Frozen {
                return Err(());
            }
            state.inflight += 1;
            state.max_inflight = state.max_inflight.max(state.inflight);
        }

        sleep(Duration::from_micros(50)).await;

        let mut state = self.state.lock().await;
        state.inflight -= 1;
        state.processed += 1;
        Ok(())
    }

    async fn snapshot(&self) -> KernelStateSnapshot {
        let state = self.state.lock().await;
        KernelStateSnapshot {
            processed: state.processed,
            max_inflight: state.max_inflight,
        }
    }
}

#[derive(Debug)]
struct KernelStateSnapshot {
    processed: usize,
    max_inflight: usize,
}

#[derive(Debug)]
struct AgentRuntimeState {
    fail_safe: bool,
    session_generation: u64,
    frozen_session_generation: Option<u64>,
    last_frozen_claim: Option<u64>,
    max_buffered: usize,
}

#[derive(Debug, Clone)]
struct AgentRuntime {
    sender: mpsc::Sender<u64>,
    state: Arc<Mutex<AgentRuntimeState>>,
}

impl AgentRuntime {
    fn new(buffer: usize, worker_concurrency: usize, kernel: FakeKernel) -> Self {
        let (sender, mut receiver) = mpsc::channel::<u64>(buffer);
        let state = Arc::new(Mutex::new(AgentRuntimeState {
            fail_safe: false,
            session_generation: 1,
            frozen_session_generation: None,
            last_frozen_claim: None,
            max_buffered: 0,
        }));

        let state_for_worker = Arc::clone(&state);
        tokio::spawn(async move {
            let permits = Arc::new(Semaphore::new(worker_concurrency));
            let mut joins = Vec::new();

            while let Some(capsule_id) = receiver.recv().await {
                let permit = Arc::clone(&permits)
                    .acquire_owned()
                    .await
                    .expect("worker semaphore should remain available");
                let kernel_clone = kernel.clone();
                let state_clone = Arc::clone(&state_for_worker);
                joins.push(tokio::spawn(async move {
                    let submit_result = kernel_clone.submit_claim_capsule(capsule_id).await;
                    if submit_result.is_err() {
                        let mut state = state_clone.lock().await;
                        state.fail_safe = true;
                        state.frozen_session_generation = Some(state.session_generation);
                        state.last_frozen_claim = Some(capsule_id);
                    }
                    drop(permit);
                }));
            }

            for handle in joins {
                handle.await.expect("worker task must not panic");
            }
        });

        Self { sender, state }
    }

    async fn enqueue_claim_capsule(&self, claim_id: u64) {
        self.sender
            .send(claim_id)
            .await
            .expect("queue should stay alive while runtime exists");

        let buffered = self.sender.max_capacity() - self.sender.capacity();
        let mut state = self.state.lock().await;
        state.max_buffered = state.max_buffered.max(buffered);
    }

    async fn fail_safe(&self) -> bool {
        self.state.lock().await.fail_safe
    }

    async fn queue_peak(&self) -> usize {
        self.state.lock().await.max_buffered
    }

    async fn reauthenticate_after_ledger_reset(&self) {
        let mut state = self.state.lock().await;
        state.session_generation += 1;
        state.fail_safe = false;
        state.frozen_session_generation = None;
        state.last_frozen_claim = None;
    }

    async fn snapshot(&self) -> AgentRuntimeStateSnapshot {
        let state = self.state.lock().await;
        AgentRuntimeStateSnapshot {
            fail_safe: state.fail_safe,
            session_generation: state.session_generation,
            frozen_session_generation: state.frozen_session_generation,
            last_frozen_claim: state.last_frozen_claim,
        }
    }
}

#[derive(Debug)]
struct AgentRuntimeStateSnapshot {
    fail_safe: bool,
    session_generation: u64,
    frozen_session_generation: Option<u64>,
    last_frozen_claim: Option<u64>,
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn kernel_cutoff_frozen_status_enters_fail_safe() {
    let kernel = FakeKernel::new();
    kernel.set_status(KernelStatus::Frozen).await;
    let runtime = AgentRuntime::new(8, 2, kernel);

    runtime.enqueue_claim_capsule(42).await;
    sleep(Duration::from_millis(20)).await;

    let state = runtime.snapshot().await;
    assert!(state.fail_safe, "runtime should enter fail-safe mode");
    assert_eq!(state.frozen_session_generation, Some(1));
    assert_eq!(state.last_frozen_claim, Some(42));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn heavy_probing_claim_capsules_applies_backpressure_without_drops() {
    let kernel = FakeKernel::new();
    let runtime = AgentRuntime::new(64, 32, kernel.clone());

    let mut senders = Vec::with_capacity(5_000);
    for claim_id in 0_u64..5_000 {
        let runtime_clone = runtime.clone();
        senders.push(tokio::spawn(async move {
            runtime_clone.enqueue_claim_capsule(claim_id).await;
        }));
    }

    for sender in senders {
        sender.await.expect("sender task should not panic");
    }

    let queue_peak = runtime.queue_peak().await;
    drop(runtime);
    sleep(Duration::from_millis(250)).await;

    let kernel_state = kernel.snapshot().await;
    assert_eq!(
        kernel_state.processed, 5_000,
        "no claim capsule should be dropped"
    );
    assert!(
        kernel_state.max_inflight <= 32,
        "gRPC client concurrency must remain bounded"
    );

    assert!(
        queue_peak <= 64,
        "queue occupancy should be capped by channel capacity"
    );
    assert!(
        queue_peak > 0,
        "heavy probing should exercise queue backpressure"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reauthentication_clears_frozen_session_state_after_ledger_reset() {
    let kernel = FakeKernel::new();
    kernel.set_status(KernelStatus::Frozen).await;
    let runtime = AgentRuntime::new(4, 1, kernel.clone());

    runtime.enqueue_claim_capsule(7).await;
    sleep(Duration::from_millis(20)).await;
    assert!(
        runtime.fail_safe().await,
        "runtime should be frozen before reset"
    );

    kernel.set_status(KernelStatus::Healthy).await;
    runtime.reauthenticate_after_ledger_reset().await;

    let state = runtime.snapshot().await;
    assert!(
        !state.fail_safe,
        "fail-safe should be cleared after re-authentication"
    );
    assert_eq!(
        state.session_generation, 2,
        "session generation should rotate"
    );
    assert_eq!(
        state.frozen_session_generation, None,
        "frozen session marker should be dropped"
    );
    assert_eq!(
        state.last_frozen_claim, None,
        "frozen claim memory should be wiped"
    );

    runtime.enqueue_claim_capsule(8).await;
    sleep(Duration::from_millis(20)).await;

    let kernel_state = kernel.snapshot().await;
    assert_eq!(
        kernel_state.processed, 1,
        "post-reset session must operate independently"
    );
}
