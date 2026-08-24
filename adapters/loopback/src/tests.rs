use synapseflow_domain::execution::{
    FrameEnvelope, FrameMessageType, FrameProtocolVersion, FrameSequence, FrameTarget,
    InFlightFrameLimit, RemainingDeadline, SessionId, StreamId,
};
use synapseflow_domain::{
    DomainError, ErrorCode, ModelReference, ShardId, TensorDescriptor, TensorDtype,
};
use synapseflow_ports::WorkerId;

use crate::{LoopbackEvent, LoopbackFault, LoopbackNetwork};

fn workers() -> (WorkerId, WorkerId, LoopbackNetwork) {
    let first = WorkerId::new("loopback-a".to_owned()).expect("first worker is valid");
    let second = WorkerId::new("loopback-b".to_owned()).expect("second worker is valid");
    let network = LoopbackNetwork::new(
        InFlightFrameLimit::new(4).expect("queue limit is valid"),
        vec![first.clone(), second.clone()],
    )
    .expect("network should construct");
    (first, second, network)
}

fn data_frame(sequence: FrameSequence) -> FrameEnvelope {
    FrameEnvelope::new(
        FrameProtocolVersion::current(),
        FrameMessageType::Data,
        SessionId::new("session-00000001".to_owned()).expect("session is valid"),
        StreamId::new(1).expect("stream is valid"),
        sequence,
        FrameTarget {
            model: ModelReference::parse(format!(
                "registry://fixtures/tinyllama@sha256:{}",
                "a".repeat(64)
            ))
            .expect("model reference is valid"),
            shard: ShardId::new("first".to_owned()).expect("shard is valid"),
        },
        Some(TensorDescriptor::new(TensorDtype::F32, vec![1]).expect("tensor is valid")),
        RemainingDeadline::new(std::time::Duration::from_millis(500)).expect("deadline is valid"),
    )
    .expect("frame is valid")
}

#[test]
fn two_workers_exchange_codec_validated_data_ack_and_nack_frames() {
    let (first_id, second_id, network) = workers();
    let first = network.worker(&first_id).expect("first worker exists");
    let second = network.worker(&second_id).expect("second worker exists");
    let frame = data_frame(FrameSequence::initial());
    let next = data_frame(
        FrameSequence::initial()
            .next()
            .expect("next sequence should exist"),
    );

    first
        .send(&second_id, &frame, &[0, 0, 0, 0], None)
        .expect("data frame should send");
    first
        .send(&second_id, &next, &[0, 0, 0, 0], None)
        .expect("next data frame should send");
    let received = second
        .receive()
        .expect("receive should succeed")
        .expect("data frame should arrive");
    assert_eq!(received.source, first_id);
    assert_eq!(received.frame.envelope.message_type, FrameMessageType::Data);
    assert_eq!(received.frame.envelope.sequence, FrameSequence::initial());
    assert_eq!(
        second
            .receive()
            .expect("ordered receive should succeed")
            .expect("next data should arrive")
            .frame
            .envelope
            .sequence
            .value(),
        1
    );

    second
        .acknowledge(&first_id, &received.frame.envelope)
        .expect("ack should encode and send");
    assert_eq!(
        first
            .receive()
            .expect("ack receive should succeed")
            .expect("ack should arrive")
            .frame
            .envelope
            .message_type,
        FrameMessageType::Ack
    );

    second
        .reject(
            &first_id,
            &received.frame.envelope,
            ErrorCode::FrameIntegrity,
        )
        .expect("nack should encode and send");
    let nack = first
        .receive()
        .expect("nack receive should succeed")
        .expect("nack should arrive");
    assert_eq!(nack.frame.envelope.message_type, FrameMessageType::Nack);
    assert_eq!(nack.frame.extensions().len(), 1);
    assert_eq!(nack.frame.extensions()[0].tag(), 1);
    assert_eq!(nack.frame.extensions()[0].value(), b"SYN-FRAME-004");
    assert!(matches!(
        network
            .transport()
            .events()
            .expect("events should be available")
            .as_slice(),
        [LoopbackEvent::NackSent {
            reason: ErrorCode::FrameIntegrity,
            ..
        }]
    ));
}

