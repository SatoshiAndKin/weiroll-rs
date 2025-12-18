use crate::calls::FunctionCall;
use crate::cmds::{Command, CommandFlags, CommandType, Literal, ReturnValue, Value};
use crate::error::WeirollError;

use alloy::dyn_abi::DynSolType;
use alloy::dyn_abi::DynSolValue;
use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use alloy::sol_types::{SolCall, SolType};
use bytes::BufMut;
use bytes::BytesMut;
#[allow(deprecated)]
use slotmap::{DefaultKey, HopSlotMap};
use std::collections::{BTreeMap, BTreeSet};

type CommandKey = DefaultKey;

#[derive(Debug, Default)]
pub struct Planner<'a> {
    #[allow(deprecated)]
    commands: HopSlotMap<CommandKey, Command<'a>>,
}

#[derive(Debug, Default)]
pub struct PlannerState {
    return_slot_map: BTreeMap<CommandKey, u8>,
    literal_slot_map: BTreeMap<Literal, u8>,
    free_slots: Vec<u8>,
    state_expirations: BTreeMap<CommandKey, Vec<u8>>,
    command_visibility: BTreeMap<CommandKey, CommandKey>,
    state: Vec<Bytes>,
}

impl<'a> Planner<'a> {
    pub fn call_sol<C>(&mut self, address: Address, call: C) -> Result<ReturnValue, WeirollError>
    where
        C: SolCall,
    {
        let params_type: DynSolType = <C::Parameters<'_> as SolType>::SOL_NAME.parse()?;

        let mut encoded_args = Vec::new();
        <C as SolCall>::abi_encode_raw(&call, &mut encoded_args);

        let decoded = match params_type {
            DynSolType::Tuple(_) => params_type.abi_decode_sequence(&encoded_args)?,
            other => other.abi_decode(&encoded_args)?,
        };

        let values: Vec<DynSolValue> = match decoded {
            DynSolValue::Tuple(v) => v,
            v => vec![v],
        };

        let args = values
            .into_iter()
            .map(|v| Value::Literal(Literal::from(v)))
            .collect();

        self.call_address::<C>(address, args)
    }

    pub fn delegatecall_sol<C>(
        &mut self,
        address: Address,
        call: C,
    ) -> Result<ReturnValue, WeirollError>
    where
        C: SolCall,
    {
        self.call_sol_with_calltype(address, call, CommandFlags::DELEGATECALL)
    }

    pub fn staticcall_sol<C>(
        &mut self,
        address: Address,
        call: C,
    ) -> Result<ReturnValue, WeirollError>
    where
        C: SolCall,
    {
        self.call_sol_with_calltype(address, call, CommandFlags::STATICCALL)
    }

    pub fn call_sol_with_value<C>(
        &mut self,
        address: Address,
        value: U256,
        call: C,
    ) -> Result<ReturnValue, WeirollError>
    where
        C: SolCall,
    {
        let params_type: DynSolType = <C::Parameters<'_> as SolType>::SOL_NAME.parse()?;

        let mut encoded_args = Vec::new();
        <C as SolCall>::abi_encode_raw(&call, &mut encoded_args);

        let decoded = match params_type {
            DynSolType::Tuple(_) => params_type.abi_decode_sequence(&encoded_args)?,
            other => other.abi_decode(&encoded_args)?,
        };

        let values: Vec<DynSolValue> = match decoded {
            DynSolValue::Tuple(v) => v,
            v => vec![v],
        };

        let args = values
            .into_iter()
            .map(|v| Value::Literal(Literal::from(v)))
            .collect();

        self.call_address_with_calltype_and_value::<C>(
            address,
            args,
            CommandFlags::CALL_WITH_VALUE,
            Some(value),
        )
    }

    fn call_sol_with_calltype<C>(
        &mut self,
        address: Address,
        call: C,
        calltype: CommandFlags,
    ) -> Result<ReturnValue, WeirollError>
    where
        C: SolCall,
    {
        let params_type: DynSolType = <C::Parameters<'_> as SolType>::SOL_NAME.parse()?;

        let mut encoded_args = Vec::new();
        <C as SolCall>::abi_encode_raw(&call, &mut encoded_args);

        let decoded = match params_type {
            DynSolType::Tuple(_) => params_type.abi_decode_sequence(&encoded_args)?,
            other => other.abi_decode(&encoded_args)?,
        };

        let values: Vec<DynSolValue> = match decoded {
            DynSolValue::Tuple(v) => v,
            v => vec![v],
        };

        let args = values
            .into_iter()
            .map(|v| Value::Literal(Literal::from(v)))
            .collect();

        self.call_address_with_calltype::<C>(address, args, calltype)
    }

