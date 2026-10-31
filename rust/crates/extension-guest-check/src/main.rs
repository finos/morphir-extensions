//! `extension-guest-check claims <module.wasm>` prints the capability claim set
//! of a built guest, read with a live describe call.

use std::path::PathBuf;
use std::process::ExitCode;

use extension_guest_check::Guest;
use morphir_extension_sdk::claims::CapabilityClaimSet;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [command, module] = args.as_slice() else {
        eprintln!("usage: extension-guest-check claims <module.wasm>");
        return ExitCode::from(2);
    };
    if command != "claims" {
        eprintln!("unknown command {command:?}; usage: extension-guest-check claims <module.wasm>");
        return ExitCode::from(2);
    }
    match Guest::load(&PathBuf::from(module)).and_then(|mut guest| guest.describe()) {
        Ok(claims) => {
            if let Err(e) = serde_json::from_value::<CapabilityClaimSet>(claims.clone()) {
                eprintln!("{module}: describe did not answer a valid claim set: {e}");
                return ExitCode::FAILURE;
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&claims).expect("claims serialize")
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
