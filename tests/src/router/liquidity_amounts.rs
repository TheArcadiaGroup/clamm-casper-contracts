#[cfg(test)]
mod get_liquidity_for_amounts {
    use crate::{router::fixture::setup_fixture, utils::encode_price_sqrt};

    #[test]
    fn test_get_liquidity_for_amount0() {
        let mut tc = setup_fixture();
        let sqrt_price_x96 = encode_price_sqrt(1, 1);
        let sqrt_price_a_x96 = encode_price_sqrt(100, 110);
        let sqrt_price_b_x96 = encode_price_sqrt(110, 100);
        let liquidity = tc.get_liquidity_for_amounts(
            &sqrt_price_x96,
            &sqrt_price_a_x96,
            &sqrt_price_b_x96,
            &100.into(),
            &200.into(),
        );

        assert!(liquidity.as_u128() == 2148);
    }

    #[test]
    fn test_amounts_for_price_below() {
        let mut tc = setup_fixture();
        let sqrt_price_x96 = encode_price_sqrt(99, 110);
        let sqrt_price_a_x96 = encode_price_sqrt(100, 110);
        let sqrt_price_b_x96 = encode_price_sqrt(110, 100);
        let liquidity = tc.get_liquidity_for_amounts(
            &sqrt_price_x96,
            &sqrt_price_a_x96,
            &sqrt_price_b_x96,
            &100.into(),
            &200.into(),
        );

        assert!(liquidity.as_u128() == 1048);
    }

    #[test]
    fn test_amounts_for_price_above() {
        let mut tc = setup_fixture();
        let sqrt_price_x96 = encode_price_sqrt(111, 100);
        let sqrt_price_a_x96 = encode_price_sqrt(100, 110);
        let sqrt_price_b_x96 = encode_price_sqrt(110, 100);
        let liquidity = tc.get_liquidity_for_amounts(
            &sqrt_price_x96,
            &sqrt_price_a_x96,
            &sqrt_price_b_x96,
            &100.into(),
            &200.into(),
        );

        assert!(liquidity.as_u128() == 2097);
    }

    #[test]
    fn test_amounts_for_price_equal_to_lower_boundary() {
        let mut tc = setup_fixture();
        let sqrt_price_a_x96 = encode_price_sqrt(100, 110);
        let sqrt_price_x96 = sqrt_price_a_x96;
        let sqrt_price_b_x96 = encode_price_sqrt(110, 100);
        let liquidity = tc.get_liquidity_for_amounts(
            &sqrt_price_x96,
            &sqrt_price_a_x96,
            &sqrt_price_b_x96,
            &100.into(),
            &200.into(),
        );

        assert!(liquidity.as_u128() == 1048);
    }

    #[test]
    fn test_amounts_for_price_equal_to_upper_boundary() {
        let mut tc = setup_fixture();
        let sqrt_price_a_x96 = encode_price_sqrt(100, 110);
        let sqrt_price_b_x96 = encode_price_sqrt(110, 100);
        let sqrt_price_x96 = sqrt_price_b_x96;
        let liquidity = tc.get_liquidity_for_amounts(
            &sqrt_price_x96,
            &sqrt_price_a_x96,
            &sqrt_price_b_x96,
            &100.into(),
            &200.into(),
        );

        assert!(liquidity.as_u128() == 2097);
    }
}

#[cfg(test)]
mod get_amounts_for_liquidity {
    use crate::{router::fixture::setup_fixture, utils::encode_price_sqrt};

    #[test]
    fn test_amounts_for_price_inside() {
        let mut tc = setup_fixture();
        let sqrt_price_x96 = encode_price_sqrt(1, 1);
        let sqrt_price_a_x96 = encode_price_sqrt(100, 110);
        let sqrt_price_b_x96 = encode_price_sqrt(110, 100);
        let (amount0, amount1) = tc.get_amounts_for_liquidity(
            &sqrt_price_x96,
            &sqrt_price_a_x96,
            &sqrt_price_b_x96,
            2148,
        );

        assert!(amount0.as_u128() == 99);
        assert!(amount1.as_u128() == 99);
    }

    #[test]
    fn test_amounts_for_price_below() {
        let mut tc = setup_fixture();
        let sqrt_price_x96 = encode_price_sqrt(99, 110);
        let sqrt_price_a_x96 = encode_price_sqrt(100, 110);
        let sqrt_price_b_x96 = encode_price_sqrt(110, 100);
        let (amount0, amount1) = tc.get_amounts_for_liquidity(
            &sqrt_price_x96,
            &sqrt_price_a_x96,
            &sqrt_price_b_x96,
            1048,
        );

        assert!(amount0.as_u128() == 99);
        assert!(amount1.as_u128() == 0);
    }

    #[test]
    fn test_amounts_for_price_above() {
        let mut tc = setup_fixture();
        let sqrt_price_x96 = encode_price_sqrt(111, 100);
        let sqrt_price_a_x96 = encode_price_sqrt(100, 110);
        let sqrt_price_b_x96 = encode_price_sqrt(110, 100);
        let (amount0, amount1) = tc.get_amounts_for_liquidity(
            &sqrt_price_x96,
            &sqrt_price_a_x96,
            &sqrt_price_b_x96,
            2097,
        );

        assert!(amount0.as_u128() == 0);
        assert!(amount1.as_u128() == 199);
    }

    #[test]
    fn test_amounts_for_price_on_lower_boundary() {
        let mut tc = setup_fixture();
        let sqrt_price_a_x96 = encode_price_sqrt(100, 110);
        let sqrt_price_x96 = sqrt_price_a_x96;
        let sqrt_price_b_x96 = encode_price_sqrt(110, 100);
        let (amount0, amount1) = tc.get_amounts_for_liquidity(
            &sqrt_price_x96,
            &sqrt_price_a_x96,
            &sqrt_price_b_x96,
            1048,
        );

        assert!(amount0.as_u128() == 99);
        assert!(amount1.as_u128() == 0);
    }

    #[test]
    fn test_amounts_for_price_on_upper_boundary() {
        let mut tc = setup_fixture();
        let sqrt_price_a_x96 = encode_price_sqrt(100, 110);
        let sqrt_price_b_x96 = encode_price_sqrt(110, 100);
        let sqrt_price_x96 = sqrt_price_b_x96;
        let (amount0, amount1) = tc.get_amounts_for_liquidity(
            &sqrt_price_x96,
            &sqrt_price_a_x96,
            &sqrt_price_b_x96,
            2097,
        );

        assert!(amount0.as_u128() == 0);
        assert!(amount1.as_u128() == 199);
    }
}