    pub fn call_address<C>(
        &mut self,
        address: Address,
        args: Vec<Value<'a>>,
    ) -> Result<ReturnValue, WeirollError>
    where
        C: SolCall,
    {
        let return_type: DynSolType = <C::ReturnTuple<'_> as SolType>::SOL_NAME.parse()?;
        let return_type = match return_type {
            DynSolType::Tuple(mut elems) if elems.len() == 1 => elems.remove(0),
            other => other,
        };

        self.call_with_calltype::<C>(address, args, return_type, CommandFlags::CALL)
    }

    pub fn call_address_with_value<C>(
        &mut self,
        address: Address,
        value: U256,
        args: Vec<Value<'a>>,
    ) -> Result<ReturnValue, WeirollError>
    where
        C: SolCall,
    {
        let return_type: DynSolType = <C::ReturnTuple<'_> as SolType>::SOL_NAME.parse()?;
        let return_type = match return_type {
            DynSolType::Tuple(mut elems) if elems.len() == 1 => elems.remove(0),
            other => other,
        };

        self.call_with_calltype_and_value::<C>(
            address,
            args,
            return_type,
            CommandFlags::CALL_WITH_VALUE,
            Some(value),
        )
    }

    pub fn delegatecall_address<C>(
        &mut self,
        address: Address,
        args: Vec<Value<'a>>,
    ) -> Result<ReturnValue, WeirollError>
    where
        C: SolCall,
    {
        let return_type: DynSolType = <C::ReturnTuple<'_> as SolType>::SOL_NAME.parse()?;
        let return_type = match return_type {
            DynSolType::Tuple(mut elems) if elems.len() == 1 => elems.remove(0),
            other => other,
        };

        self.call_with_calltype::<C>(address, args, return_type, CommandFlags::DELEGATECALL)
    }

    pub fn staticcall_address<C>(
        &mut self,
        address: Address,
        args: Vec<Value<'a>>,
    ) -> Result<ReturnValue, WeirollError>
    where
        C: SolCall,
    {
        let return_type: DynSolType = <C::ReturnTuple<'_> as SolType>::SOL_NAME.parse()?;
        let return_type = match return_type {
            DynSolType::Tuple(mut elems) if elems.len() == 1 => elems.remove(0),
            other => other,
        };

        self.call_with_calltype::<C>(address, args, return_type, CommandFlags::STATICCALL)
    }

    fn call_address_with_calltype<C>(
        &mut self,
        address: Address,
        args: Vec<Value<'a>>,
        calltype: CommandFlags,
    ) -> Result<ReturnValue, WeirollError>
    where
        C: SolCall,
    {
        let return_type: DynSolType = <C::ReturnTuple<'_> as SolType>::SOL_NAME.parse()?;
        let return_type = match return_type {
            DynSolType::Tuple(mut elems) if elems.len() == 1 => elems.remove(0),
            other => other,
        };

        self.call_with_calltype::<C>(address, args, return_type, calltype)
    }

    fn call_address_with_calltype_and_value<C>(
        &mut self,
        address: Address,
        args: Vec<Value<'a>>,
        calltype: CommandFlags,
        value: Option<U256>,
    ) -> Result<ReturnValue, WeirollError>
    where
        C: SolCall,
    {
        let return_type: DynSolType = <C::ReturnTuple<'_> as SolType>::SOL_NAME.parse()?;
        let return_type = match return_type {
            DynSolType::Tuple(mut elems) if elems.len() == 1 => elems.remove(0),
            other => other,
        };

        self.call_with_calltype_and_value::<C>(address, args, return_type, calltype, value)
    }

    pub fn call<C: SolCall>(
        &mut self,
        address: Address,
        args: Vec<Value<'a>>,
        return_type: DynSolType,
    ) -> Result<ReturnValue, WeirollError> {
        self.call_with_calltype::<C>(address, args, return_type, CommandFlags::CALL)
    }

    fn call_with_calltype<C: SolCall>(
        &mut self,
        address: Address,
        args: Vec<Value<'a>>,
        return_type: DynSolType,
        calltype: CommandFlags,
    ) -> Result<ReturnValue, WeirollError> {
        self.call_with_calltype_and_value::<C>(address, args, return_type, calltype, None)
    }

