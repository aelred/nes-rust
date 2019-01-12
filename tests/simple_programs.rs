use nes_rust::mem;
use nes_rust::Address;
use nes_rust::Memory;
use nes_rust::OpCode::*;
use nes_rust::CPU;

const PARAM_ADDRESS: Address = Address::new(0xFEED);
const RETURN_ADDRESS: Address = Address::new(0xBEEF);
const HALT_ADDRESS: Address = Address::new(0xDEAD);

macro_rules! run {
    ([$($params: expr),*] -> [$($expected: expr),*]; $( $expr: expr ),*) => {
        let mut cpu = CPU::with_memory(mem!($($expr),*));
        run(&mut cpu, &[$($params),*], &[$($expected),*]);
    };
}

fn run<M: Memory>(cpu: &mut CPU<M>, params: &[u8], expected: &[u8]) {
    for (offset, param) in params.iter().enumerate() {
        cpu.write(PARAM_ADDRESS + offset as u16, *param);
    }

    const MAX_INSTRUCTIONS: u32 = 1_000;

    let mut instructions = 0;

    while cpu.read(HALT_ADDRESS) == 0 {
        cpu.run_instruction();

        instructions += 1;

        if instructions > MAX_INSTRUCTIONS {
            panic!("Exceeded maximum number of instructions");
        }
    }

    let mut result = vec![];
    for offset in 0..expected.len() {
        result.push(cpu.read(RETURN_ADDRESS + offset as u16));
    }

    assert_eq!(result, expected);
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
