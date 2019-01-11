use nes_rust::mem;
use nes_rust::Address;
use nes_rust::OpCode::*;
use nes_rust::CPU;

const HALT_ADDRESS: Address = Address::new(0xDEAD);

macro_rules! run {
    ($result: expr; $( $expr: expr ),*) => {
        let mut cpu = CPU::with_memory(&mem!($($expr),*));
        let result = run(&mut cpu);
        assert_eq!(result, $result);
    };
}

fn run(cpu: &mut CPU) -> u8 {
    const MAX_INSTRUCTIONS: u32 = 1_000;

    let mut instructions = 0;

    while cpu.read(HALT_ADDRESS) == 0 {
        cpu.run_instruction();

        instructions += 1;

        if instructions > MAX_INSTRUCTIONS {
            panic!("Exceeded maximum number of instructions");
        }
    }

    cpu.accumulator()
}

#[test]
fn one_plus_one() {
    run!(2;
        LDAImmediate, 1u8,
        ADCImmediate, 1u8,
        STAAbsolute, HALT_ADDRESS
    );
}

#[test]
fn seven_times_six() {
    run!(42;
        LDAImmediate, 0u8,
        LDXImmediate, 6u8,
        ADCImmediate, 7u8,
        DEX,
        BNE, -5i8,
        STAAbsolute, HALT_ADDRESS
    );
}
