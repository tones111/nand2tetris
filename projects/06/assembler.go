package main

import (
	"bufio"
	"bytes"
	"fmt"
	"io"
	"os"
	"strconv"
	"strings"
)

type instruction struct {
	cmd    cmdType
	symbol string
	dest   destType
	comp   compType
	jump   jumpType
}

type cmdType uint

const (
	aCmd cmdType = iota
	cCmd
	lCmd
)

type destType int

const (
	aDest destType = 1 << iota
	dDest
	mDest
)

type binBool bool

type compType struct {
	a, c1, c2, c3, c4, c5, c6 binBool
}

type jumpType struct {
	j1, j2, j3 binBool
}

const t binBool = true
const f binBool = false

var destinations map[rune]destType = map[rune]destType{
	'A': aDest,
	'M': mDest,
	'D': dDest,
}

var commands map[string]compType = map[string]compType{
	"0":   {a: f, c1: t, c2: f, c3: t, c4: f, c5: t, c6: f},
	"1":   {a: f, c1: t, c2: t, c3: t, c4: t, c5: t, c6: t},
	"-1":  {a: f, c1: t, c2: t, c3: t, c4: f, c5: t, c6: f},
	"D":   {a: f, c1: f, c2: f, c3: t, c4: t, c5: f, c6: f},
	"A":   {a: f, c1: t, c2: t, c3: f, c4: f, c5: f, c6: f},
	"!D":  {a: f, c1: f, c2: f, c3: t, c4: t, c5: f, c6: t},
	"!A":  {a: f, c1: t, c2: t, c3: f, c4: f, c5: f, c6: t},
	"-D":  {a: f, c1: f, c2: f, c3: t, c4: t, c5: t, c6: t},
	"-A":  {a: f, c1: t, c2: t, c3: f, c4: f, c5: t, c6: t},
	"D+1": {a: f, c1: f, c2: t, c3: t, c4: t, c5: t, c6: t},
	"A+1": {a: f, c1: t, c2: t, c3: f, c4: t, c5: t, c6: t},
	"D-1": {a: f, c1: f, c2: f, c3: t, c4: t, c5: t, c6: f},
	"A-1": {a: f, c1: t, c2: t, c3: f, c4: f, c5: t, c6: f},
	"D+A": {a: f, c1: f, c2: f, c3: f, c4: f, c5: t, c6: f},
	"D-A": {a: f, c1: f, c2: t, c3: f, c4: f, c5: t, c6: t},
	"A-D": {a: f, c1: f, c2: f, c3: f, c4: t, c5: t, c6: t},
	"D&A": {a: f, c1: f, c2: f, c3: f, c4: f, c5: f, c6: f},
	"D|A": {a: f, c1: f, c2: t, c3: f, c4: t, c5: f, c6: t},
	"M":   {a: t, c1: t, c2: t, c3: f, c4: f, c5: f, c6: f},
	"!M":  {a: t, c1: t, c2: t, c3: f, c4: f, c5: f, c6: t},
	"-M":  {a: t, c1: t, c2: t, c3: f, c4: f, c5: t, c6: t},
	"M+1": {a: t, c1: t, c2: t, c3: f, c4: t, c5: t, c6: t},
	"M-1": {a: t, c1: t, c2: t, c3: f, c4: f, c5: t, c6: f},
	"D+M": {a: t, c1: f, c2: f, c3: f, c4: f, c5: t, c6: f},
	"D-M": {a: t, c1: f, c2: t, c3: f, c4: f, c5: t, c6: t},
	"M-D": {a: t, c1: f, c2: f, c3: f, c4: t, c5: t, c6: t},
	"D&M": {a: t, c1: f, c2: f, c3: f, c4: f, c5: f, c6: f},
	"D|M": {a: t, c1: f, c2: t, c3: f, c4: t, c5: f, c6: t},
}

var jumps map[string]jumpType = map[string]jumpType{
	"JGT": {j1: f, j2: f, j3: t},
	"JEQ": {j1: f, j2: t, j3: f},
	"JGE": {j1: f, j2: t, j3: t},
	"JLT": {j1: t, j2: f, j3: f},
	"JNE": {j1: t, j2: f, j3: t},
	"JLE": {j1: t, j2: t, j3: f},
	"JMP": {j1: t, j2: t, j3: t},
}

var symbols map[string]uint16 = map[string]uint16{
	"SP":     0x0000,
	"LCL":    0x0001,
	"ARG":    0x0002,
	"THIS":   0x0003,
	"THAT":   0x0004,
	"R0":     0x0000,
	"R1":     0x0001,
	"R2":     0x0002,
	"R3":     0x0003,
	"R4":     0x0004,
	"R5":     0x0005,
	"R6":     0x0006,
	"R7":     0x0007,
	"R8":     0x0008,
	"R9":     0x0009,
	"R10":    0x000A,
	"R11":    0x000B,
	"R12":    0x000C,
	"R13":    0x000D,
	"R14":    0x000E,
	"R15":    0x000F,
	"SCREEN": 0x4000,
	"KBD":    0x6000,
}

func (b binBool) String() string {
	if b {
		return "1"
	}
	return "0"
}

func (d destType) String() string {
	if d != 0 {
		return "1"
	}
	return "0"
}

func (i *instruction) String() string {
	switch i.cmd {
	case aCmd:
		return "@" + i.symbol
	case lCmd:
		return "(" + i.symbol + ")"
	case cCmd:
		var buf bytes.Buffer
		for k, v := range destinations {
			if i.dest&v != 0 {
				buf.WriteString(string(k))
			}
		}
		if i.dest != 0 {
			buf.WriteString("=")
		}

		for k, v := range commands {
			if i.comp == v {
				buf.WriteString(k)
				continue
			}
		}

		for k, v := range jumps {
			if i.jump == v {
				buf.WriteString(";")
				buf.WriteString(k)
				continue
			}
		}
		return buf.String()
	}
	return "Unknown Instruction"
}

