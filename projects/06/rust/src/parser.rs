#[derive(Debug)]
pub enum Instruction {
    A(Address),
    C(Compute),
    Label(String),
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::A(addr) => write!(f, "{}", addr),
            Instruction::C(comp) => write!(f, "{}", comp),
            Instruction::Label(_) => panic!("Labels have no binary representation"),
        }
    }
}

#[derive(Debug)]
pub enum Address {
    Constant(u16),
    Symbol(String),
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Address::Constant(val) => write!(f, "{:016b}", val & 0x7FFF),
            Address::Symbol(_) => panic!("Symbols have no binary representation"),
        }
    }
}

#[derive(Debug)]
pub struct Compute {
    dest: Option<Dest>,
    comp: Comp,
    jump: Option<Jump>,
}

impl std::fmt::Display for Compute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "1{unused:2b}{comp:07b}{dest:03b}{jump:03b}",
            unused = 0b11,
            comp = self.comp as u8 & 0x7F,
            dest = match self.dest {
                Some(d) => d as u8 & 0x07,
                _ => 0,
            },
            jump = match self.jump {
                Some(j) => j as u8 & 0x07,
                _ => 0,
            },
        )
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Dest {
    M = 1,
    D = 2,
    MD = 3,
    A = 4,
    AM = 5,
    AD = 6,
    AMD = 7,
}

#[derive(Copy, Clone, Debug)]
pub enum Jump {
    JGT = 1,
    JEQ = 2,
    JGE = 3,
    JLT = 4,
    JNE = 5,
    JLE = 6,
    JMP = 7,
}

#[rustfmt::skip]
#[derive(Copy, Clone, Debug)]
pub enum Comp {
   Zero    = 0b0101010,
   One     = 0b0111111,
   NegOne  = 0b0111010,
   D       = 0b0001100,
   A       = 0b0110000,
   NotD    = 0b0001101,
   NotA    = 0b0110001,
   NegD    = 0b0001111,
   NegA    = 0b0110011,
   IncD    = 0b0011111,
   IncA    = 0b0110111,
   DecD    = 0b0001110,
   DecA    = 0b0110010,
   DPlusA  = 0b0000010,
   DMinusA = 0b0010011,
   AMinusD = 0b0000111,
   DAndA   = 0b0000000,
   DOrA    = 0b0010101,
   M       = 0b1110000,
   NotM    = 0b1110001,
   NegM    = 0b1110011,
   IncM    = 0b1110111,
   DecM    = 0b1110010,
   DPlusM  = 0b1000010,
   DMinusM = 0b1010011,
   MMinusD = 0b1000111,
   DAndM   = 0b1000000,
   DOrM    = 0b1010101,
}

pub fn parse(asm: &str) -> Result<Vec<Instruction>, ()> {
    match read_lines(asm) {
        Ok((remain, instrs)) => {
            if remain.is_empty() {
                Ok(instrs)
            } else {
                Err(())
            }
        }
        Err(nom::Err::Failure(f)) => {
            eprintln!("failure parsing: {:?}", f);
            Err(())
        }
        Err(nom::Err::Error(e)) => {
            eprintln!("error parsing ({:?}). remaining:\n<{}>", e.1, e.0);
            Err(())
        }
        Err(e) => {
            eprintln!("other parsing issue: {:?}", e);
            Err(())
        }
    }
}

pub fn read_lines(input: &str) -> nom::IResult<&str, Vec<Instruction>> {
    nom::combinator::all_consuming(nom::multi::many0(read_line))(input)
}

