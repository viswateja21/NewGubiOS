#![allow(bad_asm_style)]
#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(global_asm)]

use core::arch::global_asm;
use core::num;
use core::panic::PanicInfo;

// Include boot.s which defines _start as inline assembly in main. This allows us to do more fine
// grained setup than if we used a naked _start function in rust. Theoretically we could use a
// naked function + some inline asm, but this seems much more straight forward.

#[repr(C,packed)]
struct MultibootMmapEntry
{
  size: u32,
  addr: u64,
  len: u64,
  types : u32,
}

#[repr(C,packed)]
pub struct MultibootInfo
{
  /* Multiboot info version number */
  flags: u32,

  /* Available memory from BIOS */
  mem_lower: u32,
  mem_upper: u32,

  /* "root" partition */
  boot_device: u32,

  /* Kernel command line */
  cmdline: u32,

  /* Boot-Module list */
  mods_count : u32,
  mods_addr : u32,

  dummy : [u8;16],

  /* Memory Mapping buffer */
  mmap_length : u32,
  mmap_addr : u32,

  /* Drive Info buffer */
  drives_length : u32,
  drives_addr : u32,

  /* ROM configuration table */
  config_table : u32,

  /* Boot Loader Name */
  boot_loader_name : *const u8,

  /* APM table */
  apm_table : u32,

}

// #[repr(C, packed)]
// pub struct MultibootInfo{
//     flags: u32,
//     mem_lower:u32,
//     mem_upper: u32,
//     boot_device: u32,
//     cmdline: u32,
//     mods_count: u16,
//     mods_addr: u16,

// }

global_asm!(include_str!("boot.s"));

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let c = c as u8;
    for i in 0..n {
        *s.add(i) = c;
    }
    s
}

fn print_stuff(arg: i32){
    // let mut terminalWriter =TerminalWriter::new();
    // terminalWriter.putint(arg);
}

#[no_mangle]
pub extern "C" fn kernel_main(multibootmagic: u32, info: *const MultibootInfo) -> i32 {
    let mut terminal_writer = TerminalWriter::new();
    // terminal_writer.write(b"We did it, a rust kernel!\n");
    unsafe {
        for i in 0..(*info).mmap_length {
            let p = ((*info).mmap_addr + (core::mem::size_of::<MultibootMmapEntry>() as u32 * (i))) as *const MultibootMmapEntry;
            terminal_writer.write(b"len: ");
            terminal_writer.put_u32((*p).len as u32);
            terminal_writer.write(b" addr: ");
            terminal_writer.put_u32((*p).addr as u32);
            terminal_writer.write(b"\n");
        }
    }
    return 0;
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/* Hardware text mode color constants. */
#[allow(dead_code)]
enum VgaColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGrey = 7,
    DarkGrey = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    LightBrown = 14,
    White = 15,
}

fn vga_entry_color(fg: VgaColor, bg: VgaColor) -> u8 {
    fg as u8 | (bg as u8) << 4
}

fn vga_entry(uc: u8, color: u8) -> u16 {
    uc as u16 | (color as u16) << 8
}

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

struct TerminalWriter {
    terminal_row: usize,
    terminal_column: usize,
    terminal_color: u8,
    terminal_buffer: *mut u16,
}

impl TerminalWriter {
    fn new() -> TerminalWriter {
        let terminal_row = 0;
        let terminal_column = 0;
        let terminal_color =
            vga_entry_color(VgaColor::LightGrey, VgaColor::Black);
        let terminal_buffer = 0xB8000 as *mut u16;
        for y in 0..VGA_HEIGHT {
            for x in 0..VGA_WIDTH {
                let index = y * VGA_WIDTH + x;
                unsafe {
                    *terminal_buffer.add(index) = vga_entry(b' ', terminal_color);
                }
            }
        }

        TerminalWriter {
            terminal_row,
            terminal_column,
            terminal_color,
            terminal_buffer,
        }
    }

    #[allow(dead_code)]
    fn set_color(&mut self, color: u8) {
        self.terminal_color = color;
    }

    fn putentryat(&mut self, c: u8, color: u8, x: usize, y: usize) {
        let index = y * VGA_WIDTH + x;
        unsafe {
            *self.terminal_buffer.add(index) = vga_entry(c, color);
        }
    }

    fn put_u64(&mut self, character: u64) {
       self.put_u32((character >> 32) as u32);
       self.put_u32(character as u32);
    }

    fn put_u32(&mut self, character: u32) {
        let mut numofdigits =1;
        loop {
            if character / 10_u32.pow(numofdigits) == 0 {
                break
            }
            numofdigits += 1;
        }
        for digit in (1..=numofdigits).rev(){
            let character = (character / 10_u32.pow(digit - 1)) % 10;
            self.putchar((character + 0x30) as u8);
        }   
    }

    fn put_i32(&mut self, mut character: i32) {
        if (character < 0){
            self.putchar(b'-');
            character = -character;
        }
        self.put_u32(character as u32);
        
    }

    fn putchar(&mut self, c: u8) {
        if b'\n' == c {
            self.terminal_row +=1;
            self.terminal_column = 0;
            return;
        }
        self.putentryat(
            c,
            self.terminal_color,
            self.terminal_column,
            self.terminal_row,
        );
        self.terminal_column += 1;
        if self.terminal_column == VGA_WIDTH {
            self.terminal_column = 0;
            self.terminal_row += 1;
            if self.terminal_row == VGA_HEIGHT {
                self.terminal_row = 0;
            }
        }
    }

    fn write(&mut self, data: &[u8]) {
        for c in data {
            self.putchar(*c);
        }
    }
}
