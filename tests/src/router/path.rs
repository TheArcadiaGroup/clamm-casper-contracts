#[cfg(test)]
mod path {
    use std::convert::TryInto;

    use casper_types::{bytesrepr::Bytes, runtime_args, Key, RuntimeArgs };

    use crate::{pool::fixture::FEE_MEDIUM, router::fixture::setup_fixture, utils::encode_path};

    fn token_addresses() -> Vec<Key> {
        vec![
            Key::Hash(TryInto::try_into(vec![1_u8; 32]).unwrap()),
            Key::Hash(TryInto::try_into(vec![2_u8; 32]).unwrap()),
            Key::Hash(TryInto::try_into(vec![3_u8; 32]).unwrap()),
        ]
    }

    fn fees() -> Vec<u32> {
        vec![FEE_MEDIUM, FEE_MEDIUM]
    }
    fn encoded_path() -> Bytes {
        encode_path(token_addresses(), fees())
    }

    #[test]
    fn test_works_on_first_pool() {
        let mut tc = setup_fixture();
        let has_multiple_pools_: bool = tc.test_env.call_view_function(
            tc.router,
            "has_multiple_pools",
            runtime_args! {
                "path" => encoded_path(),
            },
        );
        assert!(has_multiple_pools_ == true);

        let (token_a, token_b, fee): (Key, Key, u32) = tc.test_env.call_view_function(
            tc.router,
            "decode_first_pool",
            runtime_args! {
                "path" => encoded_path(),
            },
        );
        assert!(token_a == token_addresses()[0]);
        assert!(token_b == token_addresses()[1]);
        assert!(fee == FEE_MEDIUM);

        let first_pool_data: Bytes = tc.test_env.call_view_function(
            tc.router,
            "get_first_pool",
            runtime_args! {
                "path" => encoded_path(),
            },
        );

        let (token_a_2, token_b_2, fee_2): (Key, Key, u32) = tc.test_env.call_view_function(
            tc.router,
            "decode_first_pool",
            runtime_args! {
                "path" => first_pool_data,
            },
        );
        assert!(token_a == token_a_2);
        assert!(token_b == token_b_2);
        assert!(fee == fee_2);
    }

    #[test]
    fn test_skips_1_item() {
        let mut tc = setup_fixture();
        let skipped: Bytes = tc.test_env.call_view_function(
            tc.router,
            "skip_token",
            runtime_args! {
                "path" => encoded_path(),
            },
        );
        assert!(skipped == encoded_path()[36..].into());

        let has_multiple_pools_: bool = tc.test_env.call_view_function(
            tc.router,
            "has_multiple_pools",
            runtime_args! {
                "path" => skipped.clone(),
            },
        );
        assert!(has_multiple_pools_ == false);

        let (token_a, token_b, fee): (Key, Key, u32) = tc.test_env.call_view_function(
            tc.router,
            "decode_first_pool",
            runtime_args! {
                "path" => skipped,
            },
        );

        assert!(token_a == token_addresses()[1]);
        assert!(token_b == token_addresses()[2]);
        assert!(fee == FEE_MEDIUM);
    }
}