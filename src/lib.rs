pub mod bindings;
mod calls;
mod cmds;
mod error;
mod planner;

pub use calls::FunctionCall;
pub use cmds::ReturnValue;
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
    ($planner:expr, $contract:expr, ( $call:expr ) ) => {{
        $crate::call_contract!(@dispatch call, $planner, $contract, ( $call ))
    }};

    (call, $planner:expr, $contract:expr, ( $call:expr ) ) => {{
        $crate::call_contract!(@dispatch call, $planner, $contract, ( $call ))
    }};

    (delegate, $planner:expr, $contract:expr, ( $call:expr ) ) => {{
        $crate::call_contract!(@dispatch delegatecall, $planner, $contract, ( $call ))
    }};

    (delegatecall, $planner:expr, $contract:expr, ( $call:expr ) ) => {{
        $crate::call_contract!(@dispatch delegatecall, $planner, $contract, ( $call ))
    }};

    (staticcall, $planner:expr, $contract:expr, ( $call:expr ) ) => {{
        $crate::call_contract!(@dispatch staticcall, $planner, $contract, ( $call ))
    }};

    (value($value:expr), $planner:expr, $contract:expr, ( $call:expr ) ) => {{
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
        let __value: ::alloy::primitives::U256 = ($value).into();
        __planner.call_with_value_address::<$call>(__address, __value, vec![$($arg.into(),)*])
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
        let __value: ::alloy::primitives::U256 = ($value).into();
        __planner.call_with_value_sol(__address, __value, $call)
    }};
}
