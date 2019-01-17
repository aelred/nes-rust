use nes_rust::Address;
use nes_rust::CPU;
use nes_rust::instructions::*;
use nes_rust::mem;
use nes_rust::Memory;

const PARAM_ADDRESS: u8 = 0x80;
const RETURN_ADDRESS: u8 = 0xB0;
const HALT_ADDRESS: u8 = 0xFF;

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
        cpu.write(
            Address::from_bytes(0, PARAM_ADDRESS) + offset as u16,
            *param,
        );
    }

    const MAX_INSTRUCTIONS: u32 = 1_000;

    let mut instructions = 0;

    while cpu.read(Address::from_bytes(0, HALT_ADDRESS)) == 0 {
        cpu.run_instruction();

        instructions += 1;

        if instructions > MAX_INSTRUCTIONS {
            panic!("Exceeded maximum number of instructions");
        }
    }

    let mut result: Vec<u8> = vec![];
    for offset in 0..expected.len() {
        result.push(cpu.read(Address::from_bytes(0, RETURN_ADDRESS) + offset as u16));
    }

    assert_eq!(result, expected);
}

#[test]
fn hello_world() {
    run!(b"Felix\0" -> b"hello world from Felix!\0";
        0 => {
            LDX_IMMEDIATE, 255u8,
            LDY_IMMEDIATE, 16u8,
            INX,
            INY,
            LDA_ABSOLUTE_X, PARAM_ADDRESS, 0,
            STA_ABSOLUTE_Y, RETURN_ADDRESS, 0,
            BNE, -10i8 as u8,
            LDA_IMMEDIATE, 33u8,
            STA_ABSOLUTE_Y, RETURN_ADDRESS, 0,
            INY,
            LDA_IMMEDIATE, 0u8,
            STA_ABSOLUTE_Y, RETURN_ADDRESS, 0,
            LDA_IMMEDIATE, 1u8,
            STA_ABSOLUTE, HALT_ADDRESS, 0
        }
        u16::from(RETURN_ADDRESS) => {
            104u8, 101u8, 108u8, 108u8, 111u8, 32u8,
            119u8, 111u8, 114u8, 108u8, 100u8, 32u8,
            102u8, 114u8, 111u8, 109u8, 32u8
        }
    );
}

#[test]
fn one_plus_two() {
    run!([1, 2] -> [3];
        LDA_ABSOLUTE, PARAM_ADDRESS, 0,
        INX,
        ADC_ABSOLUTE_X, PARAM_ADDRESS, 0,
        STA_ABSOLUTE, RETURN_ADDRESS, 0,
        LDA_IMMEDIATE, 1u8,
        STA_ABSOLUTE, HALT_ADDRESS, 0
    );
}

#[test]
fn seven_times_six() {
    run!([7, 6] -> [42];
        LDA_IMMEDIATE, 0u8,
        LDY_ABSOLUTE, PARAM_ADDRESS, 0,
        INX,
        ADC_ABSOLUTE_X, PARAM_ADDRESS, 0,
        DEY,
        BNE, -6i8 as u8,
        STA_ABSOLUTE, RETURN_ADDRESS, 0,
        LDA_IMMEDIATE, 1u8,
        STA_ABSOLUTE, HALT_ADDRESS, 0
    );
}

#[test]
fn triangle_number() {
    run!([20] -> [210];
        LDA_IMMEDIATE, 0u8,
        LDX_ABSOLUTE, PARAM_ADDRESS, 0,
        ADC_ABSOLUTE, PARAM_ADDRESS, 0,
        DEC_ABSOLUTE, PARAM_ADDRESS, 0,
        BNE, -8i8 as u8,
        STA_ABSOLUTE, RETURN_ADDRESS, 0,
        LDA_IMMEDIATE, 1u8,
        STA_ABSOLUTE, HALT_ADDRESS, 0
    );
}

#[test]
fn triangle_number_subroutine() {
    run!([20] -> [210];
        0 => {
            JSR, 0x34, 0x12,
            LDA_IMMEDIATE, 1u8,
            STA_ABSOLUTE, HALT_ADDRESS, 0
        }
        0x1234 => {
            LDA_IMMEDIATE, 0u8,
            LDX_ABSOLUTE, PARAM_ADDRESS, 0,
            ADC_ABSOLUTE, PARAM_ADDRESS, 0,
            DEC_ABSOLUTE, PARAM_ADDRESS, 0,
            BNE, -8i8 as u8,
            STA_ABSOLUTE, RETURN_ADDRESS, 0,
            RTS
        }
    );
}

#[test]
fn fibonacci() {
    run!([11] -> [89];
        LDX_IMMEDIATE, 0x01u8,
        STX_ZERO_PAGE, 0xefu8,
        SEC,
        LDY_ABSOLUTE, PARAM_ADDRESS, 0,
        TYA,
        SBC_IMMEDIATE, 0x03u8,
        TAY,
        CLC,
        LDA_IMMEDIATE, 0x02u8,
        STA_ZERO_PAGE, 0xeeu8,
        LDX_ZERO_PAGE, 0xeeu8,
        ADC_ZERO_PAGE, 0xefu8,
        STA_ZERO_PAGE, 0xeeu8,
        STX_ZERO_PAGE, 0xefu8,
        DEY,
        BNE, -11i8 as u8,
        STA_ABSOLUTE, RETURN_ADDRESS, 0,
        LDA_IMMEDIATE, 1u8,
        STA_ABSOLUTE, HALT_ADDRESS, 0
    );
}
