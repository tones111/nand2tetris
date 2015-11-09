// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Mult.asm

// Multiplies R0 and R1 and stores the result in R2.
// (R0, R1, R2 refer to RAM[0], RAM[1], and RAM[2], respectively.)

   @R2
   M=0

   @R1   // fast-path if R1 == 0
   D=M
   @exit
   D; JEQ

   @R0   // fast-path if R0 == 0
   D=M
   @exit
   D; JEQ

(loop)      // do
   @R1
   D=M
   @R2
   M=D+M    // R2 += R1
   @R0
   MD=M-1   // while (R0 > 0)
   @loop
   D; JNE

(exit)
   @exit
   0; JMP
