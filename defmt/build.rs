use std::{env, error::Error, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    if env::var("CARGO_CFG_TARGET_OS") == Ok("windows".to_string()) {
        return Err(r#"Compiling for Windows as a target is not supported.
Change your target to an embedded device, such as thumbv7em-none-eabi.
For more details see: https://github.com/knurling-rs/defmt/issues/463"#
            .into());
    }

    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var("OUT_DIR")?);
    let linker_script = fs::read_to_string("defmt.x.in")?;
    fs::write(out.join("defmt.x"), linker_script)?;
    println!("cargo:rustc-link-search={}", out.display());
    let target = env::var("TARGET")?;

    // `"atomic-cas": false` in `--print target-spec-json`
    // last updated: rust 1.48.0
    match &target[..] {
        "avr-gnu-base"
        | "msp430-none-elf"
        | "riscv32i-unknown-none-elf"
        | "riscv32imc-unknown-none-elf"
        | "thumbv4t-none-eabi"
        | "thumbv6m-none-eabi" => {
            println!("cargo:rustc-cfg=no_cas");
        }
        _ => {}
    }
    Ok(())
}