    fn call_with_calltype_and_value<C: SolCall>(
        &mut self,
        address: Address,
        args: Vec<Value<'a>>,
        return_type: DynSolType,
        calltype: CommandFlags,
        value: Option<U256>,
    ) -> Result<ReturnValue, WeirollError> {
        debug_assert!(
            (calltype & CommandFlags::CALLTYPE_MASK) == calltype,
            "calltype must be one of CALL/DELEGATECALL/STATICCALL/CALL_WITH_VALUE"
        );
        debug_assert!(
            (calltype == CommandFlags::CALL_WITH_VALUE) == value.is_some(),
            "value must be set iff calltype is CALL_WITH_VALUE"
        );

        let dynamic = return_type.is_dynamic();
        let call = FunctionCall {
            address,
            flags: calltype,
            value,
            selector: C::SELECTOR,
            args,
            return_type,
        };

        let command = self.commands.insert(Command {
            call,
            kind: CommandType::Call,
        });

        Ok(ReturnValue { command, dynamic })
    }

    pub fn add_subplan<C: SolCall>(
        &mut self,
        address: Address,
        args: Vec<Value<'a>>,
        return_type: DynSolType,
    ) -> Result<ReturnValue, WeirollError> {
        let dynamic = return_type.is_dynamic();

        let mut has_subplan = false;
        let mut has_state = false;

        if args.len() != 2 {
            return Err(WeirollError::ArgumentCountMismatch);
        }

        for arg in args.iter() {
            match arg {
                Value::Subplan(_planner) => {
                    if has_subplan {
                        return Err(WeirollError::MultipleSubplans);
                    }
                    has_subplan = true;
                }
                Value::State(_state) => {
                    if has_state {
                        return Err(WeirollError::MultipleState);
                    }
                    has_state = true;
                }
                _ => {}
            }
        }

        if !has_subplan || !has_state {
            return Err(WeirollError::MissingStateOrSubplan);
        }

        let command = self.commands.insert(Command {
            call: FunctionCall {
                address,
                flags: CommandFlags::DELEGATECALL,
                value: None,
                selector: C::SELECTOR,
                args,
                return_type,
            },
            kind: CommandType::SubPlan,
        });

        Ok(ReturnValue { dynamic, command })
    }

    pub fn replace_state<C: SolCall>(&mut self, address: Address, args: Vec<Value<'a>>) {
        let call = FunctionCall {
            address,
            flags: CommandFlags::DELEGATECALL,
            value: None,
            selector: C::SELECTOR,
            args,
            return_type: DynSolType::Array(Box::new(DynSolType::Bytes)),
        };
        self.commands.insert(Command {
            call,
            kind: CommandType::RawCall,
        });
    }

    fn build_command_args(
        &self,
        command: &Command,
        return_slot_map: &BTreeMap<CommandKey, u8>,
        literal_slot_map: &BTreeMap<Literal, u8>,
        state: &Vec<Bytes>,
    ) -> Result<Vec<u8>, WeirollError> {
        let in_args = Vec::from_iter(command.call.args.iter());
        let mut extra_args: Vec<Value> = vec![];
        if command.call.flags & CommandFlags::CALLTYPE_MASK == CommandFlags::CALL_WITH_VALUE {
            if let Some(value) = command.call.value {
                extra_args.push(Value::Literal(value.into()));
            } else {
                return Err(WeirollError::MissingValue);
            }
        }

        let mut args = vec![];
        // NOTE: for CALL_WITH_VALUE, the value is treated as the first argument.
        for arg in extra_args.iter().chain(in_args.into_iter()) {
            let mut slot = match arg {
                Value::Return(val) => {
                    if let Some(slot) = return_slot_map.get(&val.command) {
                        *slot
                    } else {
                        return Err(WeirollError::MissingReturnSlot);
                    }
                }
                Value::Literal(val) => {
                    if let Some(slot) = literal_slot_map.get(val) {
                        *slot
                    } else {
                        return Err(WeirollError::MissingLiteralValue);
                    }
                }
                Value::State(_) => {
                    tracing::debug!("added state value, using 0xfe return slot");
                    0xFE
                }
                Value::Subplan(_) => {
                    tracing::debug!("added state value {state:?}");
                    // buildCommands has already built the subplan and put it in the last state slot
                    (state.len() - 1).try_into()?
                }
            };
            // todo- correct??
            if arg.is_dynamic_type() {
                slot |= 0x80;
            }

            args.push(slot);
        }

        Ok(args)
    }