#[test]
fn deterministically_injects_delay_timeout_dropped_ack_corruption_and_worker_failures() {
    let (first_id, second_id, network) = workers();
    let first = network.worker(&first_id).expect("first worker exists");
    let second = network.worker(&second_id).expect("second worker exists");
    let frame = data_frame(FrameSequence::initial());

    network
        .inject(LoopbackFault::DelayNext {
            destination: second_id.clone(),
            polls: 1,
        })
        .expect("delay fault should configure");
    first
        .send(&second_id, &frame, &[0, 0, 0, 0], None)
        .expect("delayed data should send");
    assert!(second
        .receive()
        .expect("delayed receive should succeed")
        .is_none());
    let delayed = second
        .receive()
        .expect("data should arrive after delay")
        .expect("delayed data should arrive");
    network
        .inject(LoopbackFault::DropNextAck {
            destination: first_id.clone(),
        })
        .expect("dropped ack fault should configure");
    second
        .acknowledge(&first_id, &delayed.frame.envelope)
        .expect("ack should be accepted for deterministic dropping");
    assert!(first
        .receive()
        .expect("dropped ack poll should succeed")
        .is_none());

    network
        .inject(LoopbackFault::TimeoutNext {
            destination: second_id.clone(),
        })
        .expect("timeout fault should configure");
    first
        .send(&second_id, &frame, &[0, 0, 0, 0], None)
        .expect("timed out data should send");
    assert!(matches!(
        second.receive(),
        Err(DomainError::DeadlineExceeded)
    ));

    network
        .inject(LoopbackFault::CorruptNextFrame {
            destination: second_id.clone(),
        })
        .expect("corruption fault should configure");
    first
        .send(&second_id, &frame, &[0, 0, 0, 0], None)
        .expect("corrupted data should enqueue");
    assert!(matches!(second.receive(), Err(DomainError::FrameIntegrity)));

    network
        .inject(LoopbackFault::Unavailable {
            worker: second_id.clone(),
            enabled: true,
        })
        .expect("availability fault should configure");
    assert!(matches!(
        first.send(&second_id, &frame, &[0, 0, 0, 0], None),
        Err(DomainError::WorkerUnavailable)
    ));
    network
        .inject(LoopbackFault::Unavailable {
            worker: second_id.clone(),
            enabled: false,
        })
        .expect("availability fault should clear");
    network
        .inject(LoopbackFault::FailNextSend {
            source: first_id.clone(),
        })
        .expect("worker failure should configure");
    assert!(matches!(
        first.send(&second_id, &frame, &[0, 0, 0, 0], None),
        Err(DomainError::WorkerUnavailable)
    ));
}

#[test]
fn cancellation_removes_queued_session_work_and_delivers_a_codec_cancel_frame() {
    let (first_id, second_id, network) = workers();
    let first = network.worker(&first_id).expect("first worker exists");
    let second = network.worker(&second_id).expect("second worker exists");
    let initial = data_frame(FrameSequence::initial());
    let next = data_frame(
        FrameSequence::initial()
            .next()
            .expect("next sequence should exist"),
    );

    first
        .send(&second_id, &initial, &[0, 0, 0, 0], None)
        .expect("initial data should send");
    first
        .send(&second_id, &next, &[0, 0, 0, 0], None)
        .expect("next data should send");
    first
        .cancel(&second_id, &initial)
        .expect("cancel should purge and send control frame");
    assert_eq!(
        second
            .receive()
            .expect("cancel receive should succeed")
            .expect("cancel should arrive")
            .frame
            .envelope
            .message_type,
        FrameMessageType::Cancel
    );
    assert!(second.receive().expect("queue should be empty").is_none());
    second.shutdown().expect("worker shutdown should succeed");
    assert!(matches!(
        first.send(&second_id, &initial, &[0, 0, 0, 0], None),
        Err(DomainError::WorkerUnavailable)
    ));
}

#[test]
fn bounded_queues_apply_backpressure_before_enqueuing_more_work() {
    let first_id = WorkerId::new("loopback-a".to_owned()).expect("first worker is valid");
    let second_id = WorkerId::new("loopback-b".to_owned()).expect("second worker is valid");
    let network = LoopbackNetwork::new(
        InFlightFrameLimit::new(1).expect("queue limit is valid"),
        vec![first_id.clone(), second_id.clone()],
    )
    .expect("network should construct");
    let first = network.worker(&first_id).expect("first worker exists");
    let frame = data_frame(FrameSequence::initial());

    first
        .send(&second_id, &frame, &[0, 0, 0, 0], None)
        .expect("first frame should fit");
    assert!(matches!(
        first.send(&second_id, &frame, &[0, 0, 0, 0], None),
        Err(DomainError::FrameBoundsExceeded)
    ));
}
