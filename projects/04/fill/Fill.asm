// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Fill.asm

// Runs an infinite loop that listens to the keyboard input.
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel;
// the screen should remain fully black as long as the key is pressed.
// When no key is pressed, the program clears the screen, i.e. writes
// "white" in every pixel;
// the screen should remain fully clear as long as no key is pressed.

// SCREEN = 0x4000
//    256 rows x 512 columns = 256 x 32 words (8192)
//    0, 0 => Top Left
//    Pixel = c%16 bit @(base + 32r + c/16)
// KBD = 0x6000

   @SCREEN
   D=A
   @ptr
   M=D   // ptr = SCREEN

   @8192
   D=D+A
   @eos
   M=D   // end-of-screen(eos) = SCREEN + 8192 (256x32)

(main)
   @KBD
   D=M
   @fill
   D; JEQ
   D=-1

(fill)
   @ptr
   A=M
   M=D

   @ptr
   D=M+1
   M=D     // ptr++

   @eos
   D=D-M
   @main
   D; JNE

   @SCREEN
   D=A
   @ptr
   M=D
   @main
   0; JMP