    fn build_commands(&self, ps: &mut PlannerState) -> Result<Vec<FixedBytes<32>>, WeirollError> {
        let mut encoded_commands = vec![];

        // Build commands, and add state entries as needed
        for (cmd_key, command) in &self.commands {
            if command.kind == CommandType::SubPlan {
                // Find the subplan
                let subplanner = command
                    .call
                    .args
                    .iter()
                    .find_map(|arg| match arg {
                        Value::Subplan(planner) => Some(planner),
                        _ => None,
                    })
                    .ok_or(WeirollError::MissingSubplan)?;

                // Build a list of commands
                let subcommands = subplanner.build_commands(ps)?;

                // Push the commands onto the state
                ps.state.push(subcommands[0].clone().to_vec().into());

                // The slot is no longer needed after this command
                ps.free_slots.push((ps.state.len() - 1).try_into()?);
            }

            let mut flags = command.call.flags;

            let mut args = self.build_command_args(
                command,
                &ps.return_slot_map,
                &ps.literal_slot_map,
                &ps.state,
            )?;

            if args.len() > 6 {
                flags |= CommandFlags::EXTENDED_COMMAND;
            }

            // Add any expired state entries to free slots
            if let Some(expr) = ps.state_expirations.get(&cmd_key) {
                ps.free_slots.extend(expr.iter().copied())
            };

            // Figure out where to put the return value
            let mut ret = 0xff;
            if let Some(expiry) = ps.command_visibility.get(&cmd_key) {
                if let CommandType::RawCall | CommandType::SubPlan = command.kind {
                    return Err(WeirollError::InvalidReturnSlot);
                }

                ret = ps.state.len().try_into()?;
                if let Some(slot) = ps.free_slots.pop() {
                    ret = slot;
                }

                ps.return_slot_map.insert(cmd_key, ret);

                ps.state_expirations.entry(*expiry).or_default().push(ret);

                if ret == u8::try_from(ps.state.len())? {
                    ps.state.push(Bytes::default());
                }

                // todo: what's this?
                if command.call.return_type.is_dynamic() {
                    tracing::debug!("ret type is dynamic, set ret to 0x80");
                    ret |= 0x80;
                }
            } else if let CommandType::RawCall | CommandType::SubPlan = command.kind {
                // todo: what's this?
                // if command.call.fragment.outputs.len() == 1 {}
                tracing::debug!("call is raw or subplan, set ret to 0xfe");
                ret = 0xfe;
            }

            if (flags & CommandFlags::EXTENDED_COMMAND) == CommandFlags::EXTENDED_COMMAND {
                // Extended command
                let mut cmd = BytesMut::with_capacity(32);

                cmd.put(&command.call.selector[..]);
                cmd.put(&flags.bits().to_le_bytes()[..]);
                cmd.put(&[0u8; 6][..]);
                cmd.put_u8(ret);
                cmd.put(&command.call.address.0.0[..]);

                // push first command, indicating extended cmd
                let word: [u8; 32] = cmd.to_vec().try_into().unwrap();
                encoded_commands.push(word.into());

                // use the next command for the actual args
                args.resize(32, 0xff);
                let word: [u8; 32] = args.try_into().unwrap();
                encoded_commands.push(word.into());
            } else {
                // Standard command
                let mut cmd = BytesMut::with_capacity(32);

                cmd.put(&command.call.selector[..]);
                cmd.put(&flags.bits().to_le_bytes()[..]);

                // pad args to 6
                args.resize(6, 0xff);

                cmd.put(&args[..]);
                cmd.put_u8(ret.to_le());
                cmd.put(&command.call.address.0.0[..]);

                let word: [u8; 32] = cmd.to_vec().try_into().unwrap();
                encoded_commands.push(word.into());
            }
        }

        Ok(encoded_commands)
    }

