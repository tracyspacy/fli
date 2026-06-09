# fli
cli tool to list directory content

<img width="160" height="160" alt="mascot" src="https://github.com/user-attachments/assets/a0742301-5891-49a3-a746-29fa4b87eabc" />

# Why
I built it to use on my raspberry pi zero which I access through ssh. 
The purpose of this project is to build file-listing cli tool with a good readability, tiny binary size and maximum execution speed. 
Speed is priority: by default directory entries are streamed directly from `readdir()` to stdout without heap allocation.
Since rust `std` contributes heavily to binary size, this project is `no_std` + `libc` (**it contains unsafe code blocks**).

**Current binary size:**
- M series mac: 51 KB,
- rpi zero w : 18 KB.

Current scope of features is intentionally limited to simple retrieval of directory content without sorting and with natural sorting by name with -s flag. New features such as displaying metadata and sorting by size will be added soon.   



<img width="495" height="222" alt="macos" src="https://github.com/user-attachments/assets/7296dff8-2715-4cf2-aa62-05d5936dc59c" />

<img width="197" height="416" alt="rpi zero w" src="https://github.com/user-attachments/assets/d6979f22-55ec-4491-accd-08bc0b587b50" />


**Sorted Long output:**

<img width="547" height="218" alt="Screenshot 2026-06-09 at 16 38 58" src="https://github.com/user-attachments/assets/f68b924c-d78a-4d4e-86d2-ea38533094de" />


**Benchmarks:**

<img width="731" height="261" alt="Screenshot 2026-06-09 at 16 39 55" src="https://github.com/user-attachments/assets/15b185e9-ae59-4cb1-ad2c-a737e15f7e17" />

