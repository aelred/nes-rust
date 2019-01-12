use nes_rust::mem;
use nes_rust::Address;
use nes_rust::Memory;
use nes_rust::OpCode::*;
use nes_rust::CPU;
use nes_rust::OpCode;
use nes_rust::SerializeBytes;
use std::fmt::Debug;

const PARAM_ADDRESS: Address = Address::new(0xFEED);
const RETURN_ADDRESS: Address = Address::new(0xBEEF);
const HALT_ADDRESS: Address = Address::new(0xDEAD);

macro_rules! run {
    ($params:tt -> $expected:expr; $( $expr: tt )*) => {
        let mut cpu = CPU::with_memory(mem!($($expr)*));
        let params: Vec<u8> = $params.into_iter().cloned().collect();
        let expected: Vec<u8> = $expected.into_iter().cloned().collect();
        run(&mut cpu, &params, &expected);
    };
}

fn run<M: Memory>(cpu: &mut CPU<M>, params: &[u8], expected: &[u8]) {
    for (offset, param) in params.iter().enumerate() {
        cpu.write(PARAM_ADDRESS + offset as u16, *param);
    }

    const MAX_INSTRUCTIONS: u32 = 1_000;

    let mut instructions = 0;

    while cpu.read::<u8>(HALT_ADDRESS) == 0 {
        cpu.run_instruction();

        instructions += 1;

        if instructions > MAX_INSTRUCTIONS {
            panic!("Exceeded maximum number of instructions");
        }
    }

    let mut result: Vec<u8> = vec![];
    for offset in 0..expected.len() {
        result.push(cpu.read(RETURN_ADDRESS + offset as u16));
    }

    assert_eq!(result, expected);
}

#[test]
fn hello_world() {
    run!({"Felix\0".as_bytes()} -> "hello world from Felix!\0".as_bytes();
        0 => {
            LDXImmediate, 255u8,
            LDYImmediate, 16u8,
            INX,
            INY,
            LDAAbsoluteX, PARAM_ADDRESS,
            STAAbsoluteY, RETURN_ADDRESS,
            BNE, -10i8,
            LDAImmediate, 33u8,
            STAAbsoluteY, RETURN_ADDRESS,
            INY,
            LDAImmediate, 0u8,
            STAAbsoluteY, RETURN_ADDRESS,
            LDAImmediate, 1u8,
            STAAbsolute, HALT_ADDRESS
        }
        RETURN_ADDRESS => {
            104u8, 101u8, 108u8, 108u8, 111u8, 32u8,
            119u8, 111u8, 114u8, 108u8, 100u8, 32u8,
            102u8, 114u8, 111u8, 109u8, 32u8
        }
    );
}

#[test]
fn one_plus_two() {
    run!([1, 2] -> [3];
        LDAAbsolute, PARAM_ADDRESS,
        INX,
        ADCAbsoluteX, PARAM_ADDRESS,
        STAAbsolute, RETURN_ADDRESS,
        LDAImmediate, 1u8,
        STAAbsolute, HALT_ADDRESS
    );
}

#[test]
fn seven_times_six() {
    run!([7, 6] -> [42];
        LDAImmediate, 0u8,
        LDYAbsolute, PARAM_ADDRESS,
        INX,
        ADCAbsoluteX, PARAM_ADDRESS,
        DEY,
        BNE, -6i8,
        STAAbsolute, RETURN_ADDRESS,
        LDAImmediate, 1u8,
        STAAbsolute, HALT_ADDRESS
    );
}

#[test]
fn triangle_number() {
    run!([20] -> [210];
        LDAImmediate, 0u8,
        LDXAbsolute, PARAM_ADDRESS,
        ADCAbsolute, PARAM_ADDRESS,
        DECAbsolute, PARAM_ADDRESS,
        BNE, -8i8,
        STAAbsolute, RETURN_ADDRESS,
        LDAImmediate, 1u8,
        STAAbsolute, HALT_ADDRESS
    );
}

#[test]
fn fibonacci() {
    run!([11] -> [89];
        LDXImmediate, 0x01u8,
        STXZeroPage, 0xffu8,
        SEC,
        LDYAbsolute, PARAM_ADDRESS,
        TYA,
        SBCImmediate, 0x03u8,
        TAY,
        CLC,
        LDAImmediate, 0x02u8,
        STAZeroPage, 0xfeu8,
        LDXZeroPage, 0xfeu8,
        ADCZeroPage, 0xffu8,
        STAZeroPage, 0xfeu8,
        STXZeroPage, 0xffu8,
        DEY,
        BNE, -11i8,
        STAAbsolute, RETURN_ADDRESS,
        LDAImmediate, 1u8,
        STAAbsolute, HALT_ADDRESS
    );
}
