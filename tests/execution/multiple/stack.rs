use crate::{assert_memory, assert_no_output, assert_step_device, setup};
use tape_device::constants::code::{CALL_ADDR, CALL_AREG, HALT, POP_REG, PUSH_REG, PUSH_VAL, RET};
use tape_device::constants::hardware::{REG_A0, REG_A1, REG_ACC, REG_D0, REG_D1};
use tape_device::device::internals::RunResult;
use tape_device::device::Dump;

#[test]
#[rustfmt::skip]
fn test_multiple_stack_ops() {
    let ops = vec![
        PUSH_VAL, 73,
        PUSH_REG, REG_D1,
        POP_REG, REG_ACC,
        CALL_ADDR, 0, 10,
        HALT,
        CALL_AREG, REG_A1,
        HALT,
        RET
    ];
    let mut device = setup(ops);
    device.data_reg = [0, 32, 0, 0];
    device.addr_reg = [0, 13];

    assert_step_device("PUSH 73", &mut device, Dump { pc: 2, data_reg: [0, 32, 0, 0], addr_reg: [0, 13], sp: 65534, ..Default::default() });
    assert_step_device("PUSH D1", &mut device, Dump { pc: 4, data_reg: [0, 32, 0, 0], addr_reg: [0, 13], sp: 65533, ..Default::default() });
    assert_step_device("POP ACC", &mut device, Dump { pc: 6, acc: 32, data_reg: [0, 32, 0, 0], addr_reg: [0, 13], sp: 65534, ..Default::default() });
    assert_step_device("CALL lbl", &mut device, Dump { pc: 10, acc: 32, data_reg: [0, 32, 0, 0], addr_reg: [0, 13], sp: 65530, fp: 65530, ..Default::default() });
    assert_step_device("CALL A1", &mut device, Dump { pc: 13, acc: 32, data_reg: [0, 32, 0, 0], addr_reg: [0, 13], sp: 65526, fp: 65526, ..Default::default() });
    assert_step_device("RET", &mut device, Dump { pc: 12, acc: 32, data_reg: [0, 32, 0, 0], addr_reg: [0, 13], sp: 65530, fp: 65530, ..Default::default() });
    assert_eq!(device.step(true), RunResult::Halt);

    assert_no_output(device);
}

#[test]
#[rustfmt::skip]
fn test_multiple_addr_stack_ops() {
    let ops = vec![
        PUSH_VAL, 73,
        PUSH_REG, REG_D0,
        PUSH_REG, REG_A1,
        POP_REG, REG_A0,
        POP_REG, REG_ACC,
        POP_REG, REG_ACC,
    ];
    let mut device = setup(ops);
    device.addr_reg = [66, 259];

    assert_step_device("PUSH 73", &mut device, Dump { pc: 2, addr_reg: [66, 259], sp: 65534, ..Default::default() });
    assert_step_device("PUSH D0", &mut device, Dump { pc: 4, addr_reg: [66, 259], sp: 65533, ..Default::default() });
    assert_step_device("PUSH A1", &mut device, Dump { pc: 6, addr_reg: [66, 259], sp: 65531, ..Default::default() });
    assert_memory(&device, 65531, &[1,3,0,73]);
    assert_step_device("POP A0", &mut device, Dump { pc: 8, addr_reg: [259, 259], sp: 65533, ..Default::default() });
    assert_memory(&device, 65531, &[1,3,0,73]);
    assert_step_device("POP ACC", &mut device, Dump { pc: 10, acc: 0, addr_reg: [259, 259], sp: 65534, ..Default::default() });
    assert_memory(&device, 65531, &[1,3,0,73]);
    assert_step_device("POP ACC", &mut device, Dump { pc: 12, acc: 73, addr_reg: [259, 259], sp: 65535, ..Default::default() });
    assert_memory(&device, 65531, &[1,3,0,73]);


    assert_no_output(device);
}
