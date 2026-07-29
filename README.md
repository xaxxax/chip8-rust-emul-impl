# CHIP-8 Emulator — in Rust

> A software recreation of the CHIP-8 virtual machine. When it's done, it loads a
> real ROM file and you watch a 1970s game — Pong, Tetris, Space Invaders — render
> and play in a window. You will have built a working CPU, entirely in software.

This repo is a **project brief, not a tutorial and not a scaffold.** The code is
yours to write. This README exists so you can turn it into user stories and work
through them ticket by ticket, the way you would on a real team.

---

## 1. Why this project

You're building this to get two things:

1. **A concrete understanding of how a CPU works.** By the end you'll *feel* why a
   processor is "just a loop that reads a number and does what the number says" —
   fetch, decode, execute, repeat. CHIP-8 is the smallest honest example of this.
2. **Real Rust, on a problem that suits it.** Opcode decoding is a showcase for
   Rust's `enum` + `match`; the machine's state is a showcase for ownership. You'll
   write idiomatic Rust, not Rust-that-looks-like-C.

It's also deliberately **finishable** — a clear finish line, reachable in days-to-
weeks, not months. That matters more than anything.

---

## 2. Definition of Done

The project is **done** when:

- [ ] It loads an arbitrary CHIP-8 ROM passed on the command line.
- [ ] It renders the 64×32 display in a window.
- [ ] It reads keyboard input mapped to the CHIP-8 keypad.
- [ ] It runs at a sensible, roughly authentic speed with working timers.
- [ ] At least one real game (e.g. Pong) is **playable**.
- [ ] The README has a screenshot or GIF of it running.

**First visible milestone** (aim for this early — it's hugely motivating): the
**IBM logo ROM** renders the letters "IBM" on screen. It only exercises a handful
of instructions, so it's the natural "hello world" of CHIP-8 and proves your core
loop + display work before you've implemented the full instruction set.

---

## 3. The machine you're modeling (reference)

CHIP-8 is tiny. This is the whole architecture — treat it as the spec, but **you**
decide how to represent each piece in Rust.

| Component | What it is |
|---|---|
| **Memory** | 4096 bytes (4 KB). Programs are loaded starting at address `0x200`. The bytes below that were the original interpreter; you'll put the font there. |
| **Registers** | 16 general-purpose 8-bit registers, `V0`–`VF`. `VF` doubles as a flag register (carry, collision) — don't use it for general storage. |
| **Index register `I`** | One 16-bit register, used to point at memory addresses. |
| **Program counter (PC)** | Points at the current instruction. Starts at `0x200`. |
| **Stack** | Holds return addresses for `CALL`/`RET`. 16 levels is plenty. Needs a stack pointer. |
| **Delay timer** | 8-bit, counts down at 60 Hz. Used for timing/events. |
| **Sound timer** | 8-bit, counts down at 60 Hz. Beeps while non-zero. |
| **Display** | 64×32 monochrome pixels. Drawing is done by **XOR-ing** sprites onto it. |
| **Keypad** | 16 keys (`0x0`–`0xF`), laid out as a 4×4 grid. |
| **Font** | 16 built-in 5-byte sprites for the hex digits `0`–`F`. You load these into low memory yourself. |

Each **instruction is 2 bytes** (big-endian), so an "opcode" is a 16-bit value.
Instructions are grouped by their top nibble; the remaining nibbles encode
registers and values. You will look up the exact bit layout of each opcode in the
reference below — that lookup, per instruction, *is* the decode work.

---

## 4. References (consult on demand — do NOT read cover-to-cover first)

Keep these open and look things up **as each ticket needs them.** Reading them all
up front is procrastination in disguise; you retain it only once you have a bug to
hang it on.

- **Cowgod's CHIP-8 Technical Reference** — the canonical opcode table. Your
  primary lookup: for each instruction ticket, find its entry here.
- **Tobias Langhoff, "Guide to making a CHIP-8 emulator"** — the best modern
  walkthrough of the *structure* and the gotchas. Read a section when you reach it.
- **Timendus' `chip8-test-suite`** — test ROMs (including the IBM logo and an
  opcode test) that tell you exactly what's broken. Get these early.

Public-domain game ROMs (Pong, Tetris, etc.) are easy to find — grab a few `.ch8`
files and drop them in a `roms/` folder (git-ignore it if the licensing is unclear).

---

## 5. Milestone backlog (turn these into user stories)

These are **epics**, roughly ordered. Break each into your own tickets with
acceptance criteria. The point isn't to follow this exactly — it's to have a
backlog you groom and pull from.

### Epic 0 — Setup & tooling
- Decide your module layout (e.g. where the machine, the CPU loop, the display, and
  input live). This is a real design decision — see §6.
- Get test ROMs and at least one game ROM into the project.
- Decide how you'll run tests (unit tests on instructions are very doable).

### Epic 1 — The machine model
- Represent memory, registers, `I`, PC, stack + SP, and the two timers.
- Load the built-in font into low memory.
- Load a ROM file from disk into memory at `0x200`.
- *Acceptance:* you can construct a fresh machine and load a ROM without panicking,
  and a test confirms the ROM bytes landed at the right address.

