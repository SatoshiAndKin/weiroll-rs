pub mod bindings;
mod calls;
mod cmds;
mod error;
mod planner;

pub use calls::FunctionCall;
pub use cmds::{CallValue, ReturnValue, Value};
pub use error::WeirollError;
pub use planner::Planner;

/// Plan a contract call into a [`Planner`].
///
/// This macro supports two syntaxes:
///
/// - `Contract::callName[args...]` (**values mode**): positional args, each coerced via `.into()`.
///   This is the mode you want when passing prior planner outputs like [`ReturnValue`].
/// - `Contract::callName { field: value, ... }` (**struct-literal mode**): expands to a real
///   `callName { ... }` struct literal and is fully type-checked, but cannot accept [`ReturnValue`]
///   fields.
#[macro_export]
macro_rules! call_contract {
    // ---- Public API: values mode (positional args) ----
    ($planner:expr, $contract:expr, $call:path [ $($arg:expr),* $(,)? ]) => {{
        $crate::call_contract!(@dispatch call, $planner, $contract, $call [ $($arg),* ])
    }};

    (call, $planner:expr, $contract:expr, $call:path [ $($arg:expr),* $(,)? ]) => {{
        $crate::call_contract!(@dispatch call, $planner, $contract, $call [ $($arg),* ])
    }};

    (delegate, $planner:expr, $contract:expr, $call:path [ $($arg:expr),* $(,)? ]) => {{
        $crate::call_contract!(@dispatch delegatecall, $planner, $contract, $call [ $($arg),* ])
    }};

    (delegatecall, $planner:expr, $contract:expr, $call:path [ $($arg:expr),* $(,)? ]) => {{
        $crate::call_contract!(@dispatch delegatecall, $planner, $contract, $call [ $($arg),* ])
    }};

    (staticcall, $planner:expr, $contract:expr, $call:path [ $($arg:expr),* $(,)? ]) => {{
        $crate::call_contract!(@dispatch staticcall, $planner, $contract, $call [ $($arg),* ])
    }};

    (value($value:expr), $planner:expr, $contract:expr, $call:path [ $($arg:expr),* $(,)? ]) => {{
        $crate::call_contract!(@dispatch value($value), $planner, $contract, $call [ $($arg),* ])
    }};

    // ---- Public API: SolCall mode (type-checked struct literal) ----
    ($planner:expr, $contract:expr, $call:expr) => {{
        $crate::call_contract!(@dispatch call, $planner, $contract, ( $call ))
    }};

    (call, $planner:expr, $contract:expr, $call:expr) => {{
        $crate::call_contract!(@dispatch call, $planner, $contract, ( $call ))
    }};

    (delegate, $planner:expr, $contract:expr, $call:expr) => {{
        $crate::call_contract!(@dispatch delegatecall, $planner, $contract, ( $call ))
    }};

    (delegatecall, $planner:expr, $contract:expr, $call:expr) => {{
        $crate::call_contract!(@dispatch delegatecall, $planner, $contract, ( $call ))
    }};

    (staticcall, $planner:expr, $contract:expr, $call:expr) => {{
        $crate::call_contract!(@dispatch staticcall, $planner, $contract, ( $call ))
    }};

    (value($value:expr), $planner:expr, $contract:expr, $call:expr) => {{
        $crate::call_contract!(@dispatch value($value), $planner, $contract, ( $call ))
    }};

    // ---- Internal implementation (ONLY these arms actually do work) ----
    (@dispatch call, $planner:expr, $contract:expr, $call:path [ $($arg:expr),* $(,)? ]) => {{
        let __planner = &mut *$planner;
        let __address = *$contract.address();
        __planner.call_address::<$call>(__address, vec![$($arg.into(),)*])
    }};

    (@dispatch delegatecall, $planner:expr, $contract:expr, $call:path [ $($arg:expr),* $(,)? ]) => {{
        let __planner = &mut *$planner;
        let __address = *$contract.address();
        __planner.delegatecall_address::<$call>(__address, vec![$($arg.into(),)*])
    }};

    (@dispatch staticcall, $planner:expr, $contract:expr, $call:path [ $($arg:expr),* $(,)? ]) => {{
        let __planner = &mut *$planner;
        let __address = *$contract.address();
        __planner.staticcall_address::<$call>(__address, vec![$($arg.into(),)*])
    }};

    (@dispatch value($value:expr), $planner:expr, $contract:expr, $call:path [ $($arg:expr),* $(,)? ]) => {{
        let __planner = &mut *$planner;
        let __address = *$contract.address();
        __planner.call_address_with_value::<$call>(__address, $value, vec![$($arg.into(),)*])
    }};

    (@dispatch call, $planner:expr, $contract:expr, ( $call:expr ) ) => {{
        let __planner = &mut *$planner;
        let __address = *$contract.address();
        __planner.call_sol(__address, $call)
    }};

    (@dispatch delegatecall, $planner:expr, $contract:expr, ( $call:expr ) ) => {{
        let __planner = &mut *$planner;
        let __address = *$contract.address();
        __planner.delegatecall_sol(__address, $call)
    }};

    (@dispatch staticcall, $planner:expr, $contract:expr, ( $call:expr ) ) => {{
        let __planner = &mut *$planner;
        let __address = *$contract.address();
        __planner.staticcall_sol(__address, $call)
    }};

    (@dispatch value($value:expr), $planner:expr, $contract:expr, ( $call:expr ) ) => {{
        let __planner = &mut *$planner;
        let __address = *$contract.address();
        __planner.call_sol_with_value(__address, $value, $call)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, U256, address};

    alloy::sol! {
        interface MacroTestContract {
            function setValue(uint256 value) external;
            function getValue() external returns (uint256);
        }
    }

    /// mock contract so we don't need to create a provider
    #[derive(Clone, Copy)]
    struct DummyContract {
        address: Address,
    }

    impl DummyContract {
        fn address(&self) -> &Address {
            &self.address
        }
    }

    #[test]
    fn accepts_calls_both_ways() {
        let mut planner = Planner::default();
        let contract = DummyContract {
            address: address!("0xdead00000000000000000000000000000000beef"),
        };

        crate::call_contract!(
            &mut planner,
            &contract,
            MacroTestContract::setValueCall {
                value: U256::from(1),
            }
        )
        .expect("macro should accept struct literal without extra parens");

        crate::call_contract!(
            &mut planner,
            &contract,
            MacroTestContract::setValueCall[1u64]
        )
        .expect("values mode should still work");

        let (commands, _state) = planner.plan().expect("plan");
        assert_eq!(commands.len(), 2);

        assert_eq!(commands[0], commands[1]);
    }

    #[test]
    fn value_macro_accepts_literals_and_return_values() {
        let mut planner = Planner::default();
        let contract = DummyContract {
            address: address!("0xdead00000000000000000000000000000000beef"),
        };

        crate::call_contract!(
            value(U256::from(1)),
            &mut planner,
            &contract,
            MacroTestContract::setValueCall[2u64]
        )
        .expect("values mode should accept literal call value");

        let value = crate::call_contract!(
            &mut planner,
            &contract,
            MacroTestContract::getValueCall[]
        )
        .expect("values mode should return a planner value");

        crate::call_contract!(
            value(value),
            &mut planner,
            &contract,
            MacroTestContract::setValueCall[4u64]
        )
        .expect("values mode should accept return-value call value");

        let (commands, _state) = planner.plan().expect("plan");
        assert_eq!(commands.len(), 3);
        assert_eq!(
            commands[2].as_slice()[4],
            crate::cmds::CommandFlags::CALL_WITH_VALUE.bits()
        );
    }
}
