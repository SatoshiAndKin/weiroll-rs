pub mod bindings;
mod calls;
mod cmds;
mod error;
mod planner;

pub use error::WeirollError;
pub use planner::Planner;

#[macro_export]
macro_rules! call_contract {
    ($planner:expr, $contract:expr, $call:path { $($field:ident : $value:expr),+ $(,)? }) => {{
        let __planner = &mut *$planner;
        let __address = *$contract.address();
        __planner.call_address::<$call>(__address, vec![$($value.into(),)+])
    }};
    ($planner:expr, $contract:expr, $call:path { $(,)? }) => {{
        let __planner = &mut *$planner;
        let __address = *$contract.address();
        __planner.call_address::<$call>(__address, vec![])
    }};
}
