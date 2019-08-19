mod parser;

use std::fs::File;

fn main() -> Result<(), std::io::Error> {
    // TODO: improve error handling
    for mut path in std::env::args()
        .skip(1)
        .map(std::path::PathBuf::from)
        .filter(|path| match path.extension() {
            Some(ext) if ext == "asm" => true,
            _ => {
                eprintln!("{}: Unsupported format", path.display());
                false
            }
        })
    {
        let in_file = match File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{}: {}", path.display(), e);
                continue;
            }
        };

        if !path.set_extension("hack") {
            eprintln!("{}: Unable to construct output path", path.display());
            continue;
        }
        let out_file = match File::create(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("{}: {}", path.display(), e);
                continue;
            }
        };

        //println!("Processing {}", path.display());
        process_asm(in_file, out_file)?;
    }
    Ok(())
}

use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref PRE_SYMBOLS: HashMap<String, u16> = {
        let mut ps = HashMap::new();
        ps.insert(String::from("SP"), 0);
        ps.insert(String::from("LCL"), 1);
        ps.insert(String::from("ARG"), 2);
        ps.insert(String::from("THIS"), 3);
        ps.insert(String::from("THAT"), 4);
        ps.insert(String::from("R0"), 0);
        ps.insert(String::from("R1"), 1);
        ps.insert(String::from("R2"), 2);
        ps.insert(String::from("R3"), 3);
        ps.insert(String::from("R4"), 4);
        ps.insert(String::from("R5"), 5);
        ps.insert(String::from("R6"), 6);
        ps.insert(String::from("R7"), 7);
        ps.insert(String::from("R8"), 8);
        ps.insert(String::from("R9"), 9);
        ps.insert(String::from("R10"), 10);
        ps.insert(String::from("R11"), 11);
        ps.insert(String::from("R12"), 12);
        ps.insert(String::from("R13"), 13);
        ps.insert(String::from("R14"), 14);
        ps.insert(String::from("R15"), 15);
        ps.insert(String::from("SCREEN"), 0x4000);
        ps.insert(String::from("KBD"), 0x6000);
        ps
    };
}

fn process_asm(mut i: File, o: File) -> std::io::Result<()> {
    use std::io::Read;
    use std::io::Write;

    let mut asm = String::new();
    i.read_to_string(&mut asm)?;
    let instrs = parser::parse(&asm).unwrap();

    let mut symbols = PRE_SYMBOLS.clone();

    // Pass 1: discover symbols
    let mut addr = 0u16;
    for instr in instrs {
        match instr {
            parser::Instruction::Label(sym) => {
                if symbols.insert(sym.clone(), addr).is_some() {
                    panic!("Duplicate symbol ({}) found", sym);
                }
            }
            _ => addr += 1,
        }
    }

    // Pass 2: serialize to disk
    let mut o = std::io::BufWriter::new(o);
    addr = *symbols.get("R15").unwrap(); // Allocate variables after R15
    for instr in parser::parse(&asm).unwrap() {
        match instr {
            parser::Instruction::Label(_) => (),
            parser::Instruction::A(parser::Address::Symbol(sym)) => write!(
                o,
                "{}\n",
                parser::Instruction::A(parser::Address::Constant(match symbols.get(&sym) {
                    Some(&val) => val,
                    None => {
                        addr += 1;
                        symbols.insert(sym, addr);
                        addr
                    }
                }))
            )
            .unwrap(),
            _ => write!(o, "{}\n", instr).unwrap(),
        }
    }

    Ok(())
}
