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
- `-U` : do not sort, list entries in directory order. Alignment is fixed-sized (20 chars for size and n_link) - direct stream, no heap allocation.
- `-2` : text output for types :`</DIR> <FILE> <LINK>` instead of default emojis.
- `-0` : color output for types : Dir: `Blue`, File: `Green`, Link : `Cyan`.   

New display options may be added soon.

### Build
**Build:** ```cargo build --release```

**Build with cross for raspberry pi zero w:** ```cross build --release --target arm-unknown-linux-gnueabihf```

**Copy to rpi** : ```scp /target/arm-unknown-linux-gnueabihf/release/fli <username>@<pi>.local: <target dir> ```


## View

<img width="495" height="222" alt="macos" src="https://github.com/user-attachments/assets/7296dff8-2715-4cf2-aa62-05d5936dc59c" />

<img width="197" height="416" alt="rpi zero w" src="https://github.com/user-attachments/assets/d6979f22-55ec-4491-accd-08bc0b587b50" />


**Sorted by name Long output:**

<img width="547" height="218" alt="Screenshot 2026-06-09 at 16 38 58" src="https://github.com/user-attachments/assets/f68b924c-d78a-4d4e-86d2-ea38533094de" />


**Sorted by size Long output:**

<img width="548" height="201" alt="Screenshot 2026-06-19 at 21 30 28" src="https://github.com/user-attachments/assets/3a409708-ee86-434b-a376-8fd6c26f4f2c" />


**Symlink with path**

<img width="838" height="101" alt="Screenshot 2026-06-19 at 21 27 07" src="https://github.com/user-attachments/assets/4943cdde-80d8-486c-9bf1-feebd8a0e75a" />


### Benchmarks

<img width="707" height="339" alt="Screenshot 2026-06-19 at 21 35 22" src="https://github.com/user-attachments/assets/3e575759-6b4b-4fb5-845b-3fb6f558ad26" />