    fn preplan(
        &self,
        literal_visibility: &mut Vec<(Literal, CommandKey)>,
        command_visibility: &mut BTreeMap<CommandKey, CommandKey>,
        seen: &mut BTreeSet<CommandKey>,
    ) -> Result<(), WeirollError> {
        for (cmd_key, command) in &self.commands {
            let in_args = &command.call.args;
            let mut extra_args = vec![];

            if command.call.flags & CommandFlags::CALLTYPE_MASK == CommandFlags::CALL_WITH_VALUE {
                if let Some(value) = command.call.value {
                    extra_args.push(value.into());
                } else {
                    return Err(WeirollError::MissingValue);
                }
            }

            // NOTE: for CALL_WITH_VALUE, the value is treated as the first argument.
            for arg in extra_args.iter().chain(in_args.iter()) {
                match arg {
                    Value::Return(val) => {
                        if !seen.contains(&val.command) {
                            return Err(WeirollError::CommandNotVisible);
                        }
                        command_visibility.insert(val.command, cmd_key);
                    }
                    Value::Literal(val) => {
                        // Remove old visibility (if exists)
                        literal_visibility.retain(|(l, _)| *l != *val);
                        literal_visibility.push((val.clone(), cmd_key));
                    }
                    Value::State(_) => {}
                    Value::Subplan(subplan) => {
                        // let mut subplan_seen = Default::default();
                        if command.call.return_type.is_dynamic() {
                            subplan.preplan(literal_visibility, command_visibility, seen)?;
                        }
                    }
                }
            }

            seen.insert(cmd_key);
        }

        dbg!(&command_visibility, &literal_visibility);

        Ok(())
    }

