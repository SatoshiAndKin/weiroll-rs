pub mod bindings;
mod calls;
mod cmds;
mod error;
mod planner;

pub use error::WeirollError;
pub use planner::Planner;

#[macro_export]
macro_rules! call_contract {
    // Values mode: positional args turned into `Value`s in order.
    ($planner:expr, $contract:expr, $call:path [ $($arg:expr),* $(,)? ] ) => {{
        let __planner = &mut *$planner;
        let __address = *$contract.address();
        __planner.call_address::<$call>(__address, vec![$($arg.into(),)*])
    }};

    (call, $planner:expr, $contract:expr, $call:path [ $($arg:expr),* $(,)? ] ) => {{
        let __planner = &mut *$planner;
        let __address = *$contract.address();
        __planner.call_address::<$call>(__address, vec![$($arg.into(),)*])
    }};

    (delegatecall, $planner:expr, $contract:expr, $call:path [ $($arg:expr),* $(,)? ] ) => {{
        let __planner = &mut *$planner;
        let __address = *$contract.address();
        __planner.delegatecall_address::<$call>(__address, vec![$($arg.into(),)*])
    }};

    (staticcall, $planner:expr, $contract:expr, $call:path [ $($arg:expr),* $(,)? ] ) => {{
        let __planner = &mut *$planner;
        let __address = *$contract.address();
        __planner.staticcall_address::<$call>(__address, vec![$($arg.into(),)*])
    }};

    // SolCall mode: pass an actual generated call struct (type-checked by Rust).
    ($planner:expr, $contract:expr, ( $call:expr ) ) => {{
        let __planner = &mut *$planner;
        let __address = *$contract.address();
        __planner.call_sol(__address, $call)
    }};

    (call, $planner:expr, $contract:expr, ( $call:expr ) ) => {{
        let __planner = &mut *$planner;
        let __address = *$contract.address();
        __planner.call_sol(__address, $call)
    }};

    (delegatecall, $planner:expr, $contract:expr, ( $call:expr ) ) => {{
        let __planner = &mut *$planner;
        let __address = *$contract.address();
        __planner.delegatecall_sol(__address, $call)
    }};

    (staticcall, $planner:expr, $contract:expr, ( $call:expr ) ) => {{
        let __planner = &mut *$planner;
        let __address = *$contract.address();
        __planner.staticcall_sol(__address, $call)
    }};
}
