/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_s3::config::Builder;
use aws_sdk_s3::types::ChecksumAlgorithm;
use aws_sdk_s3::{Client, Config};
use aws_smithy_runtime::client::http::test_util::dvr::ReplayingClient;
use aws_smithy_runtime::client::http::test_util::{ReplayEvent, StaticReplayClient};
use aws_smithy_runtime::test_util::capture_test_logs::capture_test_logs;
use aws_smithy_types::body::SdkBody;

async fn test_client<F>(update_builder: F) -> Client
where
    F: Fn(Builder) -> Builder,
{
    let sdk_config = aws_config::from_env()
        .no_credentials()
        .region("us-west-2")
        .load()
        .await;
    let config = Config::from(&sdk_config).to_builder().with_test_defaults();
    aws_sdk_s3::Client::from_conf(update_builder(config).build())
}

#[tokio::test]
async fn list_objects_v2() {
    let _logs = capture_test_logs();

    let http_client =
        ReplayingClient::from_file("tests/data/express/list-objects-v2.json").unwrap();
    let client = test_client(|b| b.http_client(http_client.clone())).await;

    let result = client
        .list_objects_v2()
        .bucket("s3express-test-bucket--usw2-az1--x-s3")
        .send()
        .await;
    dbg!(result).expect("success");

    http_client
        .validate_body_and_headers(Some(&["x-amz-s3session-token"]), "application/xml")
        .await
        .unwrap();
}

#[tokio::test]
async fn list_objects_v2_in_both_express_and_regular_buckets() {
    let _logs = capture_test_logs();

    let http_client = ReplayingClient::from_file(
        "tests/data/express/list_objects_v2_in_both_express_and_regular_buckets.json",
    )
    .unwrap();
    let client = test_client(|b| b.http_client(http_client.clone())).await;

    // A call to an S3 Express bucket where we should see two request/response pairs,
    // one for the `create_session` API and the other for `list_objects_v2` in S3 Express bucket.
    let result = client
        .list_objects_v2()
        .bucket("s3express-test-bucket--usw2-az1--x-s3")
        .send()
        .await;
    dbg!(result).expect("success");

    // A call to a regular bucket, and request headers should not contain `x-amz-s3session-token`.
    let result = client
        .list_objects_v2()
        .bucket("regular-test-bucket")
        .send()
        .await;
    dbg!(result).expect("success");

    // A call to another S3 Express bucket where we should again see two request/response pairs,
    // one for the `create_session` API and the other for `list_objects_v2` in S3 Express bucket.
    let result = client
        .list_objects_v2()
        .bucket("s3express-test-bucket-2--usw2-az3--x-s3")
        .send()
        .await;
    dbg!(result).expect("success");

    // This call should be an identity cache hit for the first S3 Express bucket,
    // thus no HTTP request should be sent to the `create_session` API.
    let result = client
        .list_objects_v2()
        .bucket("s3express-test-bucket--usw2-az1--x-s3")
        .send()
        .await;
    dbg!(result).expect("success");

    http_client
        .validate_body_and_headers(Some(&["x-amz-s3session-token"]), "application/xml")
        .await
        .unwrap();
}

fn create_session_request() -> http::Request<SdkBody> {
    http::Request::builder()
        .uri("https://s3express-test-bucket--usw2-az1--x-s3.s3express-usw2-az1.us-west-2.amazonaws.com/?session")
        .header("x-amz-create-session-mode", "ReadWrite")
        .method("GET")
        .body(SdkBody::empty())
        .unwrap()
}

fn create_session_response() -> http::Response<SdkBody> {
    http::Response::builder()
        .status(200)
        .body(SdkBody::from(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <CreateSessionResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
                <Credentials>
                    <SessionToken>TESTSESSIONTOKEN</SessionToken>
                    <SecretAccessKey>TESTSECRETKEY</SecretAccessKey>
                    <AccessKeyId>ASIARTESTID</AccessKeyId>
                    <Expiration>2024-01-29T18:53:01Z</Expiration>
                </Credentials>
            </CreateSessionResult>
            "#,
        ))
        .unwrap()
}