    pub fn plan(&self) -> Result<(Vec<FixedBytes<32>>, Vec<Bytes>), WeirollError> {
        // Tracks the last time a literal is used in the program
        let mut literal_visibility = Default::default();

        // Tracks the last time a command's output is used in the program
        let mut command_visibility = Default::default();

        // Populate visibility maps
        self.preplan(
            &mut literal_visibility,
            &mut command_visibility,
            &mut BTreeSet::new(),
        )?;

        // Maps from commands to the slots that expire on execution (if any)
        let mut state_expirations: BTreeMap<CommandKey, Vec<u8>> = Default::default();

        // Tracks the state slot each literal is stored in
        let mut literal_slot_map: BTreeMap<Literal, u8> = Default::default();

        // empty initial state
        let mut state: Vec<Bytes> = Default::default();

        // Prepopulate the state and state expirations with literals
        for (literal, last_command) in literal_visibility {
            let slot = state.len() as u8;
            state.push(literal.bytes());
            state_expirations
                .entry(last_command)
                .or_default()
                .push(slot);
            literal_slot_map.insert(literal, slot);
        }

        let mut ps = PlannerState {
            return_slot_map: Default::default(),
            literal_slot_map,
            free_slots: Default::default(),
            state_expirations,
            command_visibility,
            state,
        };

        let encoded_commands = self.build_commands(&mut ps)?;

        Ok((encoded_commands, ps.state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bindings::{math::Math, strings::Strings};
    use alloy::dyn_abi::DynSolValue;
    use alloy::{
        dyn_abi::DynSolType,
        primitives::{U256, address},
        sol,
    };

    sol! {
        interface SampleContract {
            function useState(bytes[] state) external returns (bytes[]);
        }
    }

    sol! {
        interface SubplanContract {
            function execute(bytes32[] commands, bytes[] state) external returns (bytes[]);
        }
    }

    sol! {
        interface ReadOnlySubplanContract {
            function execute(bytes32[] commands, bytes[] state) external;
        }
    }

    sol! {
        interface ExtendedCommandContract {
            function test(
                uint256 a,
                uint256 b,
                uint256 c,
                uint256 d,
                uint256 e,
                uint256 f,
                uint256 g
            ) external returns (uint256);
        }
    }

    fn addr() -> Address {
        address!("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee")
    }

    #[test]
    fn test_planner_calltype_flags() {
        let mut planner = Planner::default();

        let _ = planner
            .call_address::<Math::addCall>(addr(), vec![U256::from(1).into(), U256::from(2).into()])
            .unwrap();
        let _ = planner
            .delegatecall_address::<Math::addCall>(
                addr(),
                vec![U256::from(1).into(), U256::from(2).into()],
            )
            .unwrap();
        let _ = planner
            .staticcall_address::<Math::addCall>(
                addr(),
                vec![U256::from(1).into(), U256::from(2).into()],
            )
            .unwrap();

        let (commands, _state) = planner.plan().unwrap();
        assert_eq!(commands.len(), 3);

        // Flag byte is immediately after the 4-byte selector.
        assert_eq!(commands[0].as_slice()[4], CommandFlags::CALL.bits());
        assert_eq!(commands[1].as_slice()[4], CommandFlags::DELEGATECALL.bits());
        assert_eq!(commands[2].as_slice()[4], CommandFlags::STATICCALL.bits());
    }

    #[test]
    fn test_planner_call_with_value() {
        let mut planner = Planner::default();

        let value = U256::from(5);
        let _ = planner
            .call_address_with_value::<Math::addCall>(
                addr(),
                value,
                vec![U256::from(1).into(), U256::from(2).into()],
            )
            .unwrap();

        let (commands, state) = planner.plan().unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(
            commands[0].as_slice()[4],
            CommandFlags::CALL_WITH_VALUE.bits()
        );

        // Value is injected as the first argument.
        assert_eq!(state.len(), 3);

        let value_bytes = DynSolValue::from(value).abi_encode();
        let value_slot = state
            .iter()
            .position(|b| b.as_ref() == value_bytes)
            .expect("value is present in state") as u8;

        // Arg0 byte is immediately after selector(4) + flags(1).
        let arg0 = commands[0].as_slice()[5];
        assert_eq!(arg0, value_slot);
    }

    #[test]
    fn test_planner_add() {
        let mut planner = Planner::default();
        planner
            .call::<Math::addCall>(
                addr(),
                vec![U256::from(1).into(), U256::from(2).into()],
                DynSolType::Uint(256),
            )
            .expect("can add call");
        let (commands, state) = planner.plan().expect("plan");

        assert_eq!(commands.len(), 1);
        assert_eq!(
            commands[0],
            "0x771602f7010001ffffffffffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                .parse::<Bytes>()
                .unwrap()[..]
        );

        assert_eq!(state.len(), 2);
        assert_eq!(state[0], DynSolValue::from(U256::from(1)).abi_encode());
        assert_eq!(state[1], DynSolValue::from(U256::from(2)).abi_encode());
    }

    #[test]
    fn test_planner_deduplicates_literals() {
        let mut planner = Planner::default();
        planner
            .call::<Math::addCall>(
                addr(),
                vec![U256::from(1).into(), U256::from(1).into()],
                DynSolType::Uint(256),
            )
            .expect("can add call");
        let (_, state) = planner.plan().expect("plan");
        assert_eq!(state.len(), 1);
    }

    #[test]
    fn test_planner_return_values() {
        let mut planner = Planner::default();
        let ret = planner
            .call::<Math::addCall>(
                addr(),
                vec![U256::from(1).into(), U256::from(2).into()],
                DynSolType::Uint(256),
            )
            .expect("can add call");
        planner
            .call::<Math::addCall>(
                addr(),
                vec![ret.into(), U256::from(3).into()],
                DynSolType::Uint(256),
            )
            .expect("can add call with return val");
        let (commands, state) = planner.plan().expect("plan");
        assert_eq!(commands.len(), 2);
        assert_eq!(
            commands[0],
            "0x771602f7010001ffffffff01eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                .parse::<Bytes>()
                .unwrap()[..]
        );
        assert_eq!(
            commands[1],
            "0x771602f7010102ffffffffffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                .parse::<Bytes>()
                .unwrap()[..]
        );
        assert_eq!(state.len(), 3);
        assert_eq!(state[0], DynSolValue::from(U256::from(1)).abi_encode());
        assert_eq!(state[1], DynSolValue::from(U256::from(2)).abi_encode());
        assert_eq!(state[2], DynSolValue::from(U256::from(3)).abi_encode());
    }

    #[test]
    fn test_planner_intermediate_state_slots() {
        // todo: how is this different from test_planner_return_values?
        let mut planner = Planner::default();
        let ret = planner
            .call::<Math::addCall>(
                addr(),
                vec![U256::from(1).into(), U256::from(1).into()],
                DynSolType::Uint(256),
            )
            .expect("can add call");
        planner
            .call::<Math::addCall>(
                addr(),
                vec![U256::from(1).into(), ret.into()],
                DynSolType::Uint(256),
            )
            .expect("can add call with return val");
        let (commands, state) = planner.plan().expect("plan");
        assert_eq!(commands.len(), 2);
        assert_eq!(
            commands[0],
            "0x771602f7010000ffffffff01eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                .parse::<Bytes>()
                .unwrap()[..]
        );
        assert_eq!(
            commands[1],
            "0x771602f7010001ffffffffffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                .parse::<Bytes>()
                .unwrap()[..]
        );
        assert_eq!(state.len(), 2);
        assert_eq!(state[0], DynSolValue::from(U256::from(1)).abi_encode());
        assert_eq!(state[1], Bytes::default());
    }

    #[test]
    fn test_planner_dynamic_arguments() {
        let mut planner = Planner::default();
        planner
            .call::<Strings::strlenCall>(
                addr(),
                vec![String::from("Hello, world!").into()],
                DynSolType::Uint(256),
            )
            .expect("can add call");
        let (commands, state) = planner.plan().expect("plan");
        assert_eq!(commands.len(), 1);
        assert_eq!(
            commands[0],
            "0x367bbd780180ffffffffffffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                .parse::<Bytes>()
                .unwrap()[..]
        );
        assert_eq!(state.len(), 1);
        assert_eq!(
            state[0],
            DynSolValue::from("Hello, world!".to_string()).abi_encode()[32..]
        );
    }

    #[test]
    fn test_planner_dynamic_return_values() {
        let mut planner = Planner::default();
        planner
            .call::<Strings::strcatCall>(
                addr(),
                vec![
                    String::from("Hello, ").into(),
                    String::from("world!").into(),
                ],
                DynSolType::String,
            )
            .expect("can add call");
        let (commands, state) = planner.plan().expect("plan");
        assert_eq!(commands.len(), 1);
        assert_eq!(
            commands[0],
            "0xd824ccf3018081ffffffffffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                .parse::<Bytes>()
                .unwrap()[..]
        );
        assert_eq!(state.len(), 2);
        assert_eq!(
            state[0],
            DynSolValue::from("Hello, ".to_string()).abi_encode()[32..]
        );
        assert_eq!(
            state[1],
            DynSolValue::from("world!".to_string()).abi_encode()[32..]
        );
    }

    #[test]
    fn test_planner_dynamic_return_values_with_dynamic_arguments() {
        let mut planner = Planner::default();
        let ret = planner
            .call::<Strings::strcatCall>(
                addr(),
                vec![
                    String::from("Hello, ").into(),
                    String::from("world!").into(),
                ],
                DynSolType::String,
            )
            .expect("can add call");
        planner
            .call::<Strings::strlenCall>(addr(), vec![ret.into()], DynSolType::Uint(256))
            .expect("can add call with return val");
        let (commands, state) = planner.plan().expect("plan");
        assert_eq!(commands.len(), 2);
        assert_eq!(
            commands[0],
            "0xd824ccf3018081ffffffff81eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                .parse::<Bytes>()
                .unwrap()[..]
        );
        assert_eq!(
            commands[1],
            "0x367bbd780181ffffffffffffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                .parse::<Bytes>()
                .unwrap()[..]
        );
        assert_eq!(state.len(), 2);
        assert_eq!(
            state[0],
            DynSolValue::from("Hello, ".to_string()).abi_encode()[32..]
        );
        assert_eq!(
            state[1],
            DynSolValue::from("world!".to_string()).abi_encode()[32..]
        );
    }

    #[test]
    fn test_planner_argument_count_mismatch() {
        let mut planner = Planner::default();
        let ret = planner.add_subplan::<Math::addCall>(
            addr(),
            vec![U256::from(1).into()],
            DynSolType::Uint(256),
        );
        assert_eq!(ret.err(), Some(WeirollError::ArgumentCountMismatch));
    }

    #[test]
    fn test_planner_replace_state() {
        let mut planner = Planner::default();
        planner.replace_state::<SampleContract::useStateCall>(
            addr(),
            vec![Value::State(Default::default())],
        );
        let (commands, state) = planner.plan().expect("plan");
        assert_eq!(commands.len(), 1);
        assert_eq!(
            commands[0],
            "0x08f389c800fefffffffffffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                .parse::<Bytes>()
                .unwrap()[..]
        );
        assert_eq!(state.len(), 0);
    }

    #[test]
    fn test_planner_supports_subplans() {
        let mut subplanner = Planner::default();
        subplanner
            .call::<Math::addCall>(
                addr(),
                vec![U256::from(1).into(), U256::from(2).into()],
                DynSolType::Uint(256),
            )
            .expect("can add call");
        let mut planner = Planner::default();
        planner
            .add_subplan::<SubplanContract::executeCall>(
                addr(),
                vec![
                    Value::Subplan(&subplanner),
                    Value::State(Default::default()),
                ],
                DynSolType::Array(Box::new(DynSolType::Bytes)),
            )
            .expect("can add subplan");
        let (commands, state) = planner.plan().expect("plan");
        assert_eq!(commands.len(), 1);
        assert_eq!(
            commands[0],
            "0xde792d5f0082fefffffffffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                .parse::<Bytes>()
                .unwrap()[..]
        );
        assert_eq!(state.len(), 3);
        assert_eq!(state[0], DynSolValue::from(U256::from(1)).abi_encode());
        assert_eq!(state[1], DynSolValue::from(U256::from(2)).abi_encode());
        assert_eq!(
            state[2],
            "0x771602f7010001ffffffffffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                .parse::<Bytes>()
                .unwrap()
        );
    }

    #[test]
    #[ignore]
    fn test_planner_allows_return_value_access_in_parent_scope() {
        let mut subplanner = Planner::default();
        let sum = subplanner
            .call::<Math::addCall>(
                addr(),
                vec![U256::from(1).into(), U256::from(2).into()],
                DynSolType::Uint(256),
            )
            .expect("can add call");
        let mut planner = Planner::default();
        planner
            .add_subplan::<SubplanContract::executeCall>(
                addr(),
                vec![
                    Value::Subplan(&subplanner),
                    Value::State(Default::default()),
                ],
                DynSolType::Array(Box::new(DynSolType::Bytes)),
            )
            .expect("can add subplan");
        planner
            .call::<Math::addCall>(
                addr(),
                vec![sum.into(), U256::from(3).into()],
                DynSolType::Uint(256),
            )
            .expect("can add call");
        let (commands, _) = planner.plan().expect("plan");
        assert_eq!(commands.len(), 2);
        assert_eq!(
            commands[0],
            // Invoke subplanner
            "0xde792d5f0083fefffffffffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                .parse::<Bytes>()
                .unwrap()[..]
        );
        assert_eq!(
            commands[1],
            // sum + 3
            "0x771602f7010102ffffffffffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                .parse::<Bytes>()
                .unwrap()[..]
        );
    }

    #[test]
    #[ignore]
    fn test_planner_allows_return_value_access_across_scopes() {
        let mut subplanner1 = Planner::default();
        let sum = subplanner1
            .call::<Math::addCall>(
                addr(),
                vec![U256::from(1).into(), U256::from(2).into()],
                DynSolType::Uint(256),
            )
            .expect("can add call");

        let mut subplanner2 = Planner::default();
        subplanner2
            .call::<Math::addCall>(
                addr(),
                vec![sum.into(), U256::from(3).into()],
                DynSolType::Uint(256),
            )
            .expect("can add call");

        let mut planner = Planner::default();
        planner
            .add_subplan::<SubplanContract::executeCall>(
                addr(),
                vec![
                    Value::Subplan(&subplanner1),
                    Value::State(Default::default()),
                ],
                DynSolType::Array(Box::new(DynSolType::Bytes)),
            )
            .expect("can add subplan");
        planner
            .add_subplan::<SubplanContract::executeCall>(
                addr(),
                vec![
                    Value::Subplan(&subplanner2),
                    Value::State(Default::default()),
                ],
                DynSolType::Array(Box::new(DynSolType::Bytes)),
            )
            .expect("can add subplan");

        let (_commands, _state) = planner.plan().expect("plan");
    }

    #[test]
    fn test_uses_extended_commands_where_necessary() {
        let mut planner = Planner::default();
        planner
            .call::<ExtendedCommandContract::testCall>(
                addr(),
                vec![
                    U256::from(1).into(),
                    U256::from(2).into(),
                    U256::from(3).into(),
                    U256::from(4).into(),
                    U256::from(5).into(),
                    U256::from(6).into(),
                    U256::from(7).into(),
                ],
                DynSolType::Uint(256),
            )
            .unwrap();
        let (commands, _state) = planner.plan().expect("plan");
        assert_eq!(commands.len(), 2);
        assert_eq!(
            commands[0],
            "0xe473580d41000000000000ffeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                .parse::<Bytes>()
                .unwrap()[..]
        );
        assert_eq!(
            commands[1],
            "0x00010203040506ffffffffffffffffffffffffffffffffffffffffffffffffff"
                .parse::<Bytes>()
                .unwrap()[..]
        );
    }
}
