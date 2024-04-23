// Copyright (c) 2024 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#[generic_tests::define]
mod sample_mut {
    use iceoryx2::port::publisher::{Publisher, PublisherLoanError};
    use iceoryx2::port::subscriber::Subscriber;
    use iceoryx2::prelude::*;
    use iceoryx2::service::builder::publish_subscribe::PublishSubscribeCreateError;
    use iceoryx2::service::port_factory::publish_subscribe::PortFactory;
    use iceoryx2::service::Service;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;

    const MAX_LOANED_SAMPLES: usize = 5;

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "service_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    struct TestContext<Sut: Service> {
        service_name: ServiceName,
        service: PortFactory<Sut, u64>,
        publisher: Publisher<Sut, u64>,
        subscriber: Subscriber<Sut, u64>,
    }

    impl<Sut: Service> TestContext<Sut> {
        fn new() -> Self {
            let service_name = generate_name();
            let service = Sut::new(&service_name)
                .publish_subscribe()
                .max_publishers(1)
                .typed::<u64>()
                .create()
                .unwrap();

            let publisher = service
                .publisher()
                .max_loaned_samples(MAX_LOANED_SAMPLES)
                .create()
                .unwrap();

            let subscriber = service.subscriber().create().unwrap();

            Self {
                service_name,
                service,
                publisher,
                subscriber,
            }
        }
    }

    #[test]
    fn when_going_out_of_scope_it_is_released<Sut: Service>() {
        let test_context = TestContext::<Sut>::new();

        let mut sample_vec = vec![];

        for _ in 0..4 {
            while let Ok(sample) = test_context.publisher.loan() {
                sample_vec.push(sample);
            }

            assert_that!(sample_vec, len MAX_LOANED_SAMPLES);

            let loan_result = test_context.publisher.loan();
            assert_that!(loan_result, is_err);
            assert_that!(loan_result.err().unwrap(), eq PublisherLoanError::ExceedsMaxLoanedChunks);

            sample_vec.clear();

            assert_that!(test_context.publisher.loan(), is_ok);
        }
    }

    #[test]
    fn header_tracks_correct_origin<Sut: Service>() {
        let test_context = TestContext::<Sut>::new();
        let sample = test_context.publisher.loan().unwrap();
        assert_that!(sample.header().publisher_id(), eq test_context.publisher.id());
    }

    #[test]
    fn write_payload_works<Sut: Service>() {
        const PAYLOAD_1: u64 = 891283689123555;
        const PAYLOAD_2: u64 = 71820;
        let test_context = TestContext::<Sut>::new();
        let sample = test_context.publisher.loan_uninit().unwrap();
        let mut sample = sample.write_payload(PAYLOAD_1);

        assert_that!(*sample.payload(), eq PAYLOAD_1);
        assert_that!(*sample.payload_mut(), eq PAYLOAD_1);

        *sample.payload_mut() = PAYLOAD_2;

        assert_that!(*sample.payload(), eq PAYLOAD_2);
        assert_that!(*sample.payload_mut(), eq PAYLOAD_2);
    }

    #[test]
    fn assume_init_works<Sut: Service>() {
        const PAYLOAD: u64 = 7182055123;
        let test_context = TestContext::<Sut>::new();
        let mut sample = test_context.publisher.loan_uninit().unwrap();
        let _ = *sample.payload_mut().write(PAYLOAD);
        let mut sample = unsafe { sample.assume_init() };

        assert_that!(*sample.payload(), eq PAYLOAD);
        assert_that!(*sample.payload_mut(), eq PAYLOAD);
    }

    #[test]
    fn send_works<Sut: Service>() {
        const PAYLOAD: u64 = 3215357;
        let test_context = TestContext::<Sut>::new();
        let sample = test_context.publisher.loan_uninit().unwrap();
        let sample = sample.write_payload(PAYLOAD);

        assert_that!(sample.send(), eq Ok(1));

        let received_sample = test_context.subscriber.receive().unwrap().unwrap();
        assert_that!(*received_sample, eq PAYLOAD);
    }

    #[test]
    fn sample_of_dropped_service_does_block_new_service_creation<Sut: Service>() {
        let test_context = TestContext::<Sut>::new();
        let service_name = test_context.service_name.clone();
        let _sample = test_context.publisher.loan_uninit().unwrap();

        drop(test_context);

        let result = Sut::new(&service_name)
            .publish_subscribe()
            .typed::<u64>()
            .create();
        assert_that!(result, is_err);
        assert_that!(result.err().unwrap(), eq PublishSubscribeCreateError::OldConnectionsStillActive);
    }

    #[test]
    fn sample_of_dropped_publisher_does_not_block_new_publishers<Sut: Service>() {
        let test_context = TestContext::<Sut>::new();
        let service = test_context.service;
        let publisher = test_context.publisher;
        let _sample = publisher.loan_uninit().unwrap();

        drop(publisher);

        assert_that!(service.publisher().create(), is_ok);
    }

    #[instantiate_tests(<iceoryx2::service::zero_copy::Service>)]
    mod zero_copy {}

    #[instantiate_tests(<iceoryx2::service::process_local::Service>)]
    mod process_local {}
}
