use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{Decimal, Uint128};

const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000u128);

/// return a / b
pub fn decimal_division(a: Uint128, b: Decimal) -> Uint128 {
    let decimal = Decimal::from_ratio(a, b * DECIMAL_FRACTIONAL);
    decimal * DECIMAL_FRACTIONAL
}

/// return a * b
pub fn _decimal_multiplication_in_256(a: Decimal, b: Decimal) -> Decimal {
    let a_u256: Decimal256 = a.into();
    let b_u256: Decimal256 = b.into();
    let c_u256: Decimal = (b_u256 * a_u256).into();
    c_u256
}

/// return a + b
pub fn _decimal_summation_in_256(a: Decimal, b: Decimal) -> Decimal {
    let a_u256: Decimal256 = a.into();
    let b_u256: Decimal256 = b.into();
    let c_u256: Decimal = (b_u256 + a_u256).into();
    c_u256
}

/// return a - b
pub fn _decimal_subtraction_in_256(a: Decimal, b: Decimal) -> Decimal {
    let a_u256: Decimal256 = a.into();
    let b_u256: Decimal256 = b.into();
    let c_u256: Decimal = (a_u256 - b_u256).into();
    c_u256
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal_division() {
        let a = Uint128::new(100);
        let b = Decimal::from_ratio(Uint128::new(10), Uint128::new(50));
        let res = decimal_division(a, b);
        assert_eq!(res, Uint128::new(500));
    }

    #[test]
    fn test_decimal_multiplication() {
        let a = Uint128::new(100);
        let b = Decimal::from_ratio(Uint128::new(1111111), Uint128::new(10000000));
        let multiplication =
            _decimal_multiplication_in_256(Decimal::from_ratio(a, Uint128::new(1)), b);
        assert_eq!(multiplication.to_string(), "11.11111");
    }

    #[test]
    fn test_decimal_sumation() {
        let a = Decimal::from_ratio(Uint128::new(20), Uint128::new(50));
        let b = Decimal::from_ratio(Uint128::new(10), Uint128::new(50));
        let res = _decimal_summation_in_256(a, b);
        assert_eq!(res.to_string(), "0.6");
    }

    #[test]
    fn test_decimal_subtraction() {
        let a = Decimal::from_ratio(Uint128::new(20), Uint128::new(50));
        let b = Decimal::from_ratio(Uint128::new(10), Uint128::new(50));
        let res = _decimal_subtraction_in_256(a, b);
        assert_eq!(res.to_string(), "0.2");
    }

    #[test]
    fn test_decimal_multiplication_in_256() {
        let a = Uint128::new(100);
        let b = Decimal::from_ratio(Uint128::new(1111111), Uint128::new(10000000));
        let multiplication =
            _decimal_multiplication_in_256(Decimal::from_ratio(a, Uint128::new(1)), b);
        assert_eq!(multiplication.to_string(), "11.11111");
    }

    #[test]
    fn test_decimal_sumation_in_256() {
        let a = Decimal::from_ratio(Uint128::new(20), Uint128::new(50));
        let b = Decimal::from_ratio(Uint128::new(10), Uint128::new(50));
        let res = _decimal_summation_in_256(a, b);
        assert_eq!(res.to_string(), "0.6");
    }

    #[test]
    fn test_decimal_subtraction_in_256() {
        let a = Decimal::from_ratio(Uint128::new(20), Uint128::new(50));
        let b = Decimal::from_ratio(Uint128::new(10), Uint128::new(50));
        let res = _decimal_subtraction_in_256(a, b);
        assert_eq!(res.to_string(), "0.2");
    }
}