pub fn read_line(input: &str) -> nom::IResult<&str, Instruction> {
    let mut next = input.clone();
    loop {
        let (input, instr) = nom::branch::alt((
            nom::combinator::map(parse_a, |a: Address| Some(Instruction::A(a))),
            nom::combinator::map(parse_c, |c: Compute| Some(Instruction::C(c))),
            nom::combinator::map(parse_label, |s: &str| {
                Some(Instruction::Label(String::from(s)))
            }),
            nom::combinator::map(nom::character::complete::space0, |_| None),
        ))(next)?;

        // Allow optional end-of-line comments
        let (input, _) = nom::character::complete::space0(input)?;
        let (input, _comment) = nom::combinator::opt(parse_comment)(input)?;
        let (input, _) = nom::character::complete::multispace1(input)?;

        if let Some(instr) = instr {
            return Ok((input, instr));
        }
        next = input.clone();
    }
}

fn parse_constant(input: &str) -> nom::IResult<&str, u16> {
    let (input, digits) = nom::character::complete::digit1(input)?;

    match u16::from_str_radix(digits, 10) {
        Ok(val) => Ok((input, val)),
        _ => Err(nom::Err::Error((input, nom::error::ErrorKind::Digit))),
    }
}

fn parse_symbol(input: &str) -> nom::IResult<&str, &str> {
    // Symbols may not begin with a digit
    nom::combinator::not(nom::character::complete::digit1)(input)?;

    nom::bytes::complete::take_while1(|b: char| match b {
        'a'..='z' => true,
        'A'..='Z' => true,
        '0'..='9' => true,
        '_' => true,
        '.' => true,
        '$' => true,
        ':' => true,
        _ => false,
    })(input)
}

pub fn parse_a(input: &str) -> nom::IResult<&str, Address> {
    nom::sequence::preceded(
        nom::bytes::complete::tag("@"),
        nom::branch::alt((
            nom::combinator::map(parse_constant, |val: u16| Address::Constant(val)),
            nom::combinator::map(parse_symbol, |s: &str| Address::Symbol(String::from(s))),
        )),
    )(input)
}

#[test]
fn test_parse_a() {
    assert_eq!(
        parse_a(""),
        Err(nom::Err::Error(("", nom::error::ErrorKind::Tag)))
    );
    assert_eq!(parse_a("@0"), Ok(("", Address::Constant(0))));
}

pub fn parse_label(input: &str) -> nom::IResult<&str, &str> {
    nom::sequence::delimited(
        nom::character::complete::char('('),
        parse_symbol,
        nom::character::complete::char(')'),
    )(input)
}

pub fn parse_c(input: &str) -> nom::IResult<&str, Compute> {
    let (input, dest) = nom::combinator::opt(parse_dest)(input)?;
    let (input, comp) = parse_comp(input)?;
    let (input, jump) = nom::combinator::opt(parse_jump)(input)?;

    Ok((input, Compute { dest, comp, jump }))
}

pub fn parse_dest(input: &str) -> nom::IResult<&str, Dest> {
    // Note: Longer matches need to be searched first due to shared prefixes
    nom::sequence::terminated(
        nom::branch::alt((
            nom::combinator::map(nom::bytes::complete::tag("AMD"), |_| Dest::AMD),
            nom::combinator::map(nom::bytes::complete::tag("AD"), |_| Dest::AD),
            nom::combinator::map(nom::bytes::complete::tag("AM"), |_| Dest::AM),
            nom::combinator::map(nom::bytes::complete::tag("MD"), |_| Dest::MD),
            nom::combinator::map(nom::bytes::complete::tag("A"), |_| Dest::A),
            nom::combinator::map(nom::bytes::complete::tag("D"), |_| Dest::D),
            nom::combinator::map(nom::bytes::complete::tag("M"), |_| Dest::M),
        )),
        nom::bytes::complete::tag("="),
    )(input)
}