### Epic 2 — The core loop (fetch → decode → execute)
- **Fetch:** read the 2 bytes at PC, combine into a 16-bit opcode.
- **Decode:** turn that opcode into something you can act on. *How* you represent a
  decoded instruction is the central design decision of the project — see §6.
- **Execute:** carry out the instruction and advance PC correctly (mind the ones
  that jump vs. the ones that fall through).
- *Acceptance:* the loop steps through a ROM, PC advances as expected, and an
  unrecognized opcode fails **loudly** (not silently) so you can spot gaps.

### Epic 3 — The instruction set (many small tickets)
Implement instructions in groups; each group (or even each opcode) is a ticket.
Get the exact bit layout of each from Cowgod's reference.
- **Control flow:** clear screen, jump, call/return, the skip-if instructions.
- **Registers & arithmetic:** set, add, the ALU ops (`OR`/`AND`/`XOR`/`ADD`/`SUB`/
  shifts) — these set the carry/borrow flag in `VF`; getting `VF` right is the
  fiddly part.
- **Index & memory:** set `I`, register store/load, binary-coded-decimal.
- **Timers:** read/write delay and sound timers.
- **Display:** the `DRAW` instruction — XOR sprites onto the screen and set `VF` on
  collision. This is the single trickiest instruction; give it its own ticket.
- **Input:** skip-if-key, and the blocking wait-for-key.
- **Random:** the RNG instruction.
- *Acceptance:* the IBM logo ROM renders (early), then the opcode test ROM passes.

### Epic 4 — I/O: make it real
- **Display:** choose a windowing/pixel crate (see §7) and draw the 64×32 buffer,
  scaled up so it's visible.
- **Input:** map your host keyboard to the 16-key CHIP-8 keypad.
- **Timing:** run the CPU at roughly the right rate (a few hundred instructions per
  second is typical) with timers ticking at 60 Hz. Decoupling "CPU speed" from
  "timer speed" is a real decision.
- *Acceptance:* a game is on screen and responds to your keypresses.

### Epic 5 — Polish & Definition of Done
- **Sound:** beep while the sound timer is non-zero.
- **Quirks:** a few CHIP-8 instructions are historically ambiguous (shift behavior,
  memory-load incrementing `I`, etc.). Decide how you handle them — ideally make the
  contentious ones configurable. The test suite will flag these.
- Capture a screenshot/GIF; finish the README.

### Stretch (only after Done)
- Super-CHIP (SCHIP) extended instructions and larger display.
- Then graduate to a **Game Boy** emulator — same loop, a real CPU.

---

## 6. Design decisions that are yours (don't let me make them)

These are the choices where the learning lives. Decide them deliberately and write
down *why* in your tickets:

1. **How do you represent a decoded instruction?** A big `match` directly on the
   opcode nibbles? An `enum Instruction` with a decode step that returns it, then a
   separate execute step? The `enum` route is more Rust-idiomatic and testable; the
   direct-match route is faster to write. There's a real trade-off.
2. **Module structure.** One file, or separate modules for cpu / memory / display /
   input? When does splitting help vs. add ceremony?
3. **Error handling.** When you hit an unknown opcode or a bad ROM, do you `panic!`,
   return a `Result`, or log and continue? This shapes how debuggable it is.
4. **Where does the display buffer live**, and who owns it — the CPU, or a separate
   type the CPU borrows? An ownership decision with real consequences.
5. **How do you decouple CPU speed from the 60 Hz timers** and from screen redraws?

---

## 7. Suggested dependencies (add them when a ticket needs them)

`Cargo.toml` starts empty on purpose. Add crates as you reach the milestone that
needs them, not before:

- **Display/window + input:** `minifb` is the simplest (a pixel buffer + a window +
  key state, minimal ceremony). `pixels` + `winit` is more powerful and more modern
  but heavier. `sdl2` is the classic but needs a native library installed. Pick one
  when you reach Epic 4 and justify it in the ticket.
- **CLI args:** the standard library's `std::env::args` is enough; reach for `clap`
  only if you want flags.

Keeping the dependency list short is itself good practice — every crate is a thing
you now depend on.

---

## 8. Build & run

```bash
cargo run --release -- path/to/rom.ch8    # release build — emulation wants the speed
cargo test                                # run your instruction tests
cargo clippy                              # lint; treat its suggestions as a mentor
cargo fmt                                 # format before committing
```

---

## 9. Suggested workflow (ticket-based, like a real job)

- One branch per ticket: `feat/fetch-decode-loop`, `feat/draw-instruction`, …
- Small commits with clear messages; open the branch, do the ticket, merge, delete.
- Write the acceptance criteria *before* you write the code, and don't close the
  ticket until they're met (a passing test or a visible result).
- Keep a short backlog file or issue tracker; groom it as you learn what's actually
  involved.

The goal is not just a working emulator — it's the habit of taking one well-scoped
piece of work all the way to *done*, then pulling the next.

---

Now go write **Epic 0**. Nothing in `src/` is wired up — that's the point.
