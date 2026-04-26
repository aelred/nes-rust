use nes_rust::instructions::*;
use nes_rust::Address;
use nes_rust::Bus;
use nes_rust::{mem, CPUState, Tickable, CPU};

const PARAM_ADDRESS: u8 = 0x80;
const RETURN_ADDRESS: u8 = 0xB0;
const HALT_ADDRESS: u8 = 0xFF;

macro_rules! run {
    ($params:tt -> $expected:expr; $( $expr: tt )*) => {
        let mut memory = mem!($($expr)*);
        let cpu = CPUState::from_bus(&mut memory);
        let params: Vec<u8> = $params.iter().cloned().collect();
        let expected: Vec<u8> = $expected.iter().cloned().collect();
        run(memory, cpu, &params, &expected);
    };
}

fn run<B: Bus + Tickable>(mut bus: B, mut cpu: CPUState, params: &[u8], expected: &[u8]) {
    for (offset, param) in params.iter().enumerate() {
        bus.write(
            Address::from_bytes(0, PARAM_ADDRESS) + offset as u16,
            *param,
        );
    }

    const MAX_INSTRUCTIONS: u32 = 1_000;

    let mut instructions = 0;

    while bus.read(Address::from_bytes(0, HALT_ADDRESS)) == 0 {
        CPU::new(&mut bus, &mut cpu).run_instruction();

        instructions += 1;

        if instructions > MAX_INSTRUCTIONS {
            panic!("Exceeded maximum number of instructions");
        }
    }

    let mut result: Vec<u8> = vec![];
    for offset in 0..expected.len() {
        let address = Address::from_bytes(0, RETURN_ADDRESS) + offset as u16;
        result.push(bus.read(address));
    }

    assert_eq!(result, expected);
}

#[test]
fn hello_world() {
    run!(b"Felix\0" -> b"hello world from Felix!\0";
        0 => {
            LDX_IMM, 255u8,
            LDY_IMM, 16u8,
            INX,
            INY,
            LDA_ABX, PARAM_ADDRESS, 0,
            STA_ABY, RETURN_ADDRESS, 0,
            BNE, -10i8 as u8,
            LDA_IMM, 33u8,
            STA_ABY, RETURN_ADDRESS, 0,
            INY,
            LDA_IMM, 0u8,
            STA_ABY, RETURN_ADDRESS, 0,
            LDA_IMM, 1u8,
            STA_ABS, HALT_ADDRESS, 0
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
        LDA_ABS, PARAM_ADDRESS, 0,
        INX,
        ADC_ABX, PARAM_ADDRESS, 0,
        STA_ABS, RETURN_ADDRESS, 0,
        LDA_IMM, 1u8,
        STA_ABS, HALT_ADDRESS, 0
    );
}

#[test]
fn seven_times_six() {
    run!([7, 6] -> [42];
        LDA_IMM, 0u8,
        LDY_ABS, PARAM_ADDRESS, 0,
        INX,
        ADC_ABX, PARAM_ADDRESS, 0,
        DEY,
        BNE, -6i8 as u8,
        STA_ABS, RETURN_ADDRESS, 0,
        LDA_IMM, 1u8,
        STA_ABS, HALT_ADDRESS, 0
    );
}

#[test]
fn triangle_number() {
    run!([20] -> [210];
        LDA_IMM, 0u8,
        LDX_ABS, PARAM_ADDRESS, 0,
        ADC_ABS, PARAM_ADDRESS, 0,
        DEC_ABS, PARAM_ADDRESS, 0,
        BNE, -8i8 as u8,
        STA_ABS, RETURN_ADDRESS, 0,
        LDA_IMM, 1u8,
        STA_ABS, HALT_ADDRESS, 0
    );
}

#[test]
fn triangle_number_subroutine() {
    run!([20] -> [210];
        0 => {
            JSR, 0x34, 0x12,
            LDA_IMM, 1u8,
            STA_ABS, HALT_ADDRESS, 0
        }
        0x1234 => {
            LDA_IMM, 0u8,
            LDX_ABS, PARAM_ADDRESS, 0,
            ADC_ABS, PARAM_ADDRESS, 0,
            DEC_ABS, PARAM_ADDRESS, 0,
            BNE, -8i8 as u8,
            STA_ABS, RETURN_ADDRESS, 0,
            RTS
        }
    );
}

#[test]
fn fibonacci() {
    run!([11] -> [89];
        LDX_IMM, 0x01u8,
        STX_ZPA, 0xefu8,
        SEC,
        LDY_ABS, PARAM_ADDRESS, 0,
        TYA,
        SBC_IMM, 0x03u8,
        TAY,
        CLC,
        LDA_IMM, 0x02u8,
        STA_ZPA, 0xeeu8,
        LDX_ZPA, 0xeeu8,
        ADC_ZPA, 0xefu8,
        STA_ZPA, 0xeeu8,
        STX_ZPA, 0xefu8,
        DEY,
        BNE, -11i8 as u8,
        STA_ABS, RETURN_ADDRESS, 0,
        LDA_IMM, 1u8,
        STA_ABS, HALT_ADDRESS, 0
    );
}