pub fn parse_comp(input: &str) -> nom::IResult<&str, Comp> {
    // Note: Needed to split these due to alt list length limits
    // Note: Longer matches need to be searched first due to shared prefixes
    nom::branch::alt((
        // length 3
        nom::branch::alt((
            nom::combinator::map(nom::bytes::complete::tag("A+1"), |_| Comp::IncA),
            nom::combinator::map(nom::bytes::complete::tag("A-1"), |_| Comp::DecA),
            nom::combinator::map(nom::bytes::complete::tag("A-D"), |_| Comp::AMinusD),
            nom::combinator::map(nom::bytes::complete::tag("D&A"), |_| Comp::DAndA),
            nom::combinator::map(nom::bytes::complete::tag("D&M"), |_| Comp::DAndM),
            nom::combinator::map(nom::bytes::complete::tag("D+1"), |_| Comp::IncD),
            nom::combinator::map(nom::bytes::complete::tag("D+A"), |_| Comp::DPlusA),
            nom::combinator::map(nom::bytes::complete::tag("D+M"), |_| Comp::DPlusM),
            nom::combinator::map(nom::bytes::complete::tag("D-1"), |_| Comp::DecD),
            nom::combinator::map(nom::bytes::complete::tag("D-A"), |_| Comp::DMinusA),
            nom::combinator::map(nom::bytes::complete::tag("D-M"), |_| Comp::DMinusM),
            nom::combinator::map(nom::bytes::complete::tag("D|A"), |_| Comp::DOrA),
            nom::combinator::map(nom::bytes::complete::tag("D|M"), |_| Comp::DOrM),
            nom::combinator::map(nom::bytes::complete::tag("M+1"), |_| Comp::IncM),
            nom::combinator::map(nom::bytes::complete::tag("M-1"), |_| Comp::DecM),
            nom::combinator::map(nom::bytes::complete::tag("M-D"), |_| Comp::MMinusD),
        )),
        // length 2
        nom::branch::alt((
            nom::combinator::map(nom::bytes::complete::tag("!A"), |_| Comp::NotA),
            nom::combinator::map(nom::bytes::complete::tag("!D"), |_| Comp::NotD),
            nom::combinator::map(nom::bytes::complete::tag("!M"), |_| Comp::NotM),
            nom::combinator::map(nom::bytes::complete::tag("-1"), |_| Comp::NegOne),
            nom::combinator::map(nom::bytes::complete::tag("-A"), |_| Comp::NegA),
            nom::combinator::map(nom::bytes::complete::tag("-D"), |_| Comp::NegD),
            nom::combinator::map(nom::bytes::complete::tag("-M"), |_| Comp::NegM),
        )),
        // length 1
        nom::branch::alt((
            nom::combinator::map(nom::bytes::complete::tag("0"), |_| Comp::Zero),
            nom::combinator::map(nom::bytes::complete::tag("1"), |_| Comp::One),
            nom::combinator::map(nom::bytes::complete::tag("A"), |_| Comp::A),
            nom::combinator::map(nom::bytes::complete::tag("D"), |_| Comp::D),
            nom::combinator::map(nom::bytes::complete::tag("M"), |_| Comp::M),
        )),
    ))(input)
}

pub fn parse_jump(input: &str) -> nom::IResult<&str, Jump> {
    nom::sequence::preceded(
        nom::bytes::complete::tag(";"),
        nom::branch::alt((
            nom::combinator::map(nom::bytes::complete::tag("JEQ"), |_| Jump::JEQ),
            nom::combinator::map(nom::bytes::complete::tag("JGE"), |_| Jump::JGE),
            nom::combinator::map(nom::bytes::complete::tag("JGT"), |_| Jump::JGT),
            nom::combinator::map(nom::bytes::complete::tag("JLE"), |_| Jump::JLE),
            nom::combinator::map(nom::bytes::complete::tag("JLT"), |_| Jump::JLT),
            nom::combinator::map(nom::bytes::complete::tag("JMP"), |_| Jump::JMP),
            nom::combinator::map(nom::bytes::complete::tag("JNE"), |_| Jump::JNE),
        )),
    )(input)
}

pub fn parse_comment(input: &str) -> nom::IResult<&str, &str> {
    nom::sequence::preceded(
        nom::bytes::complete::tag("//"),
        nom::character::complete::not_line_ending,
    )(input)
}
