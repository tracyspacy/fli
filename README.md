# fli
**fli** is a cli tool to list directory content

<img width="160" height="160" alt="mascot" src="https://github.com/user-attachments/assets/a0742301-5891-49a3-a746-29fa4b87eabc" />

# Why
I have access to my raspberry pi zero via ssh only. As for me it is hard to differentiate types of files etc based on colors, I needed something more readable and clear like picture/icon or emoji as I ended up with etc. And I decided to build fli and since `ls` is obviosly preinstalled on almost any machine, the second requirement for `fli` as a complimentary tool, is to be tiny. 

**Thus fli is a tiny (18KB), easy to read file listing tool.**

While working on fli, another aspect of my interest and motivation is to check if with Rust one can build coreutils-like tools, but faster and smaller. 

**Readability**: Nice readability thanks to use of emojis (📄 and 🗂️) instead of text coloring.

**Speed**: By default directory entries are streamed directly from `readdir()` to stdout without heap allocation.

**Size:** Since rust `std` contributes heavily to binary size, this project is `no_std` + `libc` (**it contains unsafe code blocks**).

**Binary size:**
- M series mac: **51 KB**,
- rpi zero w : **18 KB**.


#### Current display options:

**Default** `fli` : short (name and type) sorted by **name**
**FLAGS**:
- `-l` : long listing format (*name, type, metadata*). Default sorting is by **name**. 
- `-S `: with `-l` long listing format sorted by **size**, smallest first.
- `-t` : with `-l` long listing format sorted by **time**, oldest first.
- `-r` : reverse order while sorting.
- `-U` : do not sort, list entries in directory order. Alignment is fixed-sized (20 chars for size and n_link) - direct stream, no heap allocation.
- `-2` : text output for types :`</DIR> <FILE> <LINK>` instead of default emojis.
- `-0` : color output for types : Dir: `Blue`, File: `Green`, Link : `Cyan`.   

**Path can be specified as an argument:** 
`fli /` or `fli /target/release/fli`

New display options may be added soon.

### Build
**Build:** ```cargo build --release```

**Build with cross for raspberry pi zero w:** ```cross build --release --target arm-unknown-linux-gnueabihf```

**Copy to rpi** : ```scp /target/arm-unknown-linux-gnueabihf/release/fli <username>@<pi>.local: <target dir> ```


## View

<img width="1200" height="800" alt="fli" src="https://github.com/user-attachments/assets/f61beac0-a0a4-4869-b772-644e7fffbda5" />


### MISC

```zsh
//arm-linux-gnueabihf
file fli
fli: ELF 32-bit LSB pie executable, ARM, EABI5 version 1 (SYSV), dynamically linked, interpreter /lib/ld-linux-armhf.so.3, for GNU/Linux 4.19.255, stripped

readelf -d fli | grep NEEDED
0x00000001 (NEEDED)                     Shared library: [libc.so.6]
0x00000001 (NEEDED)                     Shared library: [libgcc_s.so.1]

size fli
text    data     bss     dec     hex filename
15308     520       4   15832    3dd8 fli
```


### Benchmarks
*it is faster than ls, but it is not the main focus. Still probably need a more sophisticated benchmark later*
```zsh
//20k empty files, sorted by name, macos m-series.
Benchmark 1: /Users/tracyspacy/Documents/GitHub/fli/target/release/fli -l -U
  Time (mean ± σ):      39.8 ms ±   0.6 ms    [User: 14.0 ms, System: 24.6 ms]
  Range (min … max):    38.5 ms …  42.6 ms    60 runs

Benchmark 2: /Users/tracyspacy/Documents/GitHub/fli/target/release/fli -l
  Time (mean ± σ):      46.3 ms ±   0.4 ms    [User: 19.6 ms, System: 25.3 ms]
  Range (min … max):    45.0 ms …  47.4 ms    55 runs

Benchmark 3: /Users/tracyspacy/Documents/GitHub/fli/target/release/fli -l -t
  Time (mean ± σ):      40.8 ms ±   0.6 ms    [User: 14.3 ms, System: 25.0 ms]
  Range (min … max):    39.4 ms …  42.8 ms    62 runs

Benchmark 4: ls -l
  Time (mean ± σ):     145.5 ms ±   1.0 ms    [User: 82.3 ms, System: 61.8 ms]
  Range (min … max):   144.6 ms … 148.3 ms    19 runs

Benchmark 5: ls -l -U
  Time (mean ± σ):     145.4 ms ±   0.3 ms    [User: 82.4 ms, System: 61.7 ms]
  Range (min … max):   144.6 ms … 146.0 ms    19 runs

Benchmark 6: ls -l -t
  Time (mean ± σ):      98.2 ms ±   0.5 ms    [User: 35.7 ms, System: 61.2 ms]
  Range (min … max):    96.8 ms …  99.3 ms    28 runs

Summary
  /Users/tracyspacy/Documents/GitHub/fli/target/release/fli -l -U ran
    1.02 ± 0.02 times faster than /Users/tracyspacy/Documents/GitHub/fli/target/release/fli -l -t
    1.16 ± 0.02 times faster than /Users/tracyspacy/Documents/GitHub/fli/target/release/fli -l
    2.47 ± 0.04 times faster than ls -l -t
    3.65 ± 0.06 times faster than ls -l -U
    3.65 ± 0.06 times faster than ls -l
```