fn operation_request_with_checksum(
    query: &str,
    kv: Option<(&str, &str)>,
) -> http::Request<SdkBody> {
    let mut b = http::Request::builder()
        .uri(&format!("https://s3express-test-bucket--usw2-az1--x-s3.s3express-usw2-az1.us-west-2.amazonaws.com/{query}"))
        .method("GET");
    if let Some((key, value)) = kv {
        b = b.header(key, value);
    }
    b.body(SdkBody::empty()).unwrap()
}

fn response_ok() -> http::Response<SdkBody> {
    http::Response::builder()
        .status(200)
        .body(SdkBody::empty())
        .unwrap()
}

#[tokio::test]
async fn user_specified_checksum_should_be_respected() {
    async fn runner(checksum: &str, value: &str) {
        let http_client = StaticReplayClient::new(vec![
            ReplayEvent::new(create_session_request(), create_session_response()),
            ReplayEvent::new(
                operation_request_with_checksum(
                    "test?x-id=PutObject",
                    Some((
                        &format!("x-amz-checksum-{}", checksum.to_lowercase()),
                        &format!("{value}"),
                    )),
                ),
                response_ok(),
            ),
        ]);
        let client = test_client(|b| b.http_client(http_client.clone())).await;

        let _ = client
            .put_object()
            .bucket("s3express-test-bucket--usw2-az1--x-s3")
            .key("test")
            .body(SdkBody::empty().into())
            .checksum_algorithm(ChecksumAlgorithm::from(checksum))
            .send()
            .await;

        http_client.assert_requests_match(&[""]);
    }

    let crc32_checksum = "AAAAAA==";
    let crc32c_checksum = "AAAAAA==";
    let sha1_checksum = "2jmj7l5rSw0yVb/vlWAYkK/YBwk=";
    let sha256_checksum = "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=";
    for (checksum, value) in ChecksumAlgorithm::values().iter().zip(&[
        crc32_checksum,
        crc32c_checksum,
        sha1_checksum,
        sha256_checksum,
    ]) {
        runner(*checksum, *value).await;
    }
}

#[tokio::test]
async fn default_checksum_should_be_crc32_for_operation_requiring_checksum() {
    let http_client = StaticReplayClient::new(vec![
        ReplayEvent::new(create_session_request(), create_session_response()),
        ReplayEvent::new(
            operation_request_with_checksum(
                "?delete&x-id=DeleteObjects",
                Some(("x-amz-checksum-crc32", "AAAAAA==")),
            ),
            response_ok(),
        ),
    ]);
    let client = test_client(|b| b.http_client(http_client.clone())).await;

    let _ = client
        .delete_objects()
        .bucket("s3express-test-bucket--usw2-az1--x-s3")
        .send()
        .await;

    http_client.assert_requests_match(&[""]);
}

#[tokio::test]
async fn default_checksum_should_be_none() {
    let http_client = StaticReplayClient::new(vec![
        ReplayEvent::new(create_session_request(), create_session_response()),
        ReplayEvent::new(
            operation_request_with_checksum("test?x-id=PutObject", None),
            response_ok(),
        ),
    ]);
    let client = test_client(|b| b.http_client(http_client.clone())).await;

    let _ = client
        .put_object()
        .bucket("s3express-test-bucket--usw2-az1--x-s3")
        .key("test")
        .body(SdkBody::empty().into())
        .send()
        .await;

    http_client.assert_requests_match(&[""]);

    let mut all_checksums = ChecksumAlgorithm::values()
        .iter()
        .map(|checksum| format!("amz-checksum-{}", checksum.to_lowercase()))
        .chain(std::iter::once("content-md5".to_string()));

    assert!(!all_checksums.any(|checksum| http_client
        .actual_requests()
        .any(|req| req.headers().iter().any(|(key, _)| key == checksum))));
}