var varAddr uint16 = 0x0010

func (i *instruction) toBinary() string {
	switch i.cmd {
	case aCmd:
		addr, err := strconv.Atoi(i.symbol)
		if err != nil {
			symAddr, ok := symbols[i.symbol]
			if !ok {
				symAddr = varAddr
				symbols[i.symbol] = symAddr
				varAddr++
			}
			addr = int(symAddr)
		}
		return fmt.Sprintf("0%015b", addr)
	case cCmd:
		return fmt.Sprintf("111%v%v%v%v%v%v%v%v%v%v%v%v%v",
			i.comp.a, i.comp.c1, i.comp.c2, i.comp.c3, i.comp.c4, i.comp.c5, i.comp.c6,
			i.dest&aDest, i.dest&dDest, i.dest&mDest,
			i.jump.j1, i.jump.j2, i.jump.j3)
	}
	return ""
}

func halt(mesg string) {
	fmt.Fprintln(os.Stderr, mesg)
	os.Exit(1)
}

func main() {
	inFile := os.Stdin
	if len(os.Args) > 1 {
		var err error
		if inFile, err = os.Open(os.Args[1]); err != nil {
			halt(err.Error())
		}
	}
	defer inFile.Close()

	instrs := make([]*instruction, 0, 32)
	lines := scanLines(inFile)

	// First-Pass: load symbol table
	var instrCount uint16
	for instr := range parse(lines) {
		switch instr.cmd {
		case aCmd, cCmd:
			instrs = append(instrs, instr)
			instrCount++

		case lCmd:
			if _, ok := symbols[instr.symbol]; ok {
				halt(fmt.Sprintf("Symbol (%s) already defined", instr.symbol))
			}
			symbols[instr.symbol] = instrCount
		}
	}

	// Second-Pass: generate binary
	for _, instr := range instrs {
		fmt.Println(instr.toBinary())
		//fmt.Println(instr)
	}
}

func scanLines(in io.Reader) <-chan string {
	lines := make(chan string)
	go func() {
		scanner := bufio.NewScanner(in)
		for scanner.Scan() {
			lines <- scanner.Text()
		}
		if err := scanner.Err(); err != nil {
			halt("Scan Error: " + err.Error())
		}
		close(lines)
	}()
	return lines
}

func parse(cmds <-chan string) <-chan *instruction {
	instrs := make(chan *instruction)

	go func() {
		const whitespace = " \t"
		var lineNum uint

		for cmd := range cmds {
			lineNum++

			// Strip end-of-line comments
			if i := strings.Index(cmd, "//"); i >= 0 {
				cmd = cmd[:i]
			}

			instr := &instruction{}
			switch s := strings.Trim(cmd, whitespace); {
			case len(s) == 0:
				continue

			case strings.HasPrefix(s, "@"):
				instr.cmd = aCmd
				instr.symbol = strings.Trim(s[1:], whitespace)
				if len(instr.symbol) == 0 {
					halt(fmt.Sprintf("line %d: address or label name expected", lineNum))
				} else if strings.ContainsAny(instr.symbol, whitespace) {
					halt(fmt.Sprintf("line %d: invalid label name", lineNum))
				}

			case strings.HasPrefix(s, "("):
				i := strings.Index(s, ")")
				if i == -1 {
					halt(fmt.Sprintf("line %d: closing \")\" expected", lineNum))
				}
				instr.cmd = lCmd
				instr.symbol = strings.Trim(s[1:i], whitespace)
				if len(instr.symbol) == 0 {
					halt(fmt.Sprintf("line %d: label name expected", lineNum))
				} else if strings.ContainsAny(instr.symbol, whitespace) {
					halt(fmt.Sprintf("line %d: invalid label name", lineNum))
				}

			default:
				instr.cmd = cCmd
				var comp string
				if tokens := strings.Split(s, "="); len(tokens) > 1 {
					dest := strings.Trim(tokens[0], whitespace)
					comp = strings.Trim(tokens[1], whitespace)
					for k, v := range destinations {
						if count := strings.Count(dest, string(k)); count == 1 {
							instr.dest |= v
						} else if count > 1 {
							halt(fmt.Sprintf("line %d: destination %s specified multiple times", lineNum, k))
						}
						dest = strings.Trim(dest, string(k))
					}
					if len(dest) > 0 {
						halt(fmt.Sprintf("line %d: unknown destination type specified", lineNum))
					}
					s = s[len(tokens[0]):]
				}

				var ok bool
				if tokens := strings.Split(s, ";"); len(tokens) > 1 {
					jump := strings.Trim(tokens[1], whitespace)
					comp2 := strings.Trim(tokens[0], whitespace)
					if len(comp) > 0 && comp != comp2 {
						halt(fmt.Sprintf("line %d: unable to determine computation", lineNum))
					} else {
						comp = comp2
					}
					if instr.jump, ok = jumps[jump]; !ok {
						halt(fmt.Sprintf("line %d: invalid jump type", lineNum))
					}
				}

				if len(comp) == 0 {
					comp = s
				}

				if instr.comp, ok = commands[comp]; !ok {
					halt(fmt.Sprintf("line %d: invalid label name", lineNum))
				}
			}
			instrs <- instr
		}
		close(instrs)
	}()

	return instrs
}
