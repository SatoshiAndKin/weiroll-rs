use alloy::{
    dyn_abi::DynSolType,
    primitives::{Address, U256},
};
// use ethers::{abi::ParamType, prelude::*};

use crate::cmds::{CommandFlags, Value};

#[derive(Debug)]
pub struct FunctionCall<'a> {
    pub(crate) address: Address,
    pub(crate) selector: [u8; 4],
    pub(crate) flags: CommandFlags,
    pub(crate) value: Option<U256>,
    pub(crate) args: Vec<Value<'a>>,
    pub(crate) return_type: DynSolType,
}

impl FunctionCall<'_> {
    #[allow(dead_code)]
    pub fn with_value(mut self, value: U256) -> Self {
        self.flags = (self.flags & !CommandFlags::CALLTYPE_MASK) | CommandFlags::CALL_WITH_VALUE;
        self.value = Some(value);
        self
    }

    #[allow(dead_code)]
    pub fn raw_value(mut self) -> Self {
        self.flags |= CommandFlags::TUPLE_RETURN;
        self
    }

    #[allow(dead_code)]
    pub fn static_call(mut self) -> Self {
        if (self.flags & CommandFlags::CALLTYPE_MASK) != CommandFlags::CALL {
            panic!("Only CALL operations can be made static");
        }
        self.flags = (self.flags & !CommandFlags::CALLTYPE_MASK) | CommandFlags::STATICCALL;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::dyn_abi::DynSolType;
    use alloy::primitives::{U256, address};

    fn sample_call() -> FunctionCall<'static> {
        FunctionCall {
            address: address!("0x0000000000000000000000000000000000000001"),
            selector: [0u8; 4],
            flags: CommandFlags::CALL,
            value: Some(U256::ZERO),
            args: vec![],
            return_type: DynSolType::Uint(256),
        }
    }

    #[test]
    fn function_call_with_value_sets_call_with_value_flag() {
        let c = sample_call().with_value(U256::from(123));
        assert_eq!(
            c.flags & CommandFlags::CALLTYPE_MASK,
            CommandFlags::CALL_WITH_VALUE
        );
        assert_eq!(c.value, Some(U256::from(123)));
    }

    #[test]
    fn function_call_raw_value_sets_tuple_return() {
        let c = sample_call().raw_value();
        assert!(c.flags.contains(CommandFlags::TUPLE_RETURN));
    }

    #[test]
    fn function_call_static_call_switches_to_staticcall() {
        let c = sample_call().static_call();
        assert_eq!(
            c.flags & CommandFlags::CALLTYPE_MASK,
            CommandFlags::STATICCALL
        );
    }
}

// impl<M: Middleware, D: Detokenize> From<ContractCall<M, D>> for FunctionCall {
//     fn from(call: ContractCall<M, D>) -> Self {
//         let args = Vec::new();
//         Self {
//             contract: *call.tx.to_addr().unwrap(),
//             flags: CommandFlags::empty(),
//             args,
//             value: call.tx.value().cloned(),
//         }
//     }
// }
